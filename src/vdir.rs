use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::card::{Card, Defect};

#[derive(Debug, Default)]
pub struct AddressBook {
    pub dir: PathBuf,
    pub cards: Vec<(Location, Card)>,
    pub files: HashMap<PathBuf, Vec<Vec<u8>>>,
    pub skipped: Vec<Skipped>,
}

pub type Location = (PathBuf, usize);

#[derive(Debug)]
pub struct Skipped {
    pub path: PathBuf,
    pub position: Option<usize>,
    pub defect: Defect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    Changed,
    Deleted,
}

pub fn conflict(path: &Path, loaded: &[u8]) -> io::Result<Option<Conflict>> {
    match fs::read(path) {
        Ok(bytes) if bytes == loaded => Ok(None),
        Ok(_) => Ok(Some(Conflict::Changed)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Some(Conflict::Deleted)),
        Err(err) => Err(err),
    }
}

pub fn load(dir: &Path) -> io::Result<AddressBook> {
    let mut book = AddressBook {
        dir: dir.to_owned(),
        ..AddressBook::default()
    };
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "vcf") {
            continue;
        }
        let bytes = fs::read(&path)?;
        let chunks: Vec<Vec<u8>> = split(&bytes).into_iter().map(<[u8]>::to_vec).collect();
        let is_bundle = chunks.len() > 1;
        for (index, chunk) in chunks.iter().enumerate() {
            match Card::parse(chunk) {
                Ok(card) => book.cards.push(((path.clone(), index), card)),
                Err(defect) => book.skipped.push(Skipped {
                    path: path.clone(),
                    position: is_bundle.then_some(index + 1),
                    defect,
                }),
            }
        }
        book.files.insert(path, chunks);
    }
    book.skipped.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(book)
}

pub fn split(bytes: &[u8]) -> Vec<&[u8]> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut offset = 0;
    let mut has_begin = false;
    for line in bytes.split_inclusive(|&b| b == b'\n') {
        if line.trim_ascii_end().eq_ignore_ascii_case(b"BEGIN:VCARD") {
            if has_begin {
                chunks.push(&bytes[start..offset]);
                start = offset;
            }
            has_begin = true;
        }
        offset += line.len();
    }
    chunks.push(&bytes[start..]);
    chunks
}

pub fn save(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    let dir = path.parent().expect("card path is inside the address book");
    let result = write_synced(&temp, bytes, path)
        .and_then(|()| fs::rename(&temp, path))
        .and_then(|()| File::open(dir)?.sync_all());
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn write_synced(temp: &Path, bytes: &[u8], original: &Path) -> io::Result<()> {
    let mut file = File::create(temp)?;
    match fs::metadata(original) {
        Ok(metadata) => file.set_permissions(metadata.permissions())?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    file.write_all(bytes)?;
    file.sync_all()
}

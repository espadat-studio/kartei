use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::card::{Card, Defect};

#[derive(Debug, Default)]
pub struct AddressBook {
    pub path: PathBuf,
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

pub fn load(path: &Path) -> io::Result<AddressBook> {
    let paths = if fs::metadata(path)?.is_dir() {
        fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()?
            .into_iter()
            .filter(|path| is_vcf(path))
            .collect()
    } else if is_vcf(path) {
        vec![path.to_owned()]
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory or .vcf file",
        ));
    };
    let mut book = AddressBook {
        path: path.to_owned(),
        ..AddressBook::default()
    };
    for path in paths {
        let bytes = fs::read(&path)?;
        let chunks: Vec<Vec<u8>> = split(&bytes).into_iter().map(<[u8]>::to_vec).collect();
        let (cards, skipped) = parse(&path, &chunks);
        book.cards.extend(cards);
        book.skipped.extend(skipped);
        book.files.insert(path, chunks);
    }
    book.skipped.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(book)
}

pub fn parse(path: &Path, chunks: &[Vec<u8>]) -> (Vec<(Location, Card)>, Vec<Skipped>) {
    let is_bundle = chunks.len() > 1;
    let mut cards = Vec::new();
    let mut skipped = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        match Card::parse(chunk) {
            Ok(card) => cards.push(((path.to_owned(), index), card)),
            Err(defect) => skipped.push(Skipped {
                path: path.to_owned(),
                position: is_bundle.then_some(index + 1),
                defect,
            }),
        }
    }
    (cards, skipped)
}

pub(crate) fn is_vcf(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "vcf")
}

pub fn eol(bytes: &[u8]) -> &'static str {
    match bytes.iter().position(|&b| b == b'\n') {
        Some(i) if i > 0 && bytes[i - 1] != b'\r' => "\n",
        _ => "\r\n",
    }
}

pub fn append(mut chunks: Vec<Vec<u8>>, card: Vec<u8>) -> Vec<Vec<u8>> {
    let eol = eol(&chunks.concat());
    if let Some(last) = chunks.last_mut()
        && !last.is_empty()
        && !last.ends_with(b"\n")
    {
        last.extend_from_slice(eol.as_bytes());
    }
    chunks.push(card);
    chunks
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
    if start < bytes.len() {
        chunks.push(&bytes[start..]);
    }
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

pub fn remove(path: &Path) -> io::Result<()> {
    fs::remove_file(path)?;
    File::open(path.parent().expect("card path is inside the address book"))?.sync_all()
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

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::card::{Card, Defect};

#[derive(Debug, Default)]
pub struct AddressBook {
    pub cards: Vec<(PathBuf, Card)>,
    pub skipped: Vec<Skipped>,
}

#[derive(Debug)]
pub struct Skipped {
    pub path: PathBuf,
    pub defect: Defect,
}

pub fn load(dir: &Path) -> io::Result<AddressBook> {
    let mut book = AddressBook::default();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "vcf") {
            continue;
        }
        match Card::parse(&fs::read(&path)?) {
            Ok(card) => book.cards.push((path, card)),
            Err(defect) => book.skipped.push(Skipped { path, defect }),
        }
    }
    book.skipped.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(book)
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
    let permissions = fs::metadata(original)?.permissions();
    let mut file = File::create(temp)?;
    file.set_permissions(permissions)?;
    file.write_all(bytes)?;
    file.sync_all()
}

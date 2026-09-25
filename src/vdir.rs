use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::card::{Card, Defect};

#[derive(Debug, Default)]
pub struct AddressBook {
    pub cards: Vec<Card>,
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
            Ok(card) => book.cards.push(card),
            Err(defect) => book.skipped.push(Skipped { path, defect }),
        }
    }
    book.skipped.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(book)
}

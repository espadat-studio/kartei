use std::fs;
use std::io;
use std::path::Path;

use crate::card::Card;

pub fn load(dir: &Path) -> io::Result<Vec<Card>> {
    let mut cards = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "vcf") {
            cards.extend(Card::parse(&fs::read(&path)?).ok());
        }
    }
    Ok(cards)
}

use std::fs;
use std::path::Path;

use kartei::card::{Card, Labeled};

fn fixture(name: &str) -> Card {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    Card::parse(&fs::read(path).unwrap())
}

fn labeled(values: &[Labeled<String>]) -> Vec<(Option<&str>, &str)> {
    values
        .iter()
        .map(|v| (v.label.as_deref(), v.value.as_str()))
        .collect()
}

#[test]
fn apple_grouped_labels_are_decoded() {
    let card = fixture("apple-grouped-labels.vcf");
    assert_eq!(
        labeled(&card.phones()),
        [
            (Some("Mobile"), "+34 600 111 222"),
            (Some("Gym"), "+34 600 333 444"),
            (Some("cell"), "+34 600 555 666"),
        ]
    );
    assert_eq!(
        labeled(&card.emails()),
        [
            (Some("home"), "gabi@example.org"),
            (Some("Work"), "gabi@work.example"),
        ]
    );
}

#[test]
fn values_without_type_or_group_have_no_label() {
    let card = Card::parse(b"BEGIN:VCARD\r\nTEL:1\r\nitem1.EMAIL:a@b\r\nEND:VCARD\r\n");
    assert_eq!(labeled(&card.phones()), [(None, "1")]);
    assert_eq!(labeled(&card.emails()), [(None, "a@b")]);
}

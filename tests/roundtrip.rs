use std::fs;
use std::path::Path;

use kartei::card::Card;
use proptest::prelude::*;

fn fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .unwrap()
}

#[test]
fn every_fixture_round_trips_byte_for_byte() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("bad-")
        {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        let card = Card::parse(&bytes).unwrap();
        assert_eq!(card.to_bytes(), bytes, "{}", path.display());
    }
}

#[test]
fn folded_photo_card_reads_its_fields() {
    let card = Card::parse(&fixture("folded-photo.vcf")).unwrap();
    assert_eq!(card.display_name(), "Paula Photo");
    assert_eq!(card.phones()[0].value, "+49 170 1234567");
    let photo = card.lines().iter().find(|l| l.name() == "PHOTO").unwrap();
    assert!(photo.value().ends_with("AAAAAAAA/9k="));
    assert!(!photo.value().contains(' '));
}

#[test]
fn lf_card_reads_its_fields() {
    let card = Card::parse(&fixture("lf-endings.vcf")).unwrap();
    assert_eq!(card.display_name(), "Lena Linefeed");
    assert_eq!(card.structured_name(), ("Linefeed".into(), "Lena".into()));
    assert_eq!(card.emails()[0].value, "lena@example.org");
    assert_eq!(card.phones()[0].value, "tel:+33-1-23-45-67-89");
}

#[test]
fn fold_inside_a_utf8_character_unfolds_to_valid_text() {
    let card = Card::parse(&fixture("utf8-fold-midchar.vcf")).unwrap();
    assert!(card.display_name().ends_with("äöü äöü"));
    assert_eq!(
        card.structured_name(),
        ("Müller-Lüdenscheidt".into(), "Jürgen".into())
    );
}

#[test]
fn escaped_name_components_are_split_and_unescaped() {
    let card = Card::parse(
        b"BEGIN:VCARD\r\nN:Doe\\; Jr;Jane\\, Q;;;\r\nFN:Jane\\, Q Doe\r\nEND:VCARD\r\n",
    )
    .unwrap();
    assert_eq!(card.structured_name(), ("Doe; Jr".into(), "Jane, Q".into()));
    assert_eq!(card.display_name(), "Jane, Q Doe");
}

#[test]
fn property_names_match_case_insensitively() {
    let card = Card::parse(b"BEGIN:VCARD\r\nitem1.tel:123\r\nfn:x\r\nEND:VCARD\r\n").unwrap();
    assert_eq!(card.phones()[0].value, "123");
    assert_eq!(card.display_name(), "x");
}

#[derive(Debug, Clone)]
struct GeneratedLine {
    group: Option<String>,
    name: String,
    params: Vec<String>,
    value: String,
    folds: Vec<usize>,
    continuation: char,
    eol: &'static str,
}

impl GeneratedLine {
    fn logical(&self) -> String {
        let mut line = String::new();
        if let Some(group) = &self.group {
            line.push_str(group);
            line.push('.');
        }
        line.push_str(&self.name);
        for param in &self.params {
            line.push(';');
            line.push_str(param);
        }
        line.push(':');
        line.push_str(&self.value);
        line
    }

    fn physical(&self) -> Vec<u8> {
        let logical = self.logical().into_bytes();
        let mut points: Vec<usize> = self
            .folds
            .iter()
            .map(|f| 1 + f % (logical.len() - 1))
            .collect();
        points.sort_unstable();
        points.dedup();
        let mut out = Vec::new();
        let mut start = 0;
        for point in points {
            out.extend_from_slice(&logical[start..point]);
            out.extend_from_slice(self.eol.as_bytes());
            out.push(self.continuation as u8);
            start = point;
        }
        out.extend_from_slice(&logical[start..]);
        out.extend_from_slice(self.eol.as_bytes());
        out
    }
}

fn generated_line() -> impl Strategy<Value = GeneratedLine> {
    (
        proptest::option::of("[a-z][a-z0-9]{0,5}"),
        "[A-Z][A-Z0-9-]{0,9}",
        proptest::collection::vec("[A-Z]{1,6}=(\"[^\"\r\n]*\"|[^\";:,\r\n]{0,8})", 0..3),
        "[^\r\n]{0,120}",
        proptest::collection::vec(any::<usize>(), 0..4),
        prop_oneof![Just(' '), Just('\t')],
        prop_oneof![Just("\r\n"), Just("\n")],
    )
        .prop_map(
            |(group, name, params, value, folds, continuation, eol)| GeneratedLine {
                group,
                name,
                params,
                value,
                folds,
                continuation,
                eol,
            },
        )
}

proptest! {
    #[test]
    fn writing_an_unedited_parse_is_the_identity(lines in proptest::collection::vec(generated_line(), 0..8)) {
        let mut bytes = b"BEGIN:VCARD\r\n".to_vec();
        for line in &lines {
            bytes.extend(line.physical());
        }
        bytes.extend_from_slice(b"END:VCARD\r\n");

        let card = Card::parse(&bytes).unwrap();
        prop_assert_eq!(card.to_bytes(), bytes.clone());

        let parsed = &card.lines()[1..card.lines().len() - 1];
        prop_assert_eq!(parsed.len(), lines.len());
        for (parsed, generated) in parsed.iter().zip(&lines) {
            prop_assert_eq!(parsed.group(), generated.group.as_deref());
            prop_assert_eq!(parsed.name(), generated.name.as_str());
            prop_assert_eq!(parsed.params(), generated.params.as_slice());
            prop_assert_eq!(parsed.value(), generated.value.as_str());
        }
    }
}

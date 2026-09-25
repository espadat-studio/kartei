use std::fs;
use std::path::Path;

use kartei::card::{Card, Field, Kind, Labeled};
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

const FIELDS: [(Field, &str); 9] = [
    (Field::Prefixes, "N"),
    (Field::Given, "N"),
    (Field::Additional, "N"),
    (Field::Family, "N"),
    (Field::Suffixes, "N"),
    (Field::DisplayName, "FN"),
    (Field::Company, "ORG"),
    (Field::Department, "ORG"),
    (Field::Note, "NOTE"),
];

fn good_fixtures() -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with("bad-"))
            .collect();
    names.sort();
    names
}

fn lines_other_than(card: &Card, property: &str) -> Vec<Vec<u8>> {
    card.lines()
        .iter()
        .filter(|l| !l.name().eq_ignore_ascii_case(property))
        .map(|l| l.raw().to_vec())
        .collect()
}

proptest! {
    #[test]
    fn a_single_value_edit_changes_only_that_fields_lines(
        name in proptest::sample::select(good_fixtures()),
        field in 0..FIELDS.len(),
        value in "[^\r]{0,100}",
    ) {
        let before = Card::parse(&fixture(&name)).unwrap();
        let (field, property) = FIELDS[field];
        let mut after = before.clone();
        after.set(field, &value);

        prop_assert_eq!(lines_other_than(&after, property), lines_other_than(&before, property));
        let reparsed = Card::parse(&after.to_bytes()).unwrap();
        for (other, _) in FIELDS {
            let expected = if other == field { value.clone() } else { before.get(other) };
            prop_assert_eq!(reparsed.get(other), expected, "{:?}", other);
        }
    }

    #[test]
    fn edited_lines_fold_within_75_octets_on_character_boundaries(
        field in 0..FIELDS.len(),
        value in "[^\r]{0,300}",
    ) {
        let (field, property) = FIELDS[field];
        let mut card = Card::parse(
            b"BEGIN:VCARD\r\nitem1.FN;X-A=b:x\r\nN;CHARSET=utf-8:a;b;;;\r\nEND:VCARD\r\n",
        )
        .unwrap();
        card.set(field, &value);
        let edited = card.lines().iter().filter(|l| l.name() == property);
        for physical in edited.flat_map(|l| l.raw().split_inclusive(|&b| b == b'\n')) {
            let content = physical.strip_suffix(b"\r\n").unwrap();
            prop_assert!(content.len() <= 75, "{} octets", content.len());
            prop_assert!(std::str::from_utf8(content).is_ok());
        }
    }
}

const KINDS: [(Kind, &str, usize); 2] = [(Kind::Phone, "TEL", 1), (Kind::Email, "EMAIL", 1)];

fn outside_value(card: &Card, property: &str, n: usize) -> Vec<Vec<u8>> {
    let target = card
        .lines()
        .iter()
        .filter(|l| l.name().eq_ignore_ascii_case(property))
        .nth(n)
        .unwrap();
    let group = target.group().map(str::to_lowercase);
    card.lines()
        .iter()
        .filter(|l| !std::ptr::eq(*l, target))
        .filter(|l| group.is_none() || l.group().map(str::to_lowercase) != group)
        .map(|l| l.raw().to_vec())
        .collect()
}

fn raw_lines(card: &Card) -> Vec<Vec<u8>> {
    card.lines().iter().map(|l| l.raw().to_vec()).collect()
}

proptest! {
    #[test]
    fn a_multi_value_edit_changes_only_that_values_lines(
        name in proptest::sample::select(good_fixtures()),
        kind in 0..KINDS.len(),
        op in 0..3,
        pick in any::<usize>(),
        values in proptest::collection::vec("[^\r]{1,40}", 5),
        label in proptest::option::of(proptest::sample::select(vec!["home", "work", "cell", "other"])),
    ) {
        let before = Card::parse(&fixture(&name)).unwrap();
        let (kind, property, len) = KINDS[kind];
        let value = values[..len].to_vec();
        let mut expected = before.entries(kind);
        let mut after = before.clone();
        let n = pick % expected.len().max(1);
        match op {
            1 if !expected.is_empty() => {
                after.remove(kind, n);
                prop_assert_eq!(raw_lines(&after), outside_value(&before, property, n));
                expected.remove(n);
            }
            2 if !expected.is_empty() => {
                let label = label.map(str::to_owned).or(expected[n].label.clone());
                after.update(kind, n, &value, label.as_deref());
                prop_assert_eq!(outside_value(&after, property, n), outside_value(&before, property, n));
                expected[n] = Labeled { label, value };
            }
            _ => {
                after.add(kind, &value, label);
                let mut lines = raw_lines(&after);
                lines.remove(lines.len() - 2);
                prop_assert_eq!(lines, raw_lines(&before));
                expected.push(Labeled { label: label.map(str::to_owned), value });
            }
        }
        let reparsed = Card::parse(&after.to_bytes()).unwrap();
        prop_assert_eq!(reparsed.entries(kind), expected);
    }
}

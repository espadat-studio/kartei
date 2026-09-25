use std::fs;
use std::path::Path;

use kartei::card::{self, Address, Birthday, Card, Defect, Field, Kind, Labeled, Organization};

fn fixture(name: &str) -> Card {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    Card::parse(&fs::read(path).unwrap()).unwrap()
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
    let card = Card::parse(b"BEGIN:VCARD\r\nTEL:1\r\nitem1.EMAIL:a@b\r\nEND:VCARD\r\n").unwrap();
    assert_eq!(labeled(&card.phones()), [(None, "1")]);
    assert_eq!(labeled(&card.emails()), [(None, "a@b")]);
}

fn birthday(bday: &str) -> Option<Birthday> {
    Card::parse(format!("BEGIN:VCARD\r\n{bday}\r\nEND:VCARD\r\n").as_bytes())
        .unwrap()
        .birthday()
}

fn date(year: Option<u16>, month: u8, day: u8) -> Option<Birthday> {
    Some(Birthday::Date { year, month, day })
}

#[test]
fn full_date_birthday_keeps_its_year() {
    let bday = fixture("fn-custom.vcf").birthday().unwrap();
    assert_eq!(Some(bday.clone()), date(Some(1985), 7, 4));
    assert_eq!(bday.to_string(), "4 July 1985");
    assert!(!bday.is_read_only());
}

#[test]
fn apple_omit_year_birthday_has_no_year() {
    let bday = fixture("apple-omit-year.vcf").birthday().unwrap();
    assert_eq!(Some(bday.clone()), date(None, 3, 15));
    assert_eq!(bday.to_string(), "15 March");
}

#[test]
fn v4_yearless_birthdays_have_no_year() {
    assert_eq!(fixture("v4-yearless.vcf").birthday(), date(None, 3, 15));
    assert_eq!(birthday("BDAY:--03-15"), date(None, 3, 15));
}

#[test]
fn other_date_forms_are_read() {
    assert_eq!(birthday("BDAY:19850704"), date(Some(1985), 7, 4));
    assert_eq!(
        birthday("BDAY:1985-07-04T00:00:00Z"),
        date(Some(1985), 7, 4)
    );
    assert_eq!(
        birthday("BDAY;VALUE=date:2000-02-29"),
        date(Some(2000), 2, 29)
    );
}

#[test]
fn invalid_birthday_is_raw_and_read_only() {
    for raw in [
        "sometime in spring",
        "1985-13-01",
        "2001-02-29",
        "--02-30",
        "+985-07-04",
        "--0",
        "",
    ] {
        let bday = birthday(&format!("BDAY:{raw}")).unwrap();
        assert_eq!(bday, Birthday::Invalid(raw.into()));
        assert!(bday.is_read_only());
        assert_eq!(bday.to_string(), raw);
    }
}

#[test]
fn card_without_birthday_has_none() {
    assert_eq!(fixture("apple-grouped-labels.vcf").birthday(), None);
}

#[test]
fn escaped_address_components_are_split_and_unescaped() {
    let addresses = fixture("escaped-adr.vcf").addresses();
    assert_eq!(
        addresses,
        [Labeled {
            label: Some("work".into()),
            value: Address {
                po_box: "PO Box 7".into(),
                extended: "Building B".into(),
                street: "Hauptstr. 5, Hinterhaus\nc/o Meier; 2. OG".into(),
                city: "Berlin".into(),
                region: "Berlin".into(),
                postal_code: "10115".into(),
                country: "Germany".into(),
            },
        }]
    );
}

#[test]
fn grouped_address_with_missing_components_reads_empty() {
    let addresses = fixture("apple-grouped-labels.vcf").addresses();
    assert_eq!(addresses[0].label.as_deref(), Some("home"));
    assert_eq!(addresses[0].value.street, "Calle Mayor 1");
    assert_eq!(addresses[0].value.postal_code, "28013");
    let short = Card::parse(b"BEGIN:VCARD\r\nADR:;;Main St\r\nEND:VCARD\r\n")
        .unwrap()
        .addresses();
    assert_eq!(short[0].value.street, "Main St");
    assert_eq!(short[0].value.country, "");
}

#[test]
fn company_card_reads_organization_note_and_urls() {
    let card = fixture("company.vcf");
    assert_eq!(card.display_name(), "ACME Plumbing");
    assert_eq!(card.structured_name(), (String::new(), String::new()));
    assert_eq!(
        card.organization(),
        Some(Organization {
            company: "ACME Plumbing".into(),
            department: "Emergency Repairs".into(),
        })
    );
    assert_eq!(
        card.note().as_deref(),
        Some("Open 24/7.\nAsk for Bob, not Rob.")
    );
    assert_eq!(
        card.urls(),
        ["https://acme.example", "https://acme.example/emergency"]
    );
    assert_eq!(labeled(&card.phones()), [(Some("work"), "+1 555 0100")]);
}

#[test]
fn card_without_organization_or_note_has_none() {
    let card = fixture("fn-custom.vcf");
    assert_eq!(card.organization(), None);
    assert_eq!(card.note(), None);
    assert!(card.urls().is_empty());
}

fn defect(name: &str) -> Defect {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    Card::parse(&fs::read(path).unwrap()).unwrap_err()
}

#[test]
fn files_that_are_not_a_single_card_are_rejected() {
    assert_eq!(defect("bad-no-begin.vcf"), Defect::NoBegin);
    assert_eq!(defect("bad-no-end.vcf"), Defect::NoEnd);
    assert_eq!(defect("bad-invalid-utf8.vcf"), Defect::InvalidUtf8);
    assert_eq!(Card::parse(b""), Err(Defect::NoBegin));
}

#[test]
fn blank_lines_around_a_card_are_accepted() {
    let card = Card::parse(b"\r\nBEGIN:VCARD\r\nFN:x\r\nEND:VCARD\r\n\r\n").unwrap();
    assert_eq!(card.display_name(), "x");
}

fn edited(bytes: &str, field: Field, value: &str) -> String {
    let mut card = Card::parse(bytes.as_bytes()).unwrap();
    card.set(field, value);
    String::from_utf8(card.to_bytes()).unwrap()
}

#[test]
fn setting_a_name_part_rewrites_only_that_component() {
    assert_eq!(
        edited(
            "BEGIN:VCARD\r\nN:Doe\\, Sr;Jane;Q;;\r\nFN:Jane Doe\r\nEND:VCARD\r\n",
            Field::Given,
            "Ann; Marie",
        ),
        "BEGIN:VCARD\r\nN:Doe\\, Sr;Ann\\; Marie;Q;;\r\nFN:Jane Doe\r\nEND:VCARD\r\n"
    );
}

#[test]
fn setting_a_missing_property_appends_it_before_end_with_the_cards_line_ending() {
    assert_eq!(
        edited("BEGIN:VCARD\nFN:x\nEND:VCARD", Field::Note, "a,b\nc\\"),
        "BEGIN:VCARD\nFN:x\nNOTE:a\\,b\\nc\\\\\nEND:VCARD"
    );
    assert_eq!(
        edited("BEGIN:VCARD\r\nFN:x\r\nEND:VCARD\r\n", Field::Given, "Ann"),
        "BEGIN:VCARD\r\nFN:x\r\nN:;Ann;;;\r\nEND:VCARD\r\n"
    );
}

#[test]
fn clearing_note_or_organization_removes_the_line() {
    let card = "BEGIN:VCARD\r\nFN:x\r\nORG:ACME;\r\nNOTE:hi\r\nEND:VCARD\r\n";
    assert_eq!(
        edited(card, Field::Note, ""),
        "BEGIN:VCARD\r\nFN:x\r\nORG:ACME;\r\nEND:VCARD\r\n"
    );
    assert_eq!(
        edited(card, Field::Company, ""),
        "BEGIN:VCARD\r\nFN:x\r\nNOTE:hi\r\nEND:VCARD\r\n"
    );
}

#[test]
fn setting_an_unchanged_value_keeps_the_original_bytes() {
    let card = "BEGIN:VCARD\r\nitem1.fn;CHARSET=utf-8:Jane\r\nEND:VCARD\r\n";
    assert_eq!(edited(card, Field::DisplayName, "Jane"), card);
    assert_eq!(
        edited(card, Field::DisplayName, "Jo"),
        "BEGIN:VCARD\r\nitem1.fn;CHARSET=utf-8:Jo\r\nEND:VCARD\r\n"
    );
}

#[test]
fn long_values_fold_at_75_octets_without_splitting_characters() {
    let value = format!("{}ü{}", "a".repeat(73), "b".repeat(80));
    assert_eq!(
        edited("BEGIN:VCARD\r\nEND:VCARD\r\n", Field::Note, &value),
        format!(
            "BEGIN:VCARD\r\nNOTE:{}\r\n {}ü{}\r\n {}\r\nEND:VCARD\r\n",
            "a".repeat(70),
            "a".repeat(3),
            "b".repeat(69),
            "b".repeat(11)
        )
    );
}

#[test]
fn display_name_derives_from_name_parts_in_reading_order() {
    let parts = |field| match field {
        Field::Prefixes => "Dr.",
        Field::Given => "Jane",
        Field::Additional => "",
        Field::Family => "Doe",
        _ => "PhD",
    };
    assert_eq!(
        card::derived_display_name(|field| parts(field).to_owned()),
        "Dr. Jane Doe PhD"
    );
}

#[test]
fn relabeling_a_value_replaces_its_label_types_and_keeps_the_others() {
    let mut card =
        Card::parse(b"BEGIN:VCARD\r\nTEL;type=CELL;type=VOICE:1\r\nEND:VCARD\r\n").unwrap();
    card.update(Kind::Phone, 0, &["1".into()], Some("other"));
    assert_eq!(
        card.to_bytes(),
        b"BEGIN:VCARD\r\nTEL;type=VOICE;TYPE=OTHER:1\r\nEND:VCARD\r\n"
    );
}

#[test]
fn relabeling_a_grouped_value_drops_its_apple_label_and_keeps_the_group() {
    let mut card = Card::parse(
        b"BEGIN:VCARD\r\nitem1.TEL;type=pref:1\r\nitem1.X-ABLabel:Gym\r\nEND:VCARD\r\n",
    )
    .unwrap();
    card.update(Kind::Phone, 0, &["1".into()], Some("work"));
    assert_eq!(
        card.to_bytes(),
        b"BEGIN:VCARD\r\nitem1.TEL;type=pref;TYPE=WORK:1\r\nEND:VCARD\r\n"
    );
    assert_eq!(card.phones()[0].label.as_deref(), Some("work"));
}

#[test]
fn labels_cycle_through_home_work_cell_other() {
    assert_eq!(card::next_label(None), "home");
    assert_eq!(card::next_label(Some("Gym")), "home");
    assert_eq!(card::next_label(Some("home")), "work");
    assert_eq!(card::next_label(Some("cell")), "other");
    assert_eq!(card::next_label(Some("other")), "home");
}

#[test]
fn birthdays_are_written_in_the_cards_existing_form() {
    let date = |year, month, day| Birthday::Date { year, month, day };
    for (bday, birthday, expected) in [
        ("", date(Some(1985), 7, 4), "BDAY:1985-07-04"),
        ("VERSION:4.0", date(Some(1985), 7, 4), "BDAY:19850704"),
        ("VERSION:4.0", date(None, 3, 15), "BDAY:--0315"),
        (
            "",
            date(None, 3, 15),
            "BDAY;X-APPLE-OMIT-YEAR=1604:1604-03-15",
        ),
        ("BDAY:19850704", date(Some(1990), 1, 2), "BDAY:19900102"),
        (
            "BDAY:1985-07-04T00:00:00Z",
            date(Some(1985), 7, 5),
            "BDAY:1985-07-05T00:00:00Z",
        ),
        ("BDAY:--03-15", date(None, 4, 1), "BDAY:--04-01"),
        (
            "VERSION:4.0\r\nBDAY:1990-05-01",
            date(None, 5, 1),
            "BDAY:--0501",
        ),
        (
            "BDAY:1985-07-04",
            date(None, 7, 4),
            "BDAY;X-APPLE-OMIT-YEAR=1604:1604-07-04",
        ),
        (
            "BDAY;x-apple-omit-year=1900:1900-03-15",
            date(None, 4, 1),
            "BDAY;x-apple-omit-year=1900:1900-04-01",
        ),
    ] {
        let mut card =
            Card::parse(format!("BEGIN:VCARD\r\n{bday}\r\nEND:VCARD\r\n").as_bytes()).unwrap();
        card.set_birthday(Some(&birthday));
        let bytes = String::from_utf8(card.to_bytes()).unwrap();
        assert!(
            bytes.contains(&format!("{expected}\r\n")),
            "{bday:?}: {bytes}"
        );
        assert_eq!(card.birthday(), Some(birthday), "{bday:?}");
    }
}

#[test]
fn clearing_a_birthday_removes_its_line() {
    let mut card = fixture("apple-omit-year.vcf");
    card.set_birthday(None);
    assert!(!card.lines().iter().any(|l| l.name() == "BDAY"));
}

#[test]
fn a_new_card_round_trips_and_parses_back_to_the_entered_fields() {
    let mut card = Card::new("c0ffee", "\r\n");
    card.set(Field::Given, "Ann");
    card.set(Field::Family, "Lee");
    card.set(Field::DisplayName, "Ann Lee");
    card.add(Kind::Phone, &["+1 555".into()], Some("cell"));
    card.set(Field::Note, "hi, there");
    let bytes = card.to_bytes();
    assert_eq!(
        String::from_utf8(bytes.clone()).unwrap(),
        "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:c0ffee\r\nN:Lee;Ann;;;\r\nFN:Ann Lee\r\nTEL;TYPE=CELL:+1 555\r\nNOTE:hi\\, there\r\nEND:VCARD\r\n"
    );
    let parsed = Card::parse(&bytes).unwrap();
    assert_eq!(parsed.to_bytes(), bytes);
    assert_eq!(parsed.structured_name(), ("Lee".into(), "Ann".into()));
    assert_eq!(labeled(&parsed.phones()), [(Some("cell"), "+1 555")]);
    assert_eq!(parsed.note().as_deref(), Some("hi, there"));
}

#[test]
fn display_name_falls_back_to_the_company_without_a_structured_name() {
    let parts = |field| match field {
        Field::Company => "ACME".to_owned(),
        _ => String::new(),
    };
    assert_eq!(card::derived_display_name(parts), "ACME");
}

use std::fs;
use std::path::{Path, PathBuf};

use kartei::app::{App, Mode};
use kartei::vdir::{AddressBook, Conflict};
use kartei::{ui, vdir};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn address_book(test: &str, cards: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kartei-{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for (file, body) in cards {
        fs::write(dir.join(file), body).unwrap();
    }
    dir
}

fn card(structured_name: &str, display_name: &str, tel: &str, email: &str) -> String {
    format!(
        "BEGIN:VCARD\r\nVERSION:3.0\r\nN:{structured_name}\r\nFN:{display_name}\r\nTEL:{tel}\r\nEMAIL:{email}\r\nEND:VCARD\r\n"
    )
}

fn screen(app: &App) -> Vec<String> {
    screen_of_width(app, 80)
}

fn screen_of_width(app: &App, width: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
    terminal.draw(|frame| ui::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let width = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

fn press(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::from(code));
}

fn row_of(screen: &[String], text: &str) -> usize {
    screen
        .iter()
        .position(|row| row.contains(text))
        .unwrap_or_else(|| panic!("{text:?} not on screen:\n{}", screen.join("\n")))
}

fn detail(screen: &[String]) -> String {
    screen
        .iter()
        .map(|row| row.chars().skip(32).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn list(screen: &[String]) -> String {
    screen
        .iter()
        .map(|row| row.chars().take(32).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn copy_fixtures(test: &str) -> PathBuf {
    let dir = address_book(test, &[]);
    for entry in fs::read_dir(fixtures_dir()).unwrap() {
        let path = entry.unwrap().path();
        fs::copy(&path, dir.join(path.file_name().unwrap())).unwrap();
    }
    dir
}

fn open_fixtures() -> App {
    App::new(vdir::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")).unwrap())
}

fn select(app: &mut App, name: &str) {
    press(app, KeyCode::Char('g'));
    while app.selected_card().unwrap().display_name() != name {
        let before = app.selected();
        press(app, KeyCode::Char('j'));
        assert_ne!(before, app.selected(), "{name:?} not in list");
    }
}

fn assert_shows(text: &str, expected: &[&str]) {
    for want in expected {
        assert!(text.contains(want), "{want:?} missing from:\n{text}");
    }
}

fn open(test: &str) -> App {
    let dir = address_book(
        test,
        &[
            (
                "zed.vcf",
                &card(
                    "Adams;Zed;;;",
                    "Zed Adams",
                    "+1 555 0003",
                    "zed@example.org",
                ),
            ),
            (
                "bob.vcf",
                &card(
                    "Brown;Bob;;;",
                    "Bob Brown",
                    "+1 555 0004",
                    "bob@example.org",
                ),
            ),
            (
                "acme.vcf",
                &card(";;;;", "ACME Plumbing", "+1 555 0001", "info@acme.example"),
            ),
            (
                "anna.vcf",
                &card(
                    "Adams;Anna;;;",
                    "Anna Adams",
                    "+1 555 0002",
                    "anna@example.org",
                ),
            ),
            ("notes.txt", "not a card"),
        ],
    );
    App::new(vdir::load(&dir).unwrap())
}

#[test]
fn lists_cards_sorted_by_structured_name_then_display_name() {
    let screen = screen(&open("sorted"));
    let rows: Vec<usize> = ["ACME Plumbing", "Anna Adams", "Zed Adams", "Bob Brown"]
        .iter()
        .map(|name| row_of(&screen, name))
        .collect();
    assert!(rows.is_sorted(), "{}", screen.join("\n"));
    assert!(!screen.iter().any(|row| row.contains("not a card")));
}

#[test]
fn shows_details_of_the_selected_card() {
    let mut app = open("details");
    let first = detail(&screen(&app));
    assert!(
        first.contains("ACME Plumbing")
            && first.contains("+1 555 0001")
            && first.contains("info@acme.example")
    );

    press(&mut app, KeyCode::Char('j'));
    let second = detail(&screen(&app));
    assert!(
        second.contains("Anna Adams")
            && second.contains("+1 555 0002")
            && second.contains("anna@example.org")
    );
    assert!(!second.contains("ACME"));
}

#[test]
fn keys_move_the_selection() {
    let mut app = open("keys");
    let selected = |app: &App| app.selected_card().unwrap().display_name();

    press(&mut app, KeyCode::Char('k'));
    assert_eq!(selected(&app), "ACME Plumbing");
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(selected(&app), "Zed Adams");
    press(&mut app, KeyCode::Up);
    assert_eq!(selected(&app), "Anna Adams");
    press(&mut app, KeyCode::Char('G'));
    assert_eq!(selected(&app), "Bob Brown");
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(selected(&app), "Bob Brown");
    press(&mut app, KeyCode::Char('g'));
    assert_eq!(selected(&app), "ACME Plumbing");
}

#[test]
fn status_bar_shows_key_hints_and_q_quits() {
    let mut app = open("status");
    let screen = screen(&app);
    assert!(screen.last().unwrap().contains("q quit"));

    assert!(!app.should_quit());
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit());
}

#[test]
fn empty_address_book_renders_without_selection() {
    let mut app = App::new(AddressBook::default());
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('G'));
    assert!(app.selected_card().is_none());
    screen(&app);
}

#[test]
fn details_show_decoded_labels() {
    let mut app = open_fixtures();
    select(&mut app, "Gabriela Garcia");
    let detail = detail(&screen(&app));
    assert_shows(
        &detail,
        &[
            "Phone (Mobile)  +34 600 111 222",
            "Phone (Gym)  +34 600 333 444",
            "Phone (cell)  +34 600 555 666",
            "Email (home)  gabi@example.org",
            "Email (Work)  gabi@work.example",
            "Address (home)",
            "Calle Mayor 1",
            "28013 Madrid",
            "Spain",
        ],
    );
    assert!(!detail.contains("_$!<"), "{detail}");
}

#[test]
fn yearless_birthdays_show_without_1604() {
    let mut app = open_fixtures();
    for name in ["Omar Ortega", "Vera Vogel"] {
        select(&mut app, name);
        let detail = detail(&screen(&app));
        assert_shows(&detail, &["Birthday  15 March"]);
        assert!(!detail.contains("1604"), "{detail}");
    }
    select(&mut app, "Johnny D.");
    assert_shows(&detail(&screen(&app)), &["Birthday  4 July 1985"]);
}

#[test]
fn details_show_organization_note_urls_and_address() {
    let mut app = open_fixtures();
    select(&mut app, "ACME Plumbing");
    assert_shows(
        &detail(&screen(&app)),
        &[
            "Company  ACME Plumbing",
            "Department  Emergency Repairs",
            "URL  https://acme.example",
            "URL  https://acme.example/emergency",
            "Note",
            "Open 24/7.",
            "Ask for Bob, not Rob.",
        ],
    );

    select(&mut app, "Max Meier");
    let detail = detail(&screen(&app));
    assert_shows(
        &detail,
        &[
            "Address (work)",
            "Hauptstr. 5, Hinterhaus",
            "c/o Meier; 2. OG",
            "10115 Berlin",
            "Germany",
        ],
    );
    assert!(
        !detail.contains("PO Box") && !detail.contains("Building B"),
        "{detail}"
    );
}

#[test]
fn company_and_custom_display_name_cards_are_listed_under_display_name() {
    let list = list(&screen(&open_fixtures()));
    assert_shows(&list, &["ACME Plumbing", "Johnny D."]);
}

#[test]
fn bad_cards_are_skipped_and_counted_in_the_status_bar() {
    let dir = copy_fixtures("skipped-count");
    let app = App::new(vdir::load(&dir).unwrap());
    let screen = screen(&app);
    assert_eq!(app.cards().len(), 12);
    assert!(
        screen.last().unwrap().contains("3 cards skipped"),
        "{}",
        screen.join("\n")
    );
    for name in ["No Begin", "No End", "Bad"] {
        assert!(!list(&screen).contains(name), "{name}");
    }
}

#[test]
fn status_bar_has_no_skipped_count_when_all_cards_load() {
    assert!(
        !screen(&open("none-skipped"))
            .last()
            .unwrap()
            .contains("skipped")
    );
}

#[test]
fn bang_lists_every_skipped_path_with_its_reason() {
    let dir = copy_fixtures("skipped-list");
    let mut app = App::new(vdir::load(&dir).unwrap());
    assert!(screen(&app).last().unwrap().contains("! list"));

    press(&mut app, KeyCode::Char('!'));
    let screen = screen_of_width(&app, 240);
    for (file, reason) in [
        ("bad-no-begin.vcf", "no BEGIN:VCARD"),
        ("bad-no-end.vcf", "no END:VCARD"),
        ("bad-invalid-utf8.vcf", "invalid UTF-8"),
    ] {
        let row = row_of(&screen, &dir.join(file).display().to_string());
        let position = format!("{} #", dir.join(file).display());
        assert!(!screen[row].contains(&position), "{}", screen.join("\n"));
        assert!(screen[row + 1].contains(reason), "{}", screen.join("\n"));
    }

    press(&mut app, KeyCode::Esc);
    assert!(
        !screen_of_width(&app, 240)
            .join("\n")
            .contains("bad-no-begin.vcf")
    );
    assert!(!app.should_quit());
}

#[test]
fn bang_does_nothing_when_no_card_was_skipped() {
    let mut app = open("nothing-to-list");
    press(&mut app, KeyCode::Char('!'));
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit());
}

fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>, std::time::SystemTime)> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let modified = fs::metadata(&path).unwrap().modified().unwrap();
            (path.clone(), fs::read(&path).unwrap(), modified)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn skipped_files_are_never_written() {
    let dir = copy_fixtures("skipped-untouched");
    let before = snapshot(&dir);
    let mut app = App::new(vdir::load(&dir).unwrap());
    for code in [
        KeyCode::Char('!'),
        KeyCode::Esc,
        KeyCode::Char('G'),
        KeyCode::Char('!'),
        KeyCode::Enter,
        KeyCode::Char('q'),
    ] {
        press(&mut app, code);
        screen(&app);
    }
    assert_eq!(snapshot(&dir), before);
}

fn search_book(test: &str) -> App {
    let dir = address_book(
        test,
        &[
            (
                "anna.vcf",
                "BEGIN:VCARD\r\nN:Adams;Anna;;;\r\nFN:Anna Adams\r\nORG:Globex;Research\r\nEMAIL:anna@example.org\r\nTEL:+1 (555) 010-2030\r\nEND:VCARD\r\n",
            ),
            (
                "bob.vcf",
                "BEGIN:VCARD\r\nN:Brown;Bob;;;\r\nFN:Bob Brown\r\nORG:Initech\r\nEMAIL:bob@mail.test\r\nTEL:+49 170 1234567\r\nEND:VCARD\r\n",
            ),
            (
                "cara.vcf",
                "BEGIN:VCARD\r\nN:Chen;Cara;;;\r\nFN:Cara Chen\r\nEMAIL:cara@example.org\r\nTEL;VALUE=uri:tel:+33-1-23-45-67-89\r\nEND:VCARD\r\n",
            ),
        ],
    );
    App::new(vdir::load(&dir).unwrap())
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        press(app, KeyCode::Char(c));
    }
}

fn listed(app: &App) -> Vec<String> {
    app.cards().iter().map(|card| card.display_name()).collect()
}

#[test]
fn slash_filters_the_list_live_by_display_name() {
    let mut app = search_book("search-name");
    press(&mut app, KeyCode::Char('/'));
    assert_eq!(app.mode(), Mode::Search);
    type_text(&mut app, "n");
    assert_eq!(listed(&app), ["Anna Adams", "Bob Brown", "Cara Chen"]);
    type_text(&mut app, "A");
    assert_eq!(listed(&app), ["Anna Adams"]);
    let screen = screen(&app);
    assert!(
        screen.last().unwrap().contains("/nA"),
        "{}",
        screen.join("\n")
    );
    assert!(!list(&screen).contains("Cara Chen"));
    assert!(detail(&screen).contains("Anna Adams"));
}

#[test]
fn keys_typed_while_searching_are_part_of_the_query() {
    let mut app = search_book("search-literal");
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "qj?!/g");
    assert!(!app.should_quit());
    assert_eq!(app.mode(), Mode::Search);
    assert!(listed(&app).is_empty());
    assert!(screen(&app).last().unwrap().contains("/qj?!/g"));
}

#[test]
fn enter_keeps_the_filter_and_esc_clears_it() {
    let mut app = search_book("search-keep-clear");
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "chen");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(listed(&app), ["Cara Chen"]);
    assert!(list(&screen(&app)).contains("Cards /chen"));
    press(&mut app, KeyCode::Char('j'));
    assert_eq!(app.selected_card().unwrap().display_name(), "Cara Chen");

    press(&mut app, KeyCode::Char('/'));
    press(&mut app, KeyCode::Backspace);
    assert_eq!(app.query(), "che");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(app.query(), "");
    assert_eq!(listed(&app), ["Anna Adams", "Bob Brown", "Cara Chen"]);
    assert_eq!(app.selected_card().unwrap().display_name(), "Cara Chen");
    assert!(!list(&screen(&app)).contains("Cards /"));
}

#[test]
fn search_with_no_match_has_no_selection() {
    let mut app = search_book("search-empty");
    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "zzz");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Char('G'));
    assert!(app.selected_card().is_none());
    screen(&app);
}

#[test]
fn search_matches_company_email_and_phone_digits() {
    let mut app = search_book("search-fields");
    for (query, expected) in [
        ("INITECH", &["Bob Brown"][..]),
        ("research", &["Anna Adams"]),
        ("mail.test", &["Bob Brown"]),
        ("example.org", &["Anna Adams", "Cara Chen"]),
        ("5550102030", &["Anna Adams"]),
        ("(555) 010-2030", &["Anna Adams"]),
        ("+49 170", &["Bob Brown"]),
        ("3312345", &["Cara Chen"]),
        ("tel", &[]),
        ("zzz", &[]),
    ] {
        press(&mut app, KeyCode::Char('/'));
        type_text(&mut app, query);
        assert_eq!(listed(&app), expected, "{query:?}");
        press(&mut app, KeyCode::Esc);
    }
}

#[test]
fn question_mark_shows_the_keymap_and_any_key_dismisses_it() {
    let mut app = search_book("help");
    assert!(screen(&app).last().unwrap().contains("? help"));
    press(&mut app, KeyCode::Char('?'));
    assert_eq!(app.mode(), Mode::Help);
    let screen = screen(&app);
    for mode in ["Browse", "Search", "Prompt", "Help"] {
        row_of(&screen, mode);
    }
    for (key, action) in [
        ("j/k", "move"),
        ("g/G", "top/bottom"),
        ("/", "search"),
        ("!", "list skipped cards"),
        ("?", "show keys"),
        ("q", "quit"),
        ("Backspace", "delete"),
        ("Enter", "keep filter"),
        ("Esc", "clear filter"),
        ("any key", "close"),
    ] {
        let row = &screen[row_of(&screen, action)];
        assert!(row.contains(key), "{key:?} not beside {action:?}: {row}");
    }

    press(&mut app, KeyCode::Char('q'));
    assert_eq!(app.mode(), Mode::Browse);
    assert!(!app.should_quit());
    assert!(
        !screen_of_width(&app, 80)
            .join("\n")
            .contains("clear filter")
    );
}

fn save(app: &mut App) {
    app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
}

fn replace_given_name(app: &mut App, old: &str, new: &str) {
    press(app, KeyCode::Tab);
    for _ in old.chars() {
        press(app, KeyCode::Backspace);
    }
    type_text(app, new);
}

const ANNA: &str = "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Adams;Anna;;;\r\nFN:Anna Adams\r\nTEL:+1 555 0002\r\nEND:VCARD\r\n";

#[test]
fn saving_an_edited_given_name_rewrites_only_n_and_a_linked_display_name() {
    let dir = address_book("edit-linked", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    assert_eq!(app.mode(), Mode::Edit);
    assert_shows(&detail(&screen(&app)), &["Given name", "Anna", "(linked)"]);

    replace_given_name(&mut app, "Anna", "Hanna");
    assert_shows(&detail(&screen(&app)), &["Hanna Adams", "(linked)"]);
    save(&mut app);

    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(
        fs::read_to_string(dir.join("anna.vcf")).unwrap(),
        "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Adams;Hanna;;;\r\nFN:Hanna Adams\r\nTEL:+1 555 0002\r\nEND:VCARD\r\n"
    );
    assert_eq!(listed(&app), ["Hanna Adams"]);
    assert!(!fs::read_dir(&dir).unwrap().any(|e| {
        e.unwrap()
            .path()
            .extension()
            .is_some_and(|ext| ext == "tmp")
    }));
}

#[test]
fn editing_the_structured_name_leaves_a_custom_display_name_unchanged() {
    let original = fs::read_to_string(fixtures_dir().join("fn-custom.vcf")).unwrap();
    let dir = address_book("edit-custom", &[("johnny.vcf", &original)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Enter);
    assert_shows(&detail(&screen(&app)), &["Johnny D.", "(custom)"]);

    replace_given_name(&mut app, "Jonathan", "John");
    save(&mut app);

    assert_eq!(
        fs::read_to_string(dir.join("johnny.vcf")).unwrap(),
        original.replace("N:Doe;Jonathan;;;", "N:Doe;John;;;")
    );
    assert_eq!(listed(&app), ["Johnny D."]);
}

#[test]
fn esc_with_unsaved_changes_asks_before_discarding_and_writes_nothing() {
    let dir = address_book("edit-discard", &[("anna.vcf", ANNA)]);
    let before = snapshot(&dir);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode(), Mode::Browse);

    press(&mut app, KeyCode::Char('e'));
    type_text(&mut app, "Dr.");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode(), Mode::Discard);
    assert!(
        screen(&app)
            .last()
            .unwrap()
            .contains("Discard unsaved changes? y/n")
    );
    press(&mut app, KeyCode::Char('n'));
    assert_eq!(app.mode(), Mode::Edit);
    assert!(detail(&screen(&app)).contains("Dr."));

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('y'));
    assert_eq!(app.mode(), Mode::Browse);
    assert!(!detail(&screen(&app)).contains("Dr."));
    assert_eq!(snapshot(&dir), before);
}

#[test]
fn multi_line_note_and_non_ascii_text_round_trip_through_the_form() {
    let dir = address_book("edit-note", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    press(&mut app, KeyCode::BackTab);
    type_text(&mut app, "Zeile 1");
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "Grüße, 日本");
    focus(&mut app, "Family name");
    for _ in "Adams".chars() {
        press(&mut app, KeyCode::Backspace);
    }
    type_text(&mut app, "Østergård");
    save(&mut app);

    let written = fs::read_to_string(dir.join("anna.vcf")).unwrap();
    assert!(
        written.contains("N:Østergård;Anna;;;\r\nFN:Anna Østergård\r\n"),
        "{written}"
    );
    assert!(
        written.ends_with("NOTE:Zeile 1\\nGrüße\\, 日本\r\nEND:VCARD\r\n"),
        "{written}"
    );

    let mut app = App::new(vdir::load(&dir).unwrap());
    let card = app.selected_card().unwrap();
    assert_eq!(card.note().as_deref(), Some("Zeile 1\nGrüße, 日本"));
    assert_eq!(card.display_name(), "Anna Østergård");
    press(&mut app, KeyCode::Char('e'));
    assert_shows(
        &detail(&screen(&app)),
        &["Østergård", "Anna Østergård", "Zeile 1", "Grüße, 日"],
    );
}

#[test]
fn failed_save_shows_the_error_and_keeps_the_form_open() {
    let dir = address_book("edit-fail", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    fs::create_dir(dir.join("anna.vcf.tmp")).unwrap();
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Anna", "Hanna");
    save(&mut app);

    assert_eq!(app.mode(), Mode::Edit);
    assert!(app.error().unwrap().starts_with("save failed"));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| ui::draw(frame, &app)).unwrap();
    let status = terminal.backend().buffer()[(1, 23)].clone();
    assert_eq!(
        (status.symbol(), status.fg),
        ("s", ratatui::style::Color::Red)
    );
    assert!(detail(&screen(&app)).contains("Hanna"));

    press(&mut app, KeyCode::Esc);
    assert!(
        screen(&app)
            .last()
            .unwrap()
            .contains("Discard unsaved changes? y/n")
    );
}

#[test]
fn cursor_sits_after_wide_characters_in_the_focused_input() {
    let dir = address_book("edit-cursor", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "日本");
    press(&mut app, KeyCode::Left);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| ui::draw(frame, &app)).unwrap();
    let detail_x = 32;
    let label = 13;
    let anna_ri_cells = 6;
    assert_eq!(
        terminal.get_cursor_position().unwrap(),
        ratatui::layout::Position::new(detail_x + 1 + label + anna_ri_cells, 2)
    );
}

#[test]
fn a_custom_display_name_stays_custom_when_the_structured_name_catches_up_with_it() {
    let dir = address_book(
        "edit-stays-custom",
        &[(
            "jon.vcf",
            "BEGIN:VCARD\r\nN:Doe;Jon;;;\r\nFN:Jon Do\r\nEND:VCARD\r\n",
        )],
    );
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    for _ in 0..3 {
        press(&mut app, KeyCode::Tab);
    }
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Tab);
    assert!(detail(&screen(&app)).contains("(custom)"));

    for _ in 0..4 {
        press(&mut app, KeyCode::Up);
    }
    type_text(&mut app, "a");
    let detail = detail(&screen(&app));
    assert_shows(&detail, &["Jon Do", "(custom)"]);
    assert!(!detail.contains("Jona Do"), "{detail}");
}

fn alt(app: &mut App, c: char) {
    app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

fn tab(app: &mut App, times: usize) {
    for _ in 0..times {
        press(app, KeyCode::Tab);
    }
}

const FIRST_VALUE: usize = 8;

fn focus(app: &mut App, label: &str) {
    for _ in 0..100 {
        let form = app.form().unwrap();
        if form.rows()[form.focus()].label == label {
            return;
        }
        press(app, KeyCode::Tab);
    }
    panic!("no {label:?} row in the form");
}

fn open_fixture(test: &str, name: &str) -> (PathBuf, String, App) {
    let original = fs::read_to_string(fixtures_dir().join(name)).unwrap();
    let dir = address_book(test, &[(name, &original)]);
    let app = App::new(vdir::load(&dir).unwrap());
    (dir.join(name), original, app)
}

#[test]
fn editing_a_grouped_phone_rewrites_only_its_tel_line() {
    let (path, original, mut app) = open_fixture("edit-grouped-tel", "apple-grouped-labels.vcf");
    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, FIRST_VALUE);
    for _ in 0..3 {
        press(&mut app, KeyCode::Backspace);
    }
    type_text(&mut app, "999");
    save(&mut app);

    assert_eq!(
        fs::read_to_string(path).unwrap(),
        original.replace(
            "item1.TEL;type=pref:+34 600 111 222",
            "item1.TEL;type=pref:+34 600 111 999"
        )
    );
}

#[test]
fn removing_a_grouped_email_removes_its_whole_group() {
    let (path, original, mut app) =
        open_fixture("remove-grouped-email", "apple-grouped-labels.vcf");
    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, FIRST_VALUE + 4);
    assert_shows(&detail(&screen(&app)), &["gabi@work.example  (Work)"]);
    alt(&mut app, 'd');
    save(&mut app);

    assert_eq!(
        fs::read_to_string(path).unwrap(),
        original.replace(
            "item3.EMAIL;type=INTERNET:gabi@work.example\r\nitem3.X-ABLabel:_$!<Work>!$_\r\n",
            ""
        )
    );
}

#[test]
fn adding_a_phone_with_a_cycled_label_writes_one_typed_tel_line() {
    let dir = address_book("add-work-tel", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, FIRST_VALUE);
    alt(&mut app, 'a');
    type_text(&mut app, "+1 555 0009");
    alt(&mut app, 'l');
    alt(&mut app, 'l');
    assert_shows(&detail(&screen(&app)), &["+1 555 0009  (work)"]);
    save(&mut app);

    assert_eq!(
        fs::read_to_string(dir.join("anna.vcf")).unwrap(),
        ANNA.replace("END:VCARD", "TEL;TYPE=WORK:+1 555 0009\r\nEND:VCARD")
    );
}

#[test]
fn a_card_without_emails_offers_a_blank_email_that_is_only_written_when_filled() {
    let dir = address_book("blank-email", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, FIRST_VALUE + 1);
    alt(&mut app, 'd');
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode(), Mode::Browse);

    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, FIRST_VALUE + 1);
    type_text(&mut app, "anna@example.org");
    save(&mut app);
    assert_eq!(
        fs::read_to_string(dir.join("anna.vcf")).unwrap(),
        ANNA.replace("END:VCARD", "EMAIL:anna@example.org\r\nEND:VCARD")
    );
}

#[test]
fn clearing_a_phone_removes_its_line() {
    let dir = address_book("clear-tel", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, FIRST_VALUE);
    for _ in "+1 555 0002".chars() {
        press(&mut app, KeyCode::Backspace);
    }
    save(&mut app);
    assert_eq!(
        fs::read_to_string(dir.join("anna.vcf")).unwrap(),
        ANNA.replace("TEL:+1 555 0002\r\n", "")
    );
}

const BIRTHDAY: usize = FIRST_VALUE + 7;

fn set_birthday(app: &mut App, old: &str, new: &str) {
    press(app, KeyCode::Char('e'));
    tab(app, BIRTHDAY);
    for _ in old.chars() {
        press(app, KeyCode::Backspace);
    }
    type_text(app, new);
    save(app);
}

#[test]
fn a_new_yearless_birthday_uses_the_apple_form_on_3_0_and_dashes_on_4_0() {
    let v4 = ANNA
        .replace("VERSION:3.0", "VERSION:4.0")
        .replace("Anna", "Bea");
    let dir = address_book("bday-new", &[("anna.vcf", ANNA), ("anna4.vcf", &v4)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    set_birthday(&mut app, "", "--03-15");
    press(&mut app, KeyCode::Char('j'));
    set_birthday(&mut app, "", "--03-15");

    let bday = |file: &str| {
        let card = fs::read_to_string(dir.join(file)).unwrap();
        card.lines()
            .find(|l| l.starts_with("BDAY"))
            .unwrap()
            .to_owned()
    };
    assert_eq!(bday("anna.vcf"), "BDAY;X-APPLE-OMIT-YEAR=1604:1604-03-15");
    assert_eq!(bday("anna4.vcf"), "BDAY:--0315");
}

#[test]
fn editing_a_birthday_keeps_the_cards_existing_form() {
    for (fixture, old, new, before, after) in [
        (
            "apple-omit-year.vcf",
            "--03-15",
            "--04-01",
            "BDAY;X-APPLE-OMIT-YEAR=1604;VALUE=date:1604-03-15",
            "BDAY;X-APPLE-OMIT-YEAR=1604;VALUE=date:1604-04-01",
        ),
        (
            "v4-yearless.vcf",
            "--03-15",
            "--04-01",
            "BDAY:--0315",
            "BDAY:--0401",
        ),
        (
            "fn-custom.vcf",
            "1985-07-04",
            "1990-12-31",
            "BDAY:1985-07-04",
            "BDAY:1990-12-31",
        ),
        (
            "apple-omit-year.vcf",
            "--03-15",
            "1970-03-15",
            "BDAY;X-APPLE-OMIT-YEAR=1604;VALUE=date:1604-03-15",
            "BDAY;VALUE=date:1970-03-15",
        ),
    ] {
        let (path, original, mut app) = open_fixture("bday-keep", fixture);
        set_birthday(&mut app, old, new);
        assert_eq!(app.mode(), Mode::Browse, "{fixture}: {:?}", app.error());
        assert_eq!(
            fs::read_to_string(path).unwrap(),
            original.replace(before, after),
            "{fixture}"
        );
    }
}

#[test]
fn clearing_a_birthday_removes_it_and_an_invalid_one_keeps_the_form_open() {
    let (path, original, mut app) = open_fixture("bday-clear", "fn-custom.vcf");
    set_birthday(&mut app, "1985-07-04", "July 4th");
    assert_eq!(app.mode(), Mode::Edit);
    assert!(
        app.error().unwrap().contains("birthday"),
        "{:?}",
        app.error()
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), original);

    for _ in "July 4th".chars() {
        press(&mut app, KeyCode::Backspace);
    }
    save(&mut app);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        original.replace("BDAY:1985-07-04\r\n", "")
    );
}

#[test]
fn an_unreadable_birthday_is_read_only_in_the_form() {
    let dir = address_book(
        "bday-read-only",
        &[("x.vcf", &ANNA.replace("END", "BDAY:spring\r\nEND"))],
    );
    let mut app = App::new(vdir::load(&dir).unwrap());
    set_birthday(&mut app, "spring", "--01-01");
    assert_eq!(app.mode(), Mode::Browse);
    assert!(
        fs::read_to_string(dir.join("x.vcf"))
            .unwrap()
            .contains("BDAY:spring\r\n")
    );
}

const STREET: usize = FIRST_VALUE + 2;

#[test]
fn editing_an_address_city_keeps_hidden_components_and_the_apple_country_code() {
    for (fixture, old, new, before, after) in [
        (
            "escaped-adr.vcf",
            "Berlin",
            "Potsdam",
            ";Berlin;Berlin;10115;",
            ";Potsdam;Berlin;10115;",
        ),
        (
            "apple-grouped-labels.vcf",
            "Madrid",
            "Sevilla",
            "Calle Mayor 1;Madrid;",
            "Calle Mayor 1;Sevilla;",
        ),
    ] {
        let (path, original, mut app) = open_fixture("adr-city", fixture);
        press(&mut app, KeyCode::Char('e'));
        focus(&mut app, "City");
        for _ in old.chars() {
            press(&mut app, KeyCode::Backspace);
        }
        type_text(&mut app, new);
        save(&mut app);
        let written = fs::read_to_string(path).unwrap();
        assert_eq!(
            written.replace("\r\n ", ""),
            original.replace(before, after),
            "{fixture}"
        );
        assert!(written.lines().all(|l| l.len() <= 76), "{written}");
    }
}

#[test]
fn a_two_line_street_round_trips_with_its_escaped_newline() {
    let dir = address_book("adr-street", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    tab(&mut app, STREET);
    type_text(&mut app, "Main St 1");
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "Apt 2");
    tab(&mut app, 2);
    type_text(&mut app, "Springfield");
    alt(&mut app, 'l');
    save(&mut app);

    assert_eq!(
        fs::read_to_string(dir.join("anna.vcf")).unwrap(),
        ANNA.replace(
            "END:VCARD",
            "ADR;TYPE=HOME:;;Main St 1\\nApt 2;Springfield;;;\r\nEND:VCARD"
        )
    );
    let mut app = App::new(vdir::load(&dir).unwrap());
    assert_eq!(
        app.selected_card().unwrap().addresses()[0].value.street,
        "Main St 1\nApt 2"
    );
    press(&mut app, KeyCode::Char('e'));
    assert_shows(
        &detail(&screen(&app)),
        &["Street", "Main St 1", "Apt 2", "(home)"],
    );
}

fn vcf_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    files
}

#[test]
fn n_creates_one_card_named_after_its_uid_and_selects_it() {
    let dir = address_book(
        "new-card",
        &[("zed.vcf", &card("Zed;Zoe;;;", "Zoe Zed", "1", "z@z"))],
    );
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('n'));
    assert_eq!(app.mode(), Mode::Edit);
    focus(&mut app, "Given name");
    type_text(&mut app, "Ann");
    focus(&mut app, "Family name");
    type_text(&mut app, "Lee");
    focus(&mut app, "Phone");
    type_text(&mut app, "+1 555 0100");
    save(&mut app);

    assert_eq!(app.mode(), Mode::Browse);
    let files = vcf_files(&dir);
    assert_eq!(files.len(), 2, "{files:?}");
    let created = files.iter().find(|p| !p.ends_with("zed.vcf")).unwrap();
    let text = fs::read_to_string(created).unwrap();
    let uid = text.lines().find_map(|l| l.strip_prefix("UID:")).unwrap();
    assert_eq!(
        created.file_name().unwrap().to_str().unwrap(),
        format!("{uid}.vcf")
    );
    assert_eq!(
        text,
        format!(
            "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:{uid}\r\nN:Lee;Ann;;;\r\nFN:Ann Lee\r\nTEL:+1 555 0100\r\nEND:VCARD\r\n"
        )
    );
    assert_eq!(listed(&app), ["Ann Lee", "Zoe Zed"]);
    assert_eq!(app.selected_card().unwrap().display_name(), "Ann Lee");
}

#[test]
fn a_company_only_card_gets_its_display_name_from_the_company() {
    let dir = address_book("new-company", &[]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('n'));
    focus(&mut app, "Company");
    type_text(&mut app, "ACME");
    assert_shows(&detail(&screen(&app)), &["ACME  (linked)"]);
    save(&mut app);

    let text = fs::read_to_string(&vcf_files(&dir)[0]).unwrap();
    assert!(
        text.ends_with("\r\nN:;;;;\r\nFN:ACME\r\nORG:ACME\r\nEND:VCARD\r\n"),
        "{text}"
    );
    assert_eq!(listed(&app), ["ACME"]);
}

#[test]
fn a_new_card_left_empty_or_cancelled_writes_nothing() {
    let dir = address_book("new-nothing", &[]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('n'));
    save(&mut app);
    assert_eq!(app.mode(), Mode::Browse);
    press(&mut app, KeyCode::Char('n'));
    type_text(&mut app, "Dr.");
    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char('y'));
    assert!(vcf_files(&dir).is_empty());
    assert!(app.cards().is_empty());
}

const EXTERNAL: &str =
    "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Adams;Anne;;;\r\nFN:Anne Adams\r\nEND:VCARD\r\n";

fn edit_then_change_on_disk(test: &str) -> (PathBuf, App) {
    let dir = address_book(test, &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Anna", "Hanna");
    fs::write(dir.join("anna.vcf"), EXTERNAL).unwrap();
    save(&mut app);
    (dir.join("anna.vcf"), app)
}

#[test]
fn saving_over_a_changed_file_prompts_and_keeps_the_external_change() {
    let (path, app) = edit_then_change_on_disk("conflict-prompt");
    assert_eq!(app.mode(), Mode::Conflict(Conflict::Changed));
    assert_shows(
        screen(&app).last().unwrap(),
        &[
            "changed on disk",
            "r reload",
            "o overwrite",
            "Esc keep editing",
        ],
    );
    assert_eq!(fs::read_to_string(path).unwrap(), EXTERNAL);
}

#[test]
fn r_reloads_the_external_version_and_drops_the_edit() {
    let (path, mut app) = edit_then_change_on_disk("conflict-reload");
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(listed(&app), ["Anne Adams"]);
    assert!(detail(&screen(&app)).contains("Anne Adams"));
    assert_eq!(fs::read_to_string(path).unwrap(), EXTERNAL);
}

#[test]
fn o_overwrites_the_external_change_with_the_edit() {
    let (path, mut app) = edit_then_change_on_disk("conflict-overwrite");
    press(&mut app, KeyCode::Char('o'));
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        ANNA.replace("Anna", "Hanna")
    );
    assert_eq!(listed(&app), ["Hanna Adams"]);
}

#[test]
fn esc_returns_to_the_form_with_the_edit_intact() {
    let (path, mut app) = edit_then_change_on_disk("conflict-esc");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode(), Mode::Edit);
    assert_shows(&detail(&screen(&app)), &["Hanna"]);
    assert_eq!(fs::read_to_string(path).unwrap(), EXTERNAL);
}

#[test]
fn saving_a_deleted_file_offers_recreate_which_writes_the_edit_back() {
    let dir = address_book("conflict-deleted", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Anna", "Hanna");
    fs::remove_file(dir.join("anna.vcf")).unwrap();
    save(&mut app);
    assert_eq!(app.mode(), Mode::Conflict(Conflict::Deleted));
    assert_shows(
        screen(&app).last().unwrap(),
        &["deleted on disk", "o recreate"],
    );
    assert!(!dir.join("anna.vcf").exists());

    press(&mut app, KeyCode::Char('o'));
    assert_eq!(
        fs::read_to_string(dir.join("anna.vcf")).unwrap(),
        ANNA.replace("Anna", "Hanna")
    );
}

#[test]
fn shift_r_reloads_cards_added_changed_or_removed_on_disk() {
    let mut app = open("reload");
    let dir = std::env::temp_dir().join(format!("kartei-reload-{}", std::process::id()));
    select(&mut app, "Bob Brown");
    fs::write(
        dir.join("new.vcf"),
        card("Cole;Cy;;;", "Cy Cole", "1", "c@c"),
    )
    .unwrap();
    fs::write(
        dir.join("bob.vcf"),
        card("Brown;Rob;;;", "Rob Brown", "1", "r@b"),
    )
    .unwrap();
    fs::remove_file(dir.join("zed.vcf")).unwrap();
    fs::write(dir.join("broken.vcf"), "nope").unwrap();

    press(&mut app, KeyCode::Char('R'));
    assert_eq!(
        listed(&app),
        ["ACME Plumbing", "Anna Adams", "Rob Brown", "Cy Cole"]
    );
    assert_eq!(app.selected_card().unwrap().display_name(), "Rob Brown");
    assert_eq!(app.skipped().len(), 1);
}

#[test]
fn y_then_a_digit_emits_osc_52_with_that_value_and_reports_copied() {
    let mut app = open("copy");
    press(&mut app, KeyCode::Char('y'));
    assert_eq!(app.mode(), Mode::Copy);
    let screen = screen(&app);
    assert_shows(
        &screen.join("\n"),
        &["1  Phone  +1 555 0001", "2  Email  info@acme.example"],
    );

    press(&mut app, KeyCode::Char('2'));
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(
        app.take_clipboard().as_deref(),
        Some("\x1b]52;c;aW5mb0BhY21lLmV4YW1wbGU=\x07")
    );
    assert_eq!(app.take_clipboard(), None);
    assert!(self::screen(&app).last().unwrap().contains("copied"));
}

#[test]
fn esc_closes_the_copy_prompt_without_emitting() {
    let mut app = open("copy-esc");
    press(&mut app, KeyCode::Char('y'));
    press(&mut app, KeyCode::Char('9'));
    assert_eq!(app.mode(), Mode::Copy);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(app.take_clipboard(), None);
    assert!(!screen(&app).last().unwrap().contains("copied"));
}

#[test]
fn saving_without_edits_closes_the_form_even_if_the_file_changed() {
    let dir = address_book("conflict-unedited", &[("anna.vcf", ANNA)]);
    let mut app = App::new(vdir::load(&dir).unwrap());
    press(&mut app, KeyCode::Char('e'));
    fs::write(dir.join("anna.vcf"), EXTERNAL).unwrap();
    save(&mut app);
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(fs::read_to_string(dir.join("anna.vcf")).unwrap(), EXTERNAL);
}

fn bundle() -> [String; 3] {
    [
        card("Cole;Cy;;;", "Cy Cole", "+1 555 0101", "cy@example.org"),
        card(
            "Brown;Bob;;;",
            "Bob Brown",
            "+1 555 0102",
            "bob@example.org",
        ),
        card("Diaz;Dee;;;", "Dee Diaz", "+1 555 0103", "dee@example.org"),
    ]
}

fn open_bundle(test: &str, cards: &[&str]) -> (PathBuf, App) {
    let dir = address_book(test, &[("bundle.vcf", &cards.concat()), ("anna.vcf", ANNA)]);
    (dir.join("bundle.vcf"), App::new(vdir::load(&dir).unwrap()))
}

fn edit_bob_then_change_dee_on_disk(test: &str) -> (PathBuf, App, String) {
    let [cy, bob, dee] = bundle();
    let (path, mut app) = open_bundle(test, &[&cy, &bob, &dee]);
    select(&mut app, "Bob Brown");
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Bob", "Rob");
    let external = [cy, bob, dee.replace("Dee", "Dora")].concat();
    fs::write(&path, &external).unwrap();
    save(&mut app);
    (path, app, external)
}

#[test]
fn cards_in_a_bundle_are_listed_sorted_and_searchable_next_to_single_card_files() {
    let [cy, bob, dee] = bundle();
    let (_, mut app) = open_bundle("bundle-list", &[&cy, &bob, &dee]);
    assert_eq!(
        listed(&app),
        ["Anna Adams", "Bob Brown", "Cy Cole", "Dee Diaz"]
    );
    assert!(app.skipped().is_empty());

    press(&mut app, KeyCode::Char('/'));
    type_text(&mut app, "0103");
    assert_eq!(listed(&app), ["Dee Diaz"]);
    assert_shows(&detail(&screen(&app)), &["Dee Diaz", "dee@example.org"]);
}

#[test]
fn saving_a_card_in_a_bundle_rewrites_only_its_lines() {
    let [cy, bob, dee] = bundle();
    let (path, mut app) = open_bundle("bundle-edit", &[&cy, &bob, &dee]);
    select(&mut app, "Bob Brown");
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Bob", "Rob");
    save(&mut app);
    assert_eq!(app.mode(), Mode::Browse);
    let rob = bob.replace("Bob", "Rob");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        [cy.as_str(), &rob, &dee].concat()
    );

    select(&mut app, "Dee Diaz");
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Dee", "Dora");
    save(&mut app);
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        [cy, rob, dee.replace("Dee", "Dora")].concat()
    );
}

#[test]
fn changing_another_card_of_the_bundle_on_disk_prompts_and_keeps_the_change() {
    let (path, app, external) = edit_bob_then_change_dee_on_disk("bundle-conflict");
    assert_eq!(app.mode(), Mode::Conflict(Conflict::Changed));
    assert_eq!(fs::read_to_string(path).unwrap(), external);
}

#[test]
fn r_reloads_the_changed_bundle_and_drops_the_edit() {
    let (path, mut app, external) = edit_bob_then_change_dee_on_disk("bundle-reload");
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.mode(), Mode::Browse);
    assert_eq!(
        listed(&app),
        ["Anna Adams", "Bob Brown", "Cy Cole", "Dora Diaz"]
    );
    assert_eq!(fs::read_to_string(path).unwrap(), external);
}

#[test]
fn o_overwrites_the_bundle_with_the_edit_and_the_other_cards_as_loaded() {
    let (path, mut app, _) = edit_bob_then_change_dee_on_disk("bundle-overwrite");
    press(&mut app, KeyCode::Char('o'));
    assert_eq!(app.mode(), Mode::Browse);
    let [cy, bob, dee] = bundle();
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        [cy, bob.replace("Bob", "Rob"), dee].concat()
    );
    assert_eq!(
        listed(&app),
        ["Anna Adams", "Rob Brown", "Cy Cole", "Dee Diaz"]
    );
}

#[test]
fn shift_r_picks_up_cards_added_to_or_removed_from_a_bundle() {
    let [cy, bob, dee] = bundle();
    let (path, mut app) = open_bundle("bundle-shift-r", &[&cy, &bob, &dee]);
    let eve = card(
        "Evans;Eve;;;",
        "Eve Evans",
        "+1 555 0104",
        "eve@example.org",
    );
    fs::write(&path, [cy, dee, eve].concat()).unwrap();
    press(&mut app, KeyCode::Char('R'));
    assert_eq!(
        listed(&app),
        ["Anna Adams", "Cy Cole", "Dee Diaz", "Eve Evans"]
    );
}

#[test]
fn a_defective_card_in_a_bundle_is_skipped_alone_and_written_back_untouched() {
    let [cy, bob, dee] = bundle();
    let eve = card(
        "Evans;Eve;;;",
        "Eve Evans",
        "+1 555 0105",
        "eve@example.org",
    );
    let fay = card("Ford;Fay;;;", "Fay Ford", "+1 555 0106", "fay@example.org");
    let broken = "BEGIN:VCARD\r\nFN:Broken\r\n";
    let (path, mut app) = open_bundle("bundle-defect", &[&cy, &bob, &dee, broken, &eve, &fay]);
    assert_eq!(
        listed(&app),
        [
            "Anna Adams",
            "Bob Brown",
            "Cy Cole",
            "Dee Diaz",
            "Eve Evans",
            "Fay Ford"
        ]
    );
    assert!(screen(&app).last().unwrap().contains("1 card skipped"));

    press(&mut app, KeyCode::Char('!'));
    let screen = screen_of_width(&app, 240);
    let row = row_of(&screen, &format!("{} #4", path.display()));
    assert!(
        screen[row + 1].contains("no END:VCARD"),
        "{}",
        screen.join("\n")
    );
    press(&mut app, KeyCode::Esc);

    select(&mut app, "Eve Evans");
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Eve", "Eva");
    save(&mut app);
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        [&cy, &bob, &dee, broken, &eve.replace("Eve", "Eva"), &fay].concat()
    );
}

#[test]
fn an_invalid_utf8_card_in_a_bundle_is_skipped_alone() {
    let [cy, _, dee] = bundle();
    let dir = address_book("bundle-utf8", &[]);
    let path = dir.join("bundle.vcf");
    let broken: &[u8] = b"BEGIN:VCARD\r\nFN:\xff\r\nEND:VCARD\r\n";
    fs::write(&path, [cy.as_bytes(), broken, dee.as_bytes()].concat()).unwrap();
    let mut app = App::new(vdir::load(&dir).unwrap());
    assert_eq!(listed(&app), ["Cy Cole", "Dee Diaz"]);

    press(&mut app, KeyCode::Char('!'));
    let screen = screen_of_width(&app, 240);
    let row = row_of(&screen, &format!("{} #2", path.display()));
    assert!(
        screen[row + 1].contains("invalid UTF-8"),
        "{}",
        screen.join("\n")
    );
}

#[test]
fn a_thunderbird_export_loads_every_card() {
    let dir = address_book("bundle-thunderbird", &[]);
    fs::copy(
        fixtures_dir().join("bundle-thunderbird.vcf"),
        dir.join("export.vcf"),
    )
    .unwrap();
    let mut app = App::new(vdir::load(&dir).unwrap());
    assert!(app.skipped().is_empty());
    assert_eq!(
        listed(&app),
        ["Bea Bundle", "Mailing List Co", "Theo Tester"]
    );
    select(&mut app, "Theo Tester");
    assert_shows(
        &detail(&screen_of_width(&app, 200)),
        &["theo@example.org", "every contact ends up in one file."],
    );
}

#[test]
fn o_recreates_a_deleted_bundle_with_the_edit_and_the_other_cards_as_loaded() {
    let [cy, bob, dee] = bundle();
    let (path, mut app) = open_bundle("bundle-deleted", &[&cy, &bob, &dee]);
    select(&mut app, "Bob Brown");
    press(&mut app, KeyCode::Char('e'));
    replace_given_name(&mut app, "Bob", "Rob");
    fs::remove_file(&path).unwrap();
    save(&mut app);
    assert_eq!(app.mode(), Mode::Conflict(Conflict::Deleted));

    press(&mut app, KeyCode::Char('o'));
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        [cy, bob.replace("Bob", "Rob"), dee].concat()
    );
}

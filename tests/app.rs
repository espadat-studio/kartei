use std::fs;
use std::path::{Path, PathBuf};

use kartei::app::{App, Mode};
use kartei::vdir::AddressBook;
use kartei::{ui, vdir};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

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
    assert_eq!(app.cards().len(), 9);
    assert!(
        screen.last().unwrap().contains("4 cards skipped"),
        "{}",
        screen.join("\n")
    );
    for name in ["No Begin", "No End", "First", "Second", "Bad"] {
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
        ("bad-multiple-vcards.vcf", "more than one VCARD"),
        ("bad-invalid-utf8.vcf", "invalid UTF-8"),
    ] {
        let row = row_of(&screen, &dir.join(file).display().to_string());
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
    type_text(&mut app, "a");
    assert_eq!(listed(&app), ["Anna Adams", "Cara Chen"]);
    type_text(&mut app, "DA");
    assert_eq!(listed(&app), ["Anna Adams"]);
    let screen = screen(&app);
    assert!(
        screen.last().unwrap().contains("/aDA"),
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

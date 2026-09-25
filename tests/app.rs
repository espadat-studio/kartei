use std::fs;
use std::path::PathBuf;

use kartei::app::App;
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
    let mut terminal = Terminal::new(TestBackend::new(80, 12)).unwrap();
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
    let mut app = App::new(Vec::new());
    press(&mut app, KeyCode::Char('j'));
    press(&mut app, KeyCode::Char('G'));
    assert!(app.selected_card().is_none());
    screen(&app);
}

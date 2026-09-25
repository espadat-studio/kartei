use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListState, Paragraph};

use crate::app::{App, Mode};
use crate::card::{Card, Field};
use crate::form::{FIELDS, Form};

const HINTS: &str = " j/k move  / search  e edit  ? help  q quit";
const EDIT_HINTS: &str = " Tab/S-Tab move  Ctrl-s save  Esc cancel";
const LABEL_WIDTH: u16 = 13;

const KEYMAP: &[(&str, &[(&str, &str)])] = &[
    (
        "Browse",
        &[
            ("j/k Down/Up", "move"),
            ("g/G", "top/bottom"),
            ("/", "search"),
            ("e/Enter", "edit card"),
            ("!", "list skipped cards"),
            ("?", "show keys"),
            ("q", "quit"),
        ],
    ),
    (
        "Search",
        &[
            ("type", "filter by name, company, email, phone"),
            ("Backspace", "delete"),
            ("Enter", "keep filter"),
            ("Esc", "clear filter"),
        ],
    ),
    (
        "Edit",
        &[
            ("Tab/S-Tab Up/Down", "next/previous field"),
            ("Ctrl-s", "save"),
            ("Esc", "cancel"),
        ],
    ),
    ("Prompt, Help", &[("any key", "close")]),
];

pub fn draw(frame: &mut Frame, app: &App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let [list, detail] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(main);

    let title = match app.query() {
        "" => "Cards".to_owned(),
        query => format!("Cards /{query}"),
    };
    let names = app.cards().into_iter().map(Card::display_name);
    let mut state = ListState::default().with_selected(Some(app.selected()));
    frame.render_stateful_widget(
        List::new(names)
            .block(Block::bordered().title(title))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED)),
        list,
        &mut state,
    );

    match app.form() {
        Some(form) => draw_form(frame, form, detail),
        None => {
            let lines = app.selected_card().map(details).unwrap_or_default();
            frame.render_widget(Paragraph::new(lines).block(Block::bordered()), detail);
        }
    }
    let hints = match app.mode() {
        Mode::Search => format!(" /{}  (Enter keep, Esc clear)", app.query()),
        Mode::Edit => EDIT_HINTS.to_owned(),
        Mode::Discard => " Discard unsaved changes? y/n".to_owned(),
        _ => HINTS.to_owned(),
    };
    let hints = match app.error() {
        Some(error) => Paragraph::new(format!(" {error}")).style(Color::Red),
        None => Paragraph::new(hints),
    };
    frame.render_widget(hints, status);
    if let Some(skipped) = skipped_count(app.skipped().len()) {
        frame.render_widget(Paragraph::new(skipped).right_aligned(), status);
    }
    match app.mode() {
        Mode::Prompt => draw_overlay(frame, "Skipped cards", skipped(app)),
        Mode::Help => draw_overlay(frame, "Keys", keymap()),
        Mode::Browse | Mode::Search | Mode::Edit | Mode::Discard => {}
    }
}

fn draw_form(frame: &mut Frame, form: &Form, area: Rect) {
    let mut lines = Vec::new();
    let mut cursor = None;
    for (i, ((field, label), input)) in FIELDS.iter().zip(form.inputs()).enumerate() {
        let style = match i == form.focus() {
            true => Style::new().add_modifier(Modifier::BOLD),
            false => Style::new(),
        };
        if i == form.focus() {
            let (row, col) = input.cursor();
            let y = area.y + 1 + lines.len() as u16 + row;
            cursor = Some(Position::new(area.x + 1 + LABEL_WIDTH + col, y));
        }
        for (row, text) in input.text().split('\n').enumerate() {
            let label = if row == 0 { label } else { "" };
            let mut line = Line::from(vec![
                Span::styled(
                    format!("{label:<width$}", width = LABEL_WIDTH.into()),
                    style,
                ),
                Span::raw(text.to_owned()),
            ]);
            if *field == Field::DisplayName {
                let link = if form.is_linked() {
                    "  (linked)"
                } else {
                    "  (custom)"
                };
                line.push_span(Span::styled(link, Modifier::DIM));
            }
            lines.push(line);
        }
    }
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title("Edit")),
        area,
    );
    if let Some(cursor) = cursor {
        frame.set_cursor_position(cursor);
    }
}

fn skipped(app: &App) -> Vec<Line<'static>> {
    app.skipped()
        .iter()
        .flat_map(|skipped| {
            [
                Line::from(skipped.path.display().to_string()),
                Line::from(format!("  {}", skipped.defect)),
            ]
        })
        .collect()
}

fn keymap() -> Vec<Line<'static>> {
    KEYMAP
        .iter()
        .flat_map(|(mode, keys)| {
            let keys = keys
                .iter()
                .map(|(key, action)| Line::from(format!("  {key:<12} {action}")));
            std::iter::once(Line::styled(*mode, Modifier::BOLD)).chain(keys)
        })
        .collect()
}

fn draw_overlay(frame: &mut Frame, title: &str, lines: Vec<Line>) {
    let area = frame.area().inner(Margin::new(2, 1));
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .title(title)
                .title_bottom(" any key closes "),
        ),
        area,
    );
}

fn skipped_count(count: usize) -> Option<String> {
    match count {
        0 => None,
        1 => Some("1 card skipped (! list) ".into()),
        n => Some(format!("{n} cards skipped (! list) ")),
    }
}

fn details(card: &Card) -> Vec<Line<'static>> {
    let mut lines = vec![Line::styled(card.display_name(), Modifier::BOLD)];
    if let Some(org) = card.organization() {
        lines.extend(field("Company", None, &org.company));
        lines.extend(field("Department", None, &org.department));
    }
    lines.push(Line::default());
    for phone in card.phones() {
        lines.extend(field("Phone", phone.label.as_deref(), &phone.value));
    }
    for email in card.emails() {
        lines.extend(field("Email", email.label.as_deref(), &email.value));
    }
    for address in card.addresses() {
        lines.push(Line::from(heading("Address", address.label.as_deref())));
        let adr = address.value;
        let place = [adr.postal_code, adr.city]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let parts = [adr.street, place, adr.region, adr.country];
        let parts: Vec<_> = parts.into_iter().filter(|p| !p.is_empty()).collect();
        lines.extend(indented(&parts.join("\n")));
    }
    if let Some(birthday) = card.birthday() {
        lines.extend(field("Birthday", None, &birthday.to_string()));
    }
    for url in card.urls() {
        lines.extend(field("URL", None, &url));
    }
    if let Some(note) = card.note() {
        lines.push(Line::from("Note"));
        lines.extend(indented(&note));
    }
    lines
}

fn field(kind: &str, label: Option<&str>, value: &str) -> Option<Line<'static>> {
    (!value.is_empty()).then(|| Line::from(format!("{}  {value}", heading(kind, label))))
}

fn heading(kind: &str, label: Option<&str>) -> String {
    match label {
        Some(label) => format!("{kind} ({label})"),
        None => kind.to_owned(),
    }
}

fn indented(text: &str) -> Vec<Line<'static>> {
    text.lines()
        .map(|line| Line::from(format!("  {line}")))
        .collect()
}

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListState, Paragraph};

use crate::app::{App, Mode};
use crate::card::Card;
use crate::form::Form;
use crate::vdir::Conflict;

const HINTS: &str = " / search  e edit  E raw edit  n new  d delete  y copy  ? help  q quit";
const EDIT_HINTS: &str = " Ctrl-s save  Esc cancel  Tab move  Alt-a/d add/remove  Alt-l label";
const LABEL_WIDTH: u16 = 13;
const ANY_KEY: &str = " any key closes ";

const KEYMAP: &[(&str, &[(&str, &str)])] = &[
    (
        "Browse",
        &[
            ("j/k Down/Up", "move"),
            ("g/G", "top/bottom"),
            ("/", "search"),
            ("e/Enter", "edit card"),
            ("E", "edit raw vCard"),
            ("n", "new card"),
            ("d", "delete card"),
            ("y", "copy a value"),
            ("R", "reload from disk"),
            ("!", "list skipped cards"),
            ("?", "show keys"),
            ("q", "quit"),
        ],
    ),
    (
        "Search",
        &[
            ("type", "filter cards"),
            ("Backspace", "delete"),
            ("Enter", "keep filter"),
            ("Esc", "clear filter"),
        ],
    ),
    ("Prompt, Help", &[("any key", "close")]),
    (
        "Edit",
        &[
            ("Tab/Down", "next field"),
            ("S-Tab/Up", "previous field"),
            ("Alt-a/Alt-d", "add/remove value"),
            ("Alt-l", "cycle label"),
            ("Ctrl-s", "save"),
            ("Esc", "cancel"),
        ],
    ),
    (
        "Conflict",
        &[
            ("r", "reload, drop edit/delete"),
            ("o", "overwrite/recreate/delete"),
            ("Esc", "back"),
        ],
    ),
    ("Copy", &[("1-9", "copy value"), ("Esc", "cancel")]),
    ("Delete", &[("y", "delete card"), ("any key", "cancel")]),
    ("Invalid", &[("e", "edit again"), ("d", "discard")]),
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
        Mode::Copy => " 1-9 copy  Esc close".to_owned(),
        Mode::Edit => EDIT_HINTS.to_owned(),
        Mode::Invalid(reason) => format!(
            " e edit again  d discard  (not saved: {})",
            app.error().unwrap_or(reason)
        ),
        Mode::Discard => " Discard unsaved changes? y/n".to_owned(),
        Mode::Conflict(Conflict::Changed) => {
            " Card changed on disk: r reload  o overwrite  Esc keep editing".to_owned()
        }
        Mode::Conflict(Conflict::Deleted) => {
            " Card deleted on disk: r reload  o recreate  Esc keep editing".to_owned()
        }
        Mode::Delete => format!(
            " Delete {}? y/n",
            app.selected_card()
                .map(Card::display_name)
                .unwrap_or_default()
        ),
        Mode::DeleteConflict { is_bundle: false } => {
            " Card changed on disk: r reload  o delete anyway  Esc cancel".to_owned()
        }
        Mode::DeleteConflict { is_bundle: true } => {
            " Bundle changed on disk: r reload  Esc cancel".to_owned()
        }
        _ => HINTS.to_owned(),
    };
    let hints = match (app.error(), app.status()) {
        _ if matches!(app.mode(), Mode::Invalid(_)) => Paragraph::new(hints),
        (Some(error), _) => Paragraph::new(format!(" {error}")).style(Color::Red),
        (None, Some(status)) => Paragraph::new(format!(" {status}")),
        (None, None) => Paragraph::new(hints),
    };
    let count = skipped_count(app.skipped().len()).unwrap_or_default();
    let [hints_area, count_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(count.len() as u16)])
            .areas(status);
    frame.render_widget(hints, hints_area);
    frame.render_widget(Paragraph::new(count), count_area);
    match app.mode() {
        Mode::Prompt => draw_overlay(frame, "Skipped cards", ANY_KEY, vec![skipped(app)]),
        Mode::Help => draw_overlay(frame, "Keys", ANY_KEY, keymap()),
        Mode::Copy => draw_overlay(frame, "Copy", " 1-9 copy  Esc close ", vec![copyable(app)]),
        Mode::Browse
        | Mode::Search
        | Mode::Edit
        | Mode::Discard
        | Mode::Conflict(_)
        | Mode::Delete
        | Mode::DeleteConflict { .. }
        | Mode::Invalid(_) => {}
    }
}

fn draw_form(frame: &mut Frame, form: &Form, area: Rect) {
    let mut lines = Vec::new();
    let mut cursor = None;
    for (i, row) in form.rows().into_iter().enumerate() {
        let style = match i == form.focus() {
            true => Style::new().add_modifier(Modifier::BOLD),
            false => Style::new(),
        };
        if i == form.focus() {
            let (y, x) = row.input.cursor();
            cursor = Some((lines.len() as u16 + y, LABEL_WIDTH + x));
        }
        for (n, text) in row.input.text().split('\n').enumerate() {
            let label = if n == 0 { row.label } else { "" };
            let mut line = Line::from(vec![
                Span::styled(
                    format!("{label:<width$}", width = LABEL_WIDTH.into()),
                    style,
                ),
                Span::raw(text.to_owned()),
            ]);
            if let Some(suffix) = row.suffix.as_ref().filter(|_| n == 0) {
                line.push_span(Span::styled(format!("  {suffix}"), Modifier::DIM));
            }
            lines.push(line);
        }
    }
    let height = area.height.saturating_sub(2);
    let scroll = cursor.map_or(0, |(y, _)| (y + 1).saturating_sub(height));
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((scroll, 0))
            .block(Block::bordered().title("Edit")),
        area,
    );
    if let Some((y, x)) = cursor {
        frame.set_cursor_position(Position::new(area.x + 1 + x, area.y + 1 + y - scroll));
    }
}

fn copyable(app: &App) -> Vec<Line<'static>> {
    app.copyable()
        .into_iter()
        .zip(1..)
        .map(|((kind, label, value), n)| {
            let value = value.replace('\n', ", ");
            Line::from(format!("{n}  {}  {value}", heading(kind, label.as_deref())))
        })
        .collect()
}

fn skipped(app: &App) -> Vec<Line<'static>> {
    app.skipped()
        .iter()
        .flat_map(|skipped| {
            [
                Line::from(format!(
                    "{}{}",
                    skipped.path.display(),
                    skipped
                        .position
                        .map(|n| format!(" #{n}"))
                        .unwrap_or_default()
                )),
                Line::from(format!("  {}", skipped.defect)),
            ]
        })
        .collect()
}

fn keymap() -> Vec<Vec<Line<'static>>> {
    let (left, right) = KEYMAP.split_at(3);
    [left, right]
        .map(|modes| {
            modes
                .iter()
                .flat_map(|(mode, keys)| {
                    let keys = keys
                        .iter()
                        .map(|(key, action)| Line::from(format!("  {key:<12} {action}")));
                    std::iter::once(Line::styled(*mode, Modifier::BOLD)).chain(keys)
                })
                .collect()
        })
        .into()
}

fn draw_overlay(frame: &mut Frame, title: &str, footer: &str, columns: Vec<Vec<Line>>) {
    let area = frame.area().inner(Margin::new(2, 1));
    let block = Block::bordered().title(title).title_bottom(footer);
    frame.render_widget(Clear, area);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let areas = Layout::horizontal(vec![Constraint::Fill(1); columns.len()]).split(inner);
    for (lines, column) in columns.into_iter().zip(areas.iter()) {
        frame.render_widget(Paragraph::new(lines), *column);
    }
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
        lines.extend(indented(&address.value.to_string()));
    }
    if let Some(birthday) = card.birthday() {
        lines.extend(field("Birthday", None, &birthday.to_string()));
    }
    for url in card.urls() {
        lines.extend(field("URL", url.label.as_deref(), &url.value));
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

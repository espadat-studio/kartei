use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListState, Paragraph};

use crate::app::{App, Mode};
use crate::card::Card;

const HINTS: &str = " j/k move  g/G top/bottom  q quit";

pub fn draw(frame: &mut Frame, app: &App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let [list, detail] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(main);

    let names = app.cards().iter().map(Card::display_name);
    let mut state = ListState::default().with_selected(Some(app.selected()));
    frame.render_stateful_widget(
        List::new(names)
            .block(Block::bordered().title("Cards"))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED)),
        list,
        &mut state,
    );

    let lines = app.selected_card().map(details).unwrap_or_default();
    frame.render_widget(Paragraph::new(lines).block(Block::bordered()), detail);
    frame.render_widget(Paragraph::new(HINTS), status);
    if let Some(skipped) = skipped_count(app.skipped().len()) {
        frame.render_widget(Paragraph::new(skipped).right_aligned(), status);
    }
    if app.mode() == Mode::Prompt {
        draw_skipped(frame, app);
    }
}

fn draw_skipped(frame: &mut Frame, app: &App) {
    let area = frame.area().inner(Margin::new(2, 1));
    let lines: Vec<Line> = app
        .skipped()
        .iter()
        .flat_map(|skipped| {
            [
                Line::from(skipped.path.display().to_string()),
                Line::from(format!("  {}", skipped.defect)),
            ]
        })
        .collect();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .title("Skipped cards")
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

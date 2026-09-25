use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListState, Paragraph};

use crate::app::App;
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
}

fn details(card: &Card) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled(card.display_name(), Modifier::BOLD),
        Line::default(),
    ];
    lines.extend(
        card.phones()
            .into_iter()
            .map(|phone| Line::from(format!("Phone  {phone}"))),
    );
    lines.extend(
        card.emails()
            .into_iter()
            .map(|email| Line::from(format!("Email  {email}"))),
    );
    lines
}

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::card::Card;
use crate::vdir::{AddressBook, Skipped};

pub struct App {
    cards: Vec<Card>,
    skipped: Vec<Skipped>,
    selected: usize,
    should_quit: bool,
}

impl App {
    pub fn new(book: AddressBook) -> Self {
        let AddressBook { mut cards, skipped } = book;
        cards.sort_by_cached_key(sort_key);
        Self {
            cards,
            skipped,
            selected: 0,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let last = self.cards.len().saturating_sub(1);
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.selected = (self.selected + 1).min(last),
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => self.selected = last,
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn skipped(&self) -> &[Skipped] {
        &self.skipped
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_card(&self) -> Option<&Card> {
        self.cards.get(self.selected)
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

fn sort_key(card: &Card) -> (String, String, String) {
    let (family, given) = card.structured_name();
    let display_name = card.display_name().to_lowercase();
    if family.is_empty() && given.is_empty() {
        return (display_name, String::new(), String::new());
    }
    (family.to_lowercase(), given.to_lowercase(), display_name)
}

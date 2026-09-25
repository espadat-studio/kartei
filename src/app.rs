use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::card::Card;
use crate::vdir::{AddressBook, Skipped};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Search,
    Prompt,
    Help,
}

pub struct App {
    cards: Vec<Card>,
    skipped: Vec<Skipped>,
    query: String,
    visible: Vec<usize>,
    selected: usize,
    mode: Mode,
    should_quit: bool,
}

impl App {
    pub fn new(book: AddressBook) -> Self {
        let AddressBook { mut cards, skipped } = book;
        cards.sort_by_cached_key(sort_key);
        Self {
            visible: (0..cards.len()).collect(),
            cards,
            skipped,
            query: String::new(),
            selected: 0,
            mode: Mode::Browse,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Browse => self.browse(key),
            Mode::Search => self.search(key),
            Mode::Prompt | Mode::Help => self.mode = Mode::Browse,
        }
    }

    fn browse(&mut self, key: KeyEvent) {
        let last = self.visible.len().saturating_sub(1);
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.selected = (self.selected + 1).min(last),
            KeyCode::Char('k') | KeyCode::Up => self.selected = self.selected.saturating_sub(1),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => self.selected = last,
            KeyCode::Char('/') => self.mode = Mode::Search,
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Char('!') if !self.skipped.is_empty() => self.mode = Mode::Prompt,
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.query.push(c),
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Enter => self.mode = Mode::Browse,
            KeyCode::Esc => {
                self.query.clear();
                self.mode = Mode::Browse;
            }
            _ => return,
        }
        self.filter();
    }

    fn filter(&mut self) {
        let current = self.visible.get(self.selected).copied();
        self.visible = (0..self.cards.len())
            .filter(|&i| is_match(&self.cards[i], &self.query))
            .collect();
        self.selected = current
            .and_then(|card| self.visible.iter().position(|&i| i == card))
            .unwrap_or(0);
    }

    pub fn cards(&self) -> Vec<&Card> {
        self.visible.iter().map(|&i| &self.cards[i]).collect()
    }

    pub fn skipped(&self) -> &[Skipped] {
        &self.skipped
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_card(&self) -> Option<&Card> {
        self.visible.get(self.selected).map(|&i| &self.cards[i])
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}

fn is_match(card: &Card, query: &str) -> bool {
    let needle = query.to_lowercase();
    let org = card.organization().into_iter();
    let emails = card.emails().into_iter().map(|email| email.value);
    let texts = std::iter::once(card.display_name())
        .chain(org.flat_map(|org| [org.company, org.department]))
        .chain(emails);
    if texts
        .into_iter()
        .any(|text| text.to_lowercase().contains(&needle))
    {
        return true;
    }
    let digits: String = query
        .chars()
        .filter(|c| !matches!(c, ' ' | '+' | '-' | '(' | ')'))
        .collect();
    !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && card.phones().iter().any(|phone| {
            let phone: String = phone.value.chars().filter(char::is_ascii_digit).collect();
            phone.contains(&digits)
        })
}

fn sort_key(card: &Card) -> (String, String, String) {
    let (family, given) = card.structured_name();
    let display_name = card.display_name().to_lowercase();
    if family.is_empty() && given.is_empty() {
        return (display_name, String::new(), String::new());
    }
    (family.to_lowercase(), given.to_lowercase(), display_name)
}

use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::card::Card;
use crate::form::Form;
use crate::vdir::{self, AddressBook, Skipped};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Search,
    Edit,
    Discard,
    Prompt,
    Help,
}

pub struct App {
    cards: Vec<(PathBuf, Card)>,
    skipped: Vec<Skipped>,
    query: String,
    visible: Vec<usize>,
    selected: usize,
    mode: Mode,
    form: Option<Form>,
    error: Option<String>,
    should_quit: bool,
}

impl App {
    pub fn new(book: AddressBook) -> Self {
        let AddressBook { mut cards, skipped } = book;
        cards.sort_by_cached_key(|(_, card)| sort_key(card));
        Self {
            visible: (0..cards.len()).collect(),
            cards,
            skipped,
            query: String::new(),
            selected: 0,
            mode: Mode::Browse,
            form: None,
            error: None,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Browse => self.browse(key),
            Mode::Search => self.search(key),
            Mode::Edit => self.edit(key),
            Mode::Discard => self.discard(key),
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
            KeyCode::Char('e') | KeyCode::Enter => self.open_form(),
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
        self.filter(self.visible.get(self.selected).copied());
    }

    fn open_form(&mut self) {
        if let Some(card) = self.selected_card() {
            self.form = Some(Form::new(card));
            self.mode = Mode::Edit;
        }
    }

    fn edit(&mut self, key: KeyEvent) {
        self.error = None;
        let index = self.visible[self.selected];
        let form = self.form.as_mut().expect("edit mode has a form");
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => self.save(index),
            KeyCode::Esc if form.apply(&self.cards[index].1) != self.cards[index].1 => {
                self.mode = Mode::Discard
            }
            KeyCode::Esc => self.close_form(),
            _ => form.handle_key(key),
        }
    }

    fn discard(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') => self.close_form(),
            KeyCode::Char('n') | KeyCode::Esc => self.mode = Mode::Edit,
            _ => {}
        }
    }

    fn save(&mut self, index: usize) {
        let (path, card) = &self.cards[index];
        let edited = self
            .form
            .as_ref()
            .expect("edit mode has a form")
            .apply(card);
        if edited != *card
            && let Err(err) = vdir::save(path, &edited.to_bytes())
        {
            self.error = Some(format!("save failed: {err}"));
            return;
        }
        let path = path.clone();
        self.cards[index].1 = edited;
        self.close_form();
        self.cards.sort_by_cached_key(|(_, card)| sort_key(card));
        self.filter(self.cards.iter().position(|(p, _)| *p == path));
    }

    fn close_form(&mut self) {
        self.form = None;
        self.error = None;
        self.mode = Mode::Browse;
    }

    fn filter(&mut self, current: Option<usize>) {
        self.visible = (0..self.cards.len())
            .filter(|&i| is_match(&self.cards[i].1, &self.query))
            .collect();
        self.selected = current
            .and_then(|card| self.visible.iter().position(|&i| i == card))
            .unwrap_or(0);
    }

    pub fn cards(&self) -> Vec<&Card> {
        self.visible.iter().map(|&i| &self.cards[i].1).collect()
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
        self.visible.get(self.selected).map(|&i| &self.cards[i].1)
    }

    pub fn form(&self) -> Option<&Form> {
        self.form.as_ref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
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

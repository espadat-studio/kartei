use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::card::Card;
use crate::form::Form;
use crate::osc52;
use crate::vdir::{self, AddressBook, Conflict, Location, Skipped};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Search,
    Edit,
    Discard,
    Conflict(Conflict),
    Copy,
    Prompt,
    Help,
}

struct Draft {
    location: Location,
    card: Card,
    form: Form,
    is_new: bool,
}

impl Draft {
    fn edited(&self) -> Result<Card, String> {
        self.form.apply(&self.card)
    }
}

pub struct App {
    dir: PathBuf,
    cards: Vec<(Location, Card)>,
    files: HashMap<PathBuf, Vec<Vec<u8>>>,
    skipped: Vec<Skipped>,
    query: String,
    visible: Vec<usize>,
    selected: usize,
    mode: Mode,
    draft: Option<Draft>,
    error: Option<String>,
    status: Option<&'static str>,
    clipboard: Option<String>,
    should_quit: bool,
}

impl App {
    pub fn new(book: AddressBook) -> Self {
        let AddressBook {
            dir,
            mut cards,
            files,
            skipped,
        } = book;
        cards.sort_by_cached_key(|(_, card)| sort_key(card));
        Self {
            dir,
            visible: (0..cards.len()).collect(),
            cards,
            files,
            skipped,
            query: String::new(),
            selected: 0,
            mode: Mode::Browse,
            draft: None,
            error: None,
            status: None,
            clipboard: None,
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.error = None;
        self.status = None;
        match self.mode {
            Mode::Browse => self.browse(key),
            Mode::Search => self.search(key),
            Mode::Edit => self.edit(key),
            Mode::Discard => self.discard(key),
            Mode::Conflict(_) => self.conflict(key),
            Mode::Copy => self.copy(key),
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
            KeyCode::Char('n') => self.new_card(),
            KeyCode::Char('R') => self.reload(self.selected_location()),
            KeyCode::Char('y') if !self.copyable().is_empty() => self.mode = Mode::Copy,
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
        if let Some(&index) = self.visible.get(self.selected) {
            let (location, card) = &self.cards[index];
            self.start_draft(location.clone(), card.clone(), false);
        }
    }

    fn new_card(&mut self) {
        let uid = Uuid::new_v4().to_string();
        let path = self.dir.join(format!("{uid}.vcf"));
        self.start_draft((path, 0), Card::new(&uid), true);
    }

    fn start_draft(&mut self, location: Location, card: Card, is_new: bool) {
        self.draft = Some(Draft {
            location,
            form: Form::new(&card),
            card,
            is_new,
        });
        self.mode = Mode::Edit;
    }

    fn edit(&mut self, key: KeyEvent) {
        let draft = self.draft.as_mut().expect("edit mode has a draft");
        match key.code {
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => self.save(),
            KeyCode::Esc if draft.edited().as_ref() != Ok(&draft.card) => self.mode = Mode::Discard,
            KeyCode::Esc => self.close_form(),
            _ => draft.form.handle_key(key),
        }
    }

    fn discard(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') => self.close_form(),
            KeyCode::Char('n') | KeyCode::Esc => self.mode = Mode::Edit,
            _ => {}
        }
    }

    fn conflict(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') => {
                let location = self.draft.take().map(|draft| draft.location);
                self.close_form();
                self.reload(location);
            }
            KeyCode::Char('o') => self.write(),
            KeyCode::Esc => self.mode = Mode::Edit,
            _ => {}
        }
    }

    fn copy(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c @ '1'..='9') => {
                let n = c as usize - '1' as usize;
                if let Some((_, _, value)) = self.copyable().get(n) {
                    self.clipboard = Some(osc52::sequence(value));
                    self.status = Some("copied");
                    self.mode = Mode::Browse;
                }
            }
            KeyCode::Esc => self.mode = Mode::Browse,
            _ => {}
        }
    }

    fn save(&mut self) {
        let draft = self.draft.as_ref().expect("edit mode has a draft");
        let is_edited = draft.edited().is_ok_and(|edited| edited != draft.card);
        if draft.is_new || !is_edited {
            return self.write();
        }
        let (path, _) = &draft.location;
        match vdir::conflict(path, &self.loaded(path).concat()) {
            Ok(None) => self.write(),
            Ok(Some(conflict)) => self.mode = Mode::Conflict(conflict),
            Err(err) => self.error = Some(format!("save failed: {err}")),
        }
    }

    fn write(&mut self) {
        self.mode = Mode::Edit;
        let draft = self.draft.as_ref().expect("edit mode has a draft");
        let edited = match draft.edited() {
            Ok(edited) => edited,
            Err(err) => {
                self.error = Some(err);
                return;
            }
        };
        if edited == draft.card {
            return self.close_form();
        }
        let location = draft.location.clone();
        let (path, index) = &location;
        let mut chunks = self.loaded(path).to_vec();
        match chunks.get_mut(*index) {
            Some(chunk) => *chunk = edited.to_bytes(),
            None => chunks.push(edited.to_bytes()),
        }
        if let Err(err) = vdir::save(path, &chunks.concat()) {
            self.error = Some(format!("save failed: {err}"));
            return;
        }
        self.files.insert(path.clone(), chunks);
        match self.cards.iter_mut().find(|(l, _)| *l == location) {
            Some((_, card)) => *card = edited,
            None => self.cards.push((location.clone(), edited)),
        }
        self.close_form();
        self.show(Some(&location));
    }

    fn loaded(&self, path: &Path) -> &[Vec<u8>] {
        self.files.get(path).map_or(&[], Vec::as_slice)
    }

    fn reload(&mut self, location: Option<Location>) {
        match vdir::load(&self.dir) {
            Ok(book) => {
                self.cards = book.cards;
                self.files = book.files;
                self.skipped = book.skipped;
                self.show(location.as_ref());
            }
            Err(err) => self.error = Some(format!("reload failed: {err}")),
        }
    }

    fn show(&mut self, location: Option<&Location>) {
        self.cards.sort_by_cached_key(|(_, card)| sort_key(card));
        self.filter(
            location.and_then(|location| self.cards.iter().position(|(l, _)| l == location)),
        );
    }

    fn selected_location(&self) -> Option<Location> {
        self.visible
            .get(self.selected)
            .map(|&i| self.cards[i].0.clone())
    }

    fn close_form(&mut self) {
        self.draft = None;
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

    pub fn copyable(&self) -> Vec<(&'static str, Option<String>, String)> {
        let Some(card) = self.selected_card() else {
            return Vec::new();
        };
        let phones = card
            .phones()
            .into_iter()
            .map(|v| ("Phone", v.label, v.value));
        let emails = card
            .emails()
            .into_iter()
            .map(|v| ("Email", v.label, v.value));
        let addresses = card
            .addresses()
            .into_iter()
            .map(|v| ("Address", v.label, v.value.to_string()));
        let urls = card.urls().into_iter().map(|url| ("URL", None, url));
        phones
            .chain(emails)
            .chain(addresses)
            .chain(urls)
            .filter(|(_, _, value)| !value.is_empty())
            .take(9)
            .collect()
    }

    pub fn take_clipboard(&mut self) -> Option<String> {
        self.clipboard.take()
    }

    pub fn status(&self) -> Option<&str> {
        self.status
    }

    pub fn form(&self) -> Option<&Form> {
        self.draft.as_ref().map(|draft| &draft.form)
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

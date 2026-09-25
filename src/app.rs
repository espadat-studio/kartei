use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use crate::card::{Card, Defect};
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
    Delete,
    DeleteConflict { is_bundle: bool },
    Invalid(&'static str),
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

struct RawEdit {
    location: Location,
    original: Vec<u8>,
    edited: Vec<u8>,
}

pub struct App {
    path: PathBuf,
    cards: Vec<(Location, Card)>,
    files: HashMap<PathBuf, Vec<Vec<u8>>>,
    skipped: Vec<Skipped>,
    query: String,
    visible: Vec<usize>,
    selected: usize,
    mode: Mode,
    draft: Option<Draft>,
    raw: Option<RawEdit>,
    editor_request: Option<Vec<u8>>,
    error: Option<String>,
    status: Option<&'static str>,
    clipboard: Option<String>,
    should_quit: bool,
}

impl App {
    pub fn new(book: AddressBook) -> Self {
        let AddressBook {
            path,
            mut cards,
            files,
            skipped,
        } = book;
        cards.sort_by_cached_key(|(_, card)| sort_key(card));
        Self {
            path,
            visible: (0..cards.len()).collect(),
            cards,
            files,
            skipped,
            query: String::new(),
            selected: 0,
            mode: Mode::Browse,
            draft: None,
            raw: None,
            editor_request: None,
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
            Mode::Delete => self.confirm_delete(key),
            Mode::DeleteConflict { is_bundle } => self.delete_conflict(key, is_bundle),
            Mode::Invalid(_) => self.invalid(key),
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
            KeyCode::Char('E') => self.raw_edit(),
            KeyCode::Char('n') => self.new_card(),
            KeyCode::Char('d') if self.selected_card().is_some() => self.mode = Mode::Delete,
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

    fn raw_edit(&mut self) {
        let Some(location) = self.selected_location() else {
            return;
        };
        let (path, index) = &location;
        let original = self.files[path][*index].clone();
        self.editor_request = Some(original.clone());
        self.raw = Some(RawEdit {
            location,
            edited: original.clone(),
            original,
        });
    }

    pub fn finish_raw_edit(&mut self, output: io::Result<Vec<u8>>) {
        let bytes = match output {
            Ok(bytes) => bytes,
            Err(err) => {
                self.raw = None;
                self.error = Some(format!("edit failed: {err}"));
                return;
            }
        };
        let raw = self.raw.as_mut().expect("editor output answers a raw edit");
        if bytes.trim_ascii().is_empty() {
            self.raw = None;
            self.status = Some("edit aborted");
            return;
        }
        if bytes == raw.original {
            self.raw = None;
            return;
        }
        raw.edited = bytes;
        if let Err(reason) = check_raw(&raw.edited) {
            self.mode = Mode::Invalid(reason);
            return;
        }
        let (path, _) = &raw.location;
        match vdir::conflict(path, &self.files[path].concat()) {
            Ok(None) => self.write_raw(),
            Ok(Some(conflict)) => self.mode = Mode::Conflict(conflict),
            Err(err) => self.reject_raw(format!("save failed: {err}")),
        }
    }

    fn invalid(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('e') => self.reopen_editor(),
            KeyCode::Char('d') => self.close_form(),
            _ => {}
        }
    }

    fn reopen_editor(&mut self) {
        let raw = self.raw.as_ref().expect("raw edit in progress");
        self.editor_request = Some(raw.edited.clone());
        self.mode = Mode::Browse;
    }

    fn reject_raw(&mut self, error: String) {
        self.error = Some(error);
        self.mode = Mode::Invalid("save failed");
    }

    fn write_raw(&mut self) {
        let raw = self.raw.as_ref().expect("raw edit in progress");
        let location = raw.location.clone();
        let (path, index) = &location;
        let mut chunks = self.files[path].clone();
        let mut edited = raw.edited.clone();
        if index + 1 < chunks.len() && !edited.ends_with(b"\n") {
            edited.extend_from_slice(vdir::eol(&edited).as_bytes());
        }
        chunks[*index] = edited;
        if let Err(err) = vdir::save(path, &chunks.concat()) {
            return self.reject_raw(format!("save failed: {err}"));
        }
        self.store(path, chunks);
        self.close_form();
        self.show(Some(&location));
    }

    fn new_card(&mut self) {
        let uid = Uuid::new_v4().to_string();
        let (location, eol) = match self.files.get(&self.path) {
            Some(chunks) => (
                (self.path.clone(), chunks.len()),
                vdir::eol(&chunks.concat()),
            ),
            None => ((self.path.join(format!("{uid}.vcf")), 0), "\r\n"),
        };
        self.start_draft(location, Card::new(&uid, eol), true);
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
                let draft = self.draft.take().map(|draft| draft.location);
                let location = draft.or_else(|| self.raw.take().map(|raw| raw.location));
                self.close_form();
                self.reload(location);
            }
            KeyCode::Char('o') if self.raw.is_some() => self.write_raw(),
            KeyCode::Char('o') => self.write(),
            KeyCode::Esc if self.raw.is_some() => self.reopen_editor(),
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

    fn confirm_delete(&mut self, key: KeyEvent) {
        self.mode = Mode::Browse;
        if key.code != KeyCode::Char('y') {
            return;
        }
        let (path, _) = self
            .selected_location()
            .expect("delete has a selected card");
        let chunks = &self.files[&path];
        match vdir::conflict(&path, &chunks.concat()) {
            Ok(None) => self.delete(),
            Ok(Some(Conflict::Changed)) => {
                let is_bundle = chunks.len() > 1
                    || fs::read(&path).is_ok_and(|bytes| vdir::split(&bytes).len() > 1);
                self.mode = Mode::DeleteConflict { is_bundle }
            }
            Ok(Some(Conflict::Deleted)) => {
                self.store(&path, Vec::new());
                self.status = Some("card already deleted on disk");
                self.keep_position();
            }
            Err(err) => self.error = Some(format!("delete failed: {err}")),
        }
    }

    fn delete_conflict(&mut self, key: KeyEvent, is_bundle: bool) {
        match key.code {
            KeyCode::Char('r') => {
                self.mode = Mode::Browse;
                self.reload(self.selected_location());
            }
            KeyCode::Char('o') if is_bundle => {
                self.error = Some("bundle changed on disk, r reload before deleting".into())
            }
            KeyCode::Char('o') => {
                self.mode = Mode::Browse;
                self.delete();
            }
            KeyCode::Esc => self.mode = Mode::Browse,
            _ => {}
        }
    }

    fn delete(&mut self) {
        let (path, index) = self
            .selected_location()
            .expect("delete has a selected card");
        let mut chunks = self.files[&path].clone();
        chunks.remove(index);
        let result = match chunks.is_empty() && path != self.path {
            true => match vdir::remove(&path) {
                Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
                result => result,
            },
            false => vdir::save(&path, &chunks.concat()),
        };
        if let Err(err) = result {
            self.error = Some(format!("delete failed: {err}"));
            return;
        }
        self.store(&path, chunks);
        self.keep_position();
    }

    fn keep_position(&mut self) {
        let selected = self.selected;
        self.show(None);
        self.selected = selected.min(self.visible.len().saturating_sub(1));
    }

    fn save(&mut self) {
        let draft = self.draft.as_ref().expect("edit mode has a draft");
        let is_edited = draft.edited().is_ok_and(|edited| edited != draft.card);
        let (path, _) = &draft.location;
        let is_new_file = draft.is_new && !self.files.contains_key(path);
        if is_new_file || !is_edited {
            return self.write();
        }
        match vdir::conflict(path, &self.files[path].concat()) {
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
        let chunks = match draft.is_new {
            true => match self.files.get(path) {
                Some(chunks) => vdir::append(chunks.clone(), edited.to_bytes()),
                None => vec![edited.to_bytes()],
            },
            false => {
                let mut chunks = self.files[path].clone();
                chunks[*index] = edited.to_bytes();
                chunks
            }
        };
        if let Err(err) = vdir::save(path, &chunks.concat()) {
            self.error = Some(format!("save failed: {err}"));
            return;
        }
        self.store(path, chunks);
        self.close_form();
        self.show(Some(&location));
    }

    fn store(&mut self, path: &PathBuf, chunks: Vec<Vec<u8>>) {
        self.cards.retain(|((p, _), _)| p != path);
        self.skipped.retain(|skipped| &skipped.path != path);
        if chunks.is_empty() && path != &self.path {
            self.files.remove(path);
            return;
        }
        let (cards, skipped) = vdir::parse(path, &chunks);
        self.cards.extend(cards);
        self.skipped.extend(skipped);
        self.skipped.sort_by(|a, b| a.path.cmp(&b.path));
        self.files.insert(path.clone(), chunks);
    }

    fn reload(&mut self, location: Option<Location>) {
        match vdir::load(&self.path) {
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
        self.raw = None;
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
        let urls = card.urls().into_iter().map(|v| ("URL", v.label, v.value));
        phones
            .chain(emails)
            .chain(addresses)
            .chain(urls)
            .filter(|(_, _, value)| !value.is_empty())
            .take(9)
            .collect()
    }

    pub fn take_editor_request(&mut self) -> Option<Vec<u8>> {
        self.editor_request.take()
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

fn check_raw(bytes: &[u8]) -> Result<(), &'static str> {
    if vdir::split(bytes).len() > 1 {
        return Err("more than one card");
    }
    Card::parse(bytes).map(drop).map_err(Defect::reason)
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

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::card::{self, Birthday, Card, Field, Kind};
use crate::input::Input;

pub const FIELDS: [(Field, &str); 9] = [
    (Field::Prefixes, "Prefix"),
    (Field::Given, "Given name"),
    (Field::Additional, "Middle name"),
    (Field::Family, "Family name"),
    (Field::Suffixes, "Suffix"),
    (Field::DisplayName, "Display name"),
    (Field::Company, "Company"),
    (Field::Department, "Department"),
    (Field::Note, "Note"),
];

const KINDS: [(Kind, &[&str]); 4] = [
    (Kind::Phone, &["Phone"]),
    (Kind::Email, &["Email"]),
    (Kind::Url, &["URL"]),
    (
        Kind::Address,
        &["Street", "Postal code", "City", "Region", "Country"],
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Field(usize),
    Value(usize, usize),
    Birthday,
}

struct Entry {
    kind: Kind,
    origin: Option<usize>,
    label: Option<String>,
    inputs: Vec<Input>,
}

impl Entry {
    fn blank(kind: Kind) -> Self {
        Self {
            kind,
            origin: None,
            label: None,
            inputs: vec![Input::new(String::new()); components(kind).len()],
        }
    }

    fn values(&self) -> Vec<String> {
        self.inputs.iter().map(|i| i.text().to_owned()).collect()
    }

    fn is_empty(&self) -> bool {
        self.inputs.iter().all(|i| i.text().is_empty())
    }
}

pub struct Row<'a> {
    pub label: &'static str,
    pub input: &'a Input,
    pub suffix: Option<String>,
}

pub struct Form {
    fields: Vec<Input>,
    entries: Vec<Entry>,
    removed: Vec<(Kind, usize)>,
    birthday: Input,
    loaded_birthday: String,
    is_birthday_read_only: bool,
    focus: usize,
    is_linked: bool,
}

impl Form {
    pub fn new(card: &Card) -> Self {
        let mut entries = Vec::new();
        for (kind, _) in KINDS {
            let existing = card.entries(kind);
            if existing.is_empty() {
                entries.push(Entry::blank(kind));
            }
            entries.extend(existing.into_iter().enumerate().map(|(n, e)| Entry {
                kind,
                origin: Some(n),
                label: e.label,
                inputs: e.value.into_iter().map(Input::new).collect(),
            }));
        }
        let birthday = card.birthday();
        let loaded_birthday = birthday.as_ref().map(Birthday::to_iso).unwrap_or_default();
        Self {
            fields: FIELDS.map(|(field, _)| Input::new(card.get(field))).into(),
            entries,
            removed: Vec::new(),
            birthday: Input::new(loaded_birthday.clone()),
            loaded_birthday,
            is_birthday_read_only: birthday.is_some_and(|b| b.is_read_only()),
            focus: 0,
            is_linked: card.display_name() == card::derived_display_name(|f| card.get(f)),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let slots = self.slots();
        let count = slots.len();
        let slot = slots[self.focus];
        let is_alt = key.modifiers.contains(KeyModifiers::ALT);
        match (key.code, slot) {
            (KeyCode::Tab | KeyCode::Down, _) => self.focus = (self.focus + 1) % count,
            (KeyCode::BackTab | KeyCode::Up, _) => self.focus = (self.focus + count - 1) % count,
            (KeyCode::Char('a'), Slot::Value(e, _)) if is_alt => self.add(e),
            (KeyCode::Char('d'), Slot::Value(e, _)) if is_alt => self.remove(e),
            (KeyCode::Char('l'), Slot::Value(e, _)) if is_alt => {
                let entry = &mut self.entries[e];
                entry.label = Some(entry.kind.next_label(entry.label.as_deref()).to_owned());
            }
            _ => self.type_key(key, slot),
        }
    }

    fn type_key(&mut self, key: KeyEvent, slot: Slot) {
        if slot == Slot::Birthday && self.is_birthday_read_only {
            return;
        }
        let is_multiline = match slot {
            Slot::Value(e, 0) => self.entries[e].kind == Kind::Address,
            slot => slot == Slot::Field(index_of(Field::Note)),
        };
        let input = self.input_mut(slot);
        let before = input.text().to_owned();
        match key.code {
            KeyCode::Left => input.left(),
            KeyCode::Right => input.right(),
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Enter if is_multiline => input.insert('\n'),
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                input.insert(c)
            }
            _ => {}
        }
        if let Slot::Field(edited) = slot
            && self.fields[edited].text() != before
        {
            self.link_display_name(edited);
        }
    }

    fn add(&mut self, e: usize) {
        self.entries
            .insert(e + 1, Entry::blank(self.entries[e].kind));
        self.focus_entry(e + 1);
    }

    fn remove(&mut self, e: usize) {
        let entry = self.entries.remove(e);
        self.removed.extend(entry.origin.map(|n| (entry.kind, n)));
        if !self.entries.iter().any(|other| other.kind == entry.kind) {
            self.entries.insert(e, Entry::blank(entry.kind));
        }
        self.focus_entry(e.min(self.entries.len() - 1));
    }

    fn focus_entry(&mut self, e: usize) {
        self.focus = self
            .slots()
            .iter()
            .position(|slot| *slot == Slot::Value(e, 0))
            .expect("every entry has a slot");
    }

    fn link_display_name(&mut self, edited: usize) {
        let derived = card::derived_display_name(|f| self.text(f).to_owned());
        let display_name = index_of(Field::DisplayName);
        if edited == display_name {
            self.is_linked = self.fields[display_name].text() == derived;
        } else if self.is_linked && self.fields[display_name].text() != derived {
            self.fields[display_name].replace(derived);
        }
    }

    fn text(&self, field: Field) -> &str {
        self.fields[index_of(field)].text()
    }

    fn slots(&self) -> Vec<Slot> {
        let note = index_of(Field::Note);
        let values = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(e, entry)| (0..entry.inputs.len()).map(move |c| Slot::Value(e, c)));
        (0..note)
            .map(Slot::Field)
            .chain(values)
            .chain([Slot::Birthday, Slot::Field(note)])
            .collect()
    }

    fn input(&self, slot: Slot) -> &Input {
        match slot {
            Slot::Field(i) => &self.fields[i],
            Slot::Value(e, c) => &self.entries[e].inputs[c],
            Slot::Birthday => &self.birthday,
        }
    }

    fn input_mut(&mut self, slot: Slot) -> &mut Input {
        match slot {
            Slot::Field(i) => &mut self.fields[i],
            Slot::Value(e, c) => &mut self.entries[e].inputs[c],
            Slot::Birthday => &mut self.birthday,
        }
    }

    pub fn apply(&self, card: &Card) -> Result<Card, String> {
        let mut card = card.clone();
        for ((field, _), input) in FIELDS.iter().zip(&self.fields) {
            card.set(*field, input.text());
        }
        let birthday = self.birthday.text();
        if birthday != self.loaded_birthday {
            let birthday = match birthday {
                "" => None,
                text => Some(Birthday::from_iso(text).ok_or(BIRTHDAY_ERROR)?),
            };
            card.set_birthday(birthday.as_ref());
        }
        let mut removed = self.removed.clone();
        for entry in &self.entries {
            match entry.origin {
                Some(n) if entry.is_empty() => removed.push((entry.kind, n)),
                Some(n) => card.update(entry.kind, n, &entry.values(), entry.label.as_deref()),
                None => {}
            }
        }
        removed.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
        for (kind, n) in removed {
            card.remove(kind, n);
        }
        for entry in self.entries.iter().filter(|e| e.origin.is_none()) {
            if !entry.is_empty() {
                card.add(entry.kind, &entry.values(), entry.label.as_deref());
            }
        }
        Ok(card)
    }

    pub fn rows(&self) -> Vec<Row<'_>> {
        self.slots()
            .into_iter()
            .map(|slot| {
                let (label, suffix) = match slot {
                    Slot::Field(i) => {
                        let suffix =
                            (FIELDS[i].0 == Field::DisplayName).then(|| match self.is_linked {
                                true => "(linked)".to_owned(),
                                false => "(custom)".to_owned(),
                            });
                        (FIELDS[i].1, suffix)
                    }
                    Slot::Value(e, c) => {
                        let entry = &self.entries[e];
                        let label = entry.label.as_ref().filter(|_| c == 0);
                        (components(entry.kind)[c], label.map(|l| format!("({l})")))
                    }
                    Slot::Birthday if self.is_birthday_read_only => {
                        ("Birthday", Some("(read-only)".to_owned()))
                    }
                    Slot::Birthday => (
                        "Birthday",
                        self.birthday
                            .text()
                            .is_empty()
                            .then(|| BIRTHDAY_FORMAT.to_owned()),
                    ),
                };
                Row {
                    label,
                    input: self.input(slot),
                    suffix,
                }
            })
            .collect()
    }

    pub fn focus(&self) -> usize {
        self.focus
    }

    pub fn is_linked(&self) -> bool {
        self.is_linked
    }
}

const BIRTHDAY_FORMAT: &str = "(YYYY-MM-DD or --MM-DD)";
const BIRTHDAY_ERROR: &str = "invalid birthday: use YYYY-MM-DD or --MM-DD";

fn components(kind: Kind) -> &'static [&'static str] {
    KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, components)| *components)
        .expect("every kind has components")
}

fn index_of(field: Field) -> usize {
    FIELDS
        .iter()
        .position(|(f, _)| *f == field)
        .expect("every field has an input")
}

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::card::{self, Card, Field};
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

pub struct Form {
    inputs: Vec<Input>,
    focus: usize,
    is_linked: bool,
}

impl Form {
    pub fn new(card: &Card) -> Self {
        Self {
            inputs: FIELDS.map(|(field, _)| Input::new(card.get(field))).into(),
            focus: 0,
            is_linked: card.display_name() == card::derived_display_name(|f| card.get(f)),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let count = self.inputs.len();
        let edited = self.focus;
        let before = self.inputs[edited].text().to_owned();
        let is_note = FIELDS[self.focus].0 == Field::Note;
        let input = &mut self.inputs[self.focus];
        match key.code {
            KeyCode::Tab | KeyCode::Down => self.focus = (self.focus + 1) % count,
            KeyCode::BackTab | KeyCode::Up => self.focus = (self.focus + count - 1) % count,
            KeyCode::Left => input.left(),
            KeyCode::Right => input.right(),
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Enter if is_note => input.insert('\n'),
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                input.insert(c)
            }
            _ => return,
        }
        if self.inputs[edited].text() != before {
            self.link_display_name(edited);
        }
    }

    fn link_display_name(&mut self, edited: usize) {
        let derived = card::derived_display_name(|f| self.text(f).to_owned());
        let display_name = index_of(Field::DisplayName);
        if edited == display_name {
            self.is_linked = self.inputs[display_name].text() == derived;
        } else if self.is_linked && self.inputs[display_name].text() != derived {
            self.inputs[display_name].replace(derived);
        }
    }

    fn text(&self, field: Field) -> &str {
        self.inputs[index_of(field)].text()
    }

    pub fn apply(&self, card: &Card) -> Card {
        let mut card = card.clone();
        for ((field, _), input) in FIELDS.iter().zip(&self.inputs) {
            card.set(*field, input.text());
        }
        card
    }

    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn focus(&self) -> usize {
        self.focus
    }

    pub fn is_linked(&self) -> bool {
        self.is_linked
    }
}

fn index_of(field: Field) -> usize {
    FIELDS
        .iter()
        .position(|(f, _)| *f == field)
        .expect("every field has an input")
}

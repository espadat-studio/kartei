mod bday;
mod label;
mod line;

use std::fmt;

pub use bday::Birthday;
pub use label::next as next_label;
pub use line::ContentLine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Labeled<T> {
    pub label: Option<String>,
    pub value: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub po_box: String,
    pub extended: String,
    pub street: String,
    pub city: String,
    pub region: String,
    pub postal_code: String,
    pub country: String,
}

impl Address {
    fn parse(value: &str) -> Self {
        let mut components = line::split_unescaped(value, ';')
            .into_iter()
            .map(line::unescape);
        let mut next = || components.next().unwrap_or_default();
        Self {
            po_box: next(),
            extended: next(),
            street: next(),
            city: next(),
            region: next(),
            postal_code: next(),
            country: next(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pub company: String,
    pub department: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Prefixes,
    Given,
    Additional,
    Family,
    Suffixes,
    DisplayName,
    Company,
    Department,
    Note,
}

impl Field {
    fn property(self) -> (&'static str, Option<usize>) {
        match self {
            Self::Family => ("N", Some(0)),
            Self::Given => ("N", Some(1)),
            Self::Additional => ("N", Some(2)),
            Self::Prefixes => ("N", Some(3)),
            Self::Suffixes => ("N", Some(4)),
            Self::DisplayName => ("FN", None),
            Self::Company => ("ORG", Some(0)),
            Self::Department => ("ORG", Some(1)),
            Self::Note => ("NOTE", None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Phone,
    Email,
}

impl Kind {
    fn property(self) -> &'static str {
        match self {
            Self::Phone => "TEL",
            Self::Email => "EMAIL",
        }
    }
}

pub fn derived_display_name(part: impl Fn(Field) -> String) -> String {
    [
        Field::Prefixes,
        Field::Given,
        Field::Additional,
        Field::Family,
        Field::Suffixes,
    ]
    .map(part)
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Defect {
    NoBegin,
    NoEnd,
    MultipleCards,
    InvalidUtf8,
}

impl fmt::Display for Defect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::NoBegin => "no BEGIN:VCARD",
            Self::NoEnd => "no END:VCARD",
            Self::MultipleCards => "more than one VCARD",
            Self::InvalidUtf8 => "invalid UTF-8",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    lines: Vec<ContentLine>,
}

impl Card {
    pub fn parse(bytes: &[u8]) -> Result<Self, Defect> {
        let lines = line::split(bytes).ok_or(Defect::InvalidUtf8)?;
        let mut content = lines.iter().filter(|l| !l.is_blank());
        if !content.next().is_some_and(|l| l.is("BEGIN", "VCARD")) {
            return Err(Defect::NoBegin);
        }
        if lines.iter().filter(|l| l.is("BEGIN", "VCARD")).count() > 1 {
            return Err(Defect::MultipleCards);
        }
        if !content.next_back().is_some_and(|l| l.is("END", "VCARD")) {
            return Err(Defect::NoEnd);
        }
        Ok(Self { lines })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.lines
            .iter()
            .flat_map(ContentLine::raw)
            .copied()
            .collect()
    }

    pub fn lines(&self) -> &[ContentLine] {
        &self.lines
    }

    pub fn get(&self, field: Field) -> String {
        let (name, component) = field.property();
        let value = self.values(name).next().unwrap_or_default();
        match component {
            None => line::unescape(value),
            Some(i) => line::split_unescaped(value, ';')
                .get(i)
                .map(|c| line::unescape(c))
                .unwrap_or_default(),
        }
    }

    pub fn set(&mut self, field: Field, value: &str) {
        if self.get(field) == value {
            return;
        }
        let (name, component) = field.property();
        let index = self
            .lines
            .iter()
            .position(|l| l.name().eq_ignore_ascii_case(name));
        let (raw, is_empty) = match component {
            None => (line::escape(value), value.is_empty()),
            Some(i) => {
                let current = index.map_or("", |index| self.lines[index].value());
                let mut parts: Vec<String> = line::split_unescaped(current, ';')
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
                let len = if name == "N" { 5 } else { i + 1 };
                if parts.len() < len {
                    parts.resize(len, String::new());
                }
                parts[i] = line::escape(value);
                let is_empty = parts.iter().all(String::is_empty);
                (parts.join(";"), is_empty)
            }
        };
        let is_removed =
            is_empty && matches!(field, Field::Company | Field::Department | Field::Note);
        match index {
            Some(index) if is_removed => {
                self.lines.remove(index);
            }
            Some(index) => self.lines[index] = self.lines[index].with_value(&raw),
            None => self.insert(name, &[], &raw),
        }
    }

    pub fn entries(&self, kind: Kind) -> Vec<Labeled<Vec<String>>> {
        self.labeled(kind.property(), |value| vec![line::unescape(value)])
    }

    pub fn update(&mut self, kind: Kind, n: usize, value: &[String], label: Option<&str>) {
        let index = self.position(kind, n);
        let property = &self.lines[index];
        let is_relabeled = self.label(property).as_deref() != label;
        let raw = match line::unescape(property.value()) == value[0] {
            true => property.value().to_owned(),
            false => line::escape(&value[0]),
        };
        if !is_relabeled && raw == property.value() {
            return;
        }
        let params = match is_relabeled {
            true => label::retype(property.params(), label),
            false => property.params().to_vec(),
        };
        let group = property.group().map(str::to_owned);
        self.lines[index] = property.with(&params, &raw);
        if is_relabeled && let Some(group) = group {
            self.lines.retain(|l| {
                !(l.name().eq_ignore_ascii_case("X-ABLabel")
                    && l.group().is_some_and(|g| g.eq_ignore_ascii_case(&group)))
            });
        }
    }

    pub fn add(&mut self, kind: Kind, value: &[String], label: Option<&str>) {
        let params = label::retype(&[], label);
        self.insert(kind.property(), &params, &line::escape(&value[0]));
    }

    pub fn remove(&mut self, kind: Kind, n: usize) {
        let index = self.position(kind, n);
        match self.lines[index].group().map(str::to_owned) {
            Some(group) => self
                .lines
                .retain(|l| !l.group().is_some_and(|g| g.eq_ignore_ascii_case(&group))),
            None => {
                self.lines.remove(index);
            }
        }
    }

    fn position(&self, kind: Kind, n: usize) -> usize {
        self.lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.name().eq_ignore_ascii_case(kind.property()))
            .nth(n)
            .map(|(i, _)| i)
            .expect("value index comes from this card's entries")
    }

    fn insert(&mut self, name: &str, params: &[String], raw: &str) {
        let end = self
            .lines
            .iter()
            .rposition(|l| l.is("END", "VCARD"))
            .expect("parsed card ends with END:VCARD");
        let eol = self
            .lines
            .iter()
            .find(|l| l.is("BEGIN", "VCARD"))
            .expect("parsed card starts with BEGIN:VCARD")
            .eol();
        self.lines
            .insert(end, ContentLine::new(None, name, params, raw, eol));
    }

    pub fn display_name(&self) -> String {
        self.get(Field::DisplayName)
    }

    pub fn structured_name(&self) -> (String, String) {
        (self.get(Field::Family), self.get(Field::Given))
    }

    pub fn phones(&self) -> Vec<Labeled<String>> {
        self.labeled("TEL", line::unescape)
    }

    pub fn emails(&self) -> Vec<Labeled<String>> {
        self.labeled("EMAIL", line::unescape)
    }

    pub fn organization(&self) -> Option<Organization> {
        let value = self.values("ORG").next()?;
        let mut components = line::split_unescaped(value, ';')
            .into_iter()
            .map(line::unescape);
        Some(Organization {
            company: components.next().unwrap_or_default(),
            department: components.next().unwrap_or_default(),
        })
    }

    pub fn note(&self) -> Option<String> {
        self.values("NOTE").next().map(line::unescape)
    }

    pub fn urls(&self) -> Vec<String> {
        self.values("URL").map(line::unescape).collect()
    }

    pub fn addresses(&self) -> Vec<Labeled<Address>> {
        self.labeled("ADR", Address::parse)
    }

    pub fn birthday(&self) -> Option<Birthday> {
        self.properties("BDAY")
            .next()
            .map(|l| Birthday::parse(l.value(), l.params()))
    }

    fn labeled<T>(&self, name: &str, read: impl Fn(&str) -> T) -> Vec<Labeled<T>> {
        self.properties(name)
            .map(|l| Labeled {
                label: self.label(l),
                value: read(l.value()),
            })
            .collect()
    }

    fn label(&self, property: &ContentLine) -> Option<String> {
        property
            .group()
            .and_then(|group| {
                self.lines.iter().find(|l| {
                    l.name().eq_ignore_ascii_case("X-ABLabel")
                        && l.group().is_some_and(|g| g.eq_ignore_ascii_case(group))
                })
            })
            .map(|l| label::decode(&line::unescape(l.value())))
            .or_else(|| label::from_types(property.params()))
    }

    fn values(&self, name: &str) -> impl Iterator<Item = &str> {
        self.properties(name).map(ContentLine::value)
    }

    fn properties(&self, name: &str) -> impl Iterator<Item = &ContentLine> {
        self.lines
            .iter()
            .filter(move |l| l.name().eq_ignore_ascii_case(name))
    }
}

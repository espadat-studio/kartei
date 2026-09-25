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

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let place = [&self.postal_code, &self.city]
            .into_iter()
            .filter(|part| !part.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        let parts = [self.street.as_str(), &place, &self.region, &self.country];
        let parts: Vec<&str> = parts.into_iter().filter(|p| !p.is_empty()).collect();
        f.write_str(&parts.join("\n"))
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
    Address,
}

impl Kind {
    fn property(self) -> &'static str {
        match self {
            Self::Phone => "TEL",
            Self::Email => "EMAIL",
            Self::Address => "ADR",
        }
    }

    fn decode(self, raw: &str) -> Vec<String> {
        match self {
            Self::Address => {
                let parts = line::split_unescaped(raw, ';');
                ADR_COMPONENTS
                    .map(|i| parts.get(i).map(|p| line::unescape(p)).unwrap_or_default())
                    .into()
            }
            _ => vec![line::unescape(raw)],
        }
    }

    fn encode(self, raw: &str, value: &[String]) -> String {
        match self {
            Self::Address => {
                let updates = ADR_COMPONENTS
                    .into_iter()
                    .zip(value.iter().map(String::as_str));
                replace_components(raw, 7, updates)
            }
            _ if line::unescape(raw) == value[0] => raw.to_owned(),
            _ => line::escape(&value[0]),
        }
    }
}

const ADR_COMPONENTS: [usize; 5] = [2, 5, 3, 4, 6];

fn replace_components<'a>(
    raw: &str,
    len: usize,
    updates: impl IntoIterator<Item = (usize, &'a str)>,
) -> String {
    let mut parts: Vec<String> = line::split_unescaped(raw, ';')
        .into_iter()
        .map(str::to_owned)
        .collect();
    for (i, value) in updates {
        if parts.get(i).map(|p| line::unescape(p)).unwrap_or_default() == value {
            continue;
        }
        if parts.len() < len {
            parts.resize(len, String::new());
        }
        parts[i] = line::escape(value);
    }
    parts.join(";")
}

pub fn derived_display_name(part: impl Fn(Field) -> String) -> String {
    let name = [
        Field::Prefixes,
        Field::Given,
        Field::Additional,
        Field::Family,
        Field::Suffixes,
    ]
    .map(&part)
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    match name.is_empty() {
        true => part(Field::Company),
        false => name,
    }
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
    pub fn new(uid: &str) -> Self {
        let bytes =
            format!("BEGIN:VCARD\r\nVERSION:3.0\r\nUID:{uid}\r\nN:;;;;\r\nFN:\r\nEND:VCARD\r\n");
        Self::parse(bytes.as_bytes()).expect("template is a valid card")
    }

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
                let len = if name == "N" { 5 } else { i + 1 };
                let raw = replace_components(current, len, [(i, value)]);
                let is_empty = line::split_unescaped(&raw, ';')
                    .iter()
                    .all(|p| p.is_empty());
                (raw, is_empty)
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
        self.labeled(kind.property(), |value| kind.decode(value))
    }

    pub fn update(&mut self, kind: Kind, n: usize, value: &[String], label: Option<&str>) {
        let index = self.position(kind, n);
        let property = &self.lines[index];
        let is_relabeled = self.label(property).as_deref() != label;
        let raw = kind.encode(property.value(), value);
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
        self.insert(kind.property(), &params, &kind.encode("", value));
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

    pub fn set_birthday(&mut self, birthday: Option<&Birthday>) {
        if self.birthday().as_ref() == birthday {
            return;
        }
        let index = self
            .lines
            .iter()
            .position(|l| l.name().eq_ignore_ascii_case("BDAY"));
        match (birthday, index) {
            (None, Some(index)) => {
                self.lines.remove(index);
            }
            (Some(&Birthday::Date { year, month, day }), _) => {
                let is_v4 = self.values("VERSION").next() == Some("4.0");
                let existing = index.map(|i| (self.lines[i].params(), self.lines[i].value()));
                let (params, value) = bday::write(year, month, day, existing, is_v4);
                match index {
                    Some(i) => self.lines[i] = self.lines[i].with(&params, &value),
                    None => self.insert("BDAY", &params, &value),
                }
            }
            (None, None) | (Some(Birthday::Invalid(_)), _) => {}
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

mod bday;
mod label;
mod line;

pub use bday::Birthday;
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
pub struct Card {
    lines: Vec<ContentLine>,
}

impl Card {
    pub fn parse(bytes: &[u8]) -> Self {
        Self {
            lines: line::split(bytes),
        }
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

    pub fn display_name(&self) -> String {
        self.values("FN")
            .next()
            .map(line::unescape)
            .unwrap_or_default()
    }

    pub fn structured_name(&self) -> (String, String) {
        let value = self.values("N").next().unwrap_or_default();
        let mut components = line::split_unescaped(value, ';')
            .into_iter()
            .map(line::unescape);
        (
            components.next().unwrap_or_default(),
            components.next().unwrap_or_default(),
        )
    }

    pub fn phones(&self) -> Vec<Labeled<String>> {
        self.labeled("TEL", line::unescape)
    }

    pub fn emails(&self) -> Vec<Labeled<String>> {
        self.labeled("EMAIL", line::unescape)
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

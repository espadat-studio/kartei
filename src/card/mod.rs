mod line;

pub use line::ContentLine;

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

    pub fn phones(&self) -> Vec<String> {
        self.values("TEL").map(line::unescape).collect()
    }

    pub fn emails(&self) -> Vec<String> {
        self.values("EMAIL").map(line::unescape).collect()
    }

    fn values(&self, name: &str) -> impl Iterator<Item = &str> {
        self.lines
            .iter()
            .filter(move |l| l.name().eq_ignore_ascii_case(name))
            .map(ContentLine::value)
    }
}

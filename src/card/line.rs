#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentLine {
    raw: Vec<u8>,
    group: Option<String>,
    name: String,
    params: Vec<String>,
    value: String,
}

impl ContentLine {
    fn from_raw(raw: Vec<u8>) -> Option<Self> {
        let text = String::from_utf8(unfold(&raw)).ok()?;
        let (head, params, value) = split_head(&text);
        let (group, name) = match head.split_once('.') {
            Some((group, name)) => (Some(group.to_owned()), name.to_owned()),
            None => (None, head.to_owned()),
        };
        Some(Self {
            group,
            name,
            params: params.into_iter().map(str::to_owned).collect(),
            value: value.to_owned(),
            raw,
        })
    }

    pub(crate) fn new(name: &str, value: &str, eol: &str) -> Self {
        Self::build(&format!("{name}:{value}"), eol)
    }

    pub(crate) fn with_value(&self, value: &str) -> Self {
        let mut logical = self
            .group
            .as_ref()
            .map_or_else(String::new, |group| format!("{group}."));
        logical.push_str(&self.name);
        for param in &self.params {
            logical.push(';');
            logical.push_str(param);
        }
        logical.push(':');
        logical.push_str(value);
        Self::build(&logical, self.eol())
    }

    fn build(logical: &str, eol: &str) -> Self {
        Self::from_raw(fold(logical, eol)).expect("folded from a str")
    }

    pub(crate) fn eol(&self) -> &'static str {
        if self.raw.ends_with(b"\r\n") {
            "\r\n"
        } else if self.raw.ends_with(b"\n") {
            "\n"
        } else {
            ""
        }
    }

    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[String] {
        &self.params
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub(crate) fn is(&self, name: &str, value: &str) -> bool {
        self.group.is_none()
            && self.params.is_empty()
            && self.name.eq_ignore_ascii_case(name)
            && self.value.eq_ignore_ascii_case(value)
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.raw.trim_ascii().is_empty()
    }
}

pub(crate) fn split(bytes: &[u8]) -> Option<Vec<ContentLine>> {
    let mut lines: Vec<Vec<u8>> = Vec::new();
    for physical in bytes.split_inclusive(|&b| b == b'\n') {
        match lines.last_mut() {
            Some(raw) if matches!(physical.first(), Some(b' ' | b'\t')) => {
                raw.extend_from_slice(physical)
            }
            _ => lines.push(physical.to_vec()),
        }
    }
    lines.into_iter().map(ContentLine::from_raw).collect()
}

fn fold(logical: &str, eol: &str) -> Vec<u8> {
    let mut out = String::with_capacity(logical.len() + eol.len());
    let mut width = 0;
    for c in logical.chars() {
        if width + c.len_utf8() > 75 {
            out.push_str(eol);
            out.push(' ');
            width = 1;
        }
        out.push(c);
        width += c.len_utf8();
    }
    out.push_str(eol);
    out.into_bytes()
}

fn unfold(raw: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    for (i, physical) in raw.split_inclusive(|&b| b == b'\n').enumerate() {
        let content = physical.strip_suffix(b"\n").unwrap_or(physical);
        let content = content.strip_suffix(b"\r").unwrap_or(content);
        out.extend_from_slice(if i == 0 { content } else { &content[1..] });
    }
    out
}

fn split_head(text: &str) -> (&str, Vec<&str>, &str) {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    for (i, c) in text.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ';' if !in_quotes => {
                segments.push(&text[start..i]);
                start = i + 1;
            }
            ':' if !in_quotes => {
                segments.push(&text[start..i]);
                return (segments[0], segments.split_off(1), &text[i + 1..]);
            }
            _ => {}
        }
    }
    segments.push(&text[start..]);
    (segments[0], segments.split_off(1), "")
}

pub(crate) fn split_unescaped(value: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    for (i, c) in value.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            c if c == separator => {
                parts.push(&value[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&value[start..]);
    parts
}

pub(crate) fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(escaped) => out.push(escaped),
            None => out.push('\\'),
        }
    }
    out
}

pub(crate) fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' | ',' | ';' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

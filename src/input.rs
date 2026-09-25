use ratatui::text::Line;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    text: String,
    cursor: usize,
}

impl Input {
    pub fn new(text: String) -> Self {
        Self {
            cursor: text.len(),
            text,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn replace(&mut self, text: String) {
        *self = Self::new(text);
    }

    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.left();
            self.text.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    pub fn left(&mut self) {
        if let Some(c) = self.text[..self.cursor].chars().next_back() {
            self.cursor -= c.len_utf8();
        }
    }

    pub fn right(&mut self) {
        if let Some(c) = self.text[self.cursor..].chars().next() {
            self.cursor += c.len_utf8();
        }
    }

    pub fn cursor(&self) -> (u16, u16) {
        let before = &self.text[..self.cursor];
        let row = before.matches('\n').count();
        let line = before.rsplit('\n').next().unwrap_or_default();
        (row as u16, Line::from(line).width() as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::Input;

    #[test]
    fn typing_inserts_at_the_cursor_and_backspace_deletes_before_it() {
        let mut input = Input::new("ac".into());
        input.left();
        input.insert('b');
        assert_eq!(input.text(), "abc");
        input.backspace();
        input.backspace();
        assert_eq!(input.text(), "c");
        input.backspace();
        assert_eq!(input.text(), "c");
        input.delete();
        assert_eq!(input.text(), "");
    }

    #[test]
    fn cursor_moves_by_character_and_counts_display_cells() {
        let mut input = Input::new("日本é".into());
        assert_eq!(input.cursor(), (0, 5));
        input.left();
        assert_eq!(input.cursor(), (0, 4));
        input.left();
        input.backspace();
        assert_eq!(input.text(), "本é");
        assert_eq!(input.cursor(), (0, 0));
        input.left();
        input.right();
        input.right();
        input.right();
        assert_eq!(input.cursor(), (0, 3));
        input.delete();
        assert_eq!(input.text(), "本é");
    }

    #[test]
    fn newlines_move_the_cursor_to_the_next_row() {
        let mut input = Input::new("ab".into());
        input.insert('\n');
        assert_eq!(input.cursor(), (1, 0));
        input.insert('ü');
        assert_eq!(input.cursor(), (1, 1));
        assert_eq!(input.text(), "ab\nü");
    }
}

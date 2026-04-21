use unicode_segmentation::UnicodeSegmentation;

/// Unicode-aware input buffer with cursor tracking.
#[derive(Clone)]
pub struct InputBuffer {
    text: String,
    cursor: usize, // cursor position in grapheme clusters
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    /// Insert a character at the cursor.
    pub fn insert(&mut self, ch: char) {
        let idx = self.cursor_to_byte();
        self.text.insert(idx, ch);
        self.cursor += 1;
    }

    /// Delete the character before the cursor (backspace).
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let byte_idx = self.cursor_to_byte();
        // Find the start of the previous grapheme cluster.
        let prev = self.text[..byte_idx]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(byte_idx.saturating_sub(1));
        self.text.drain(prev..byte_idx);
        self.cursor -= 1;
    }

    /// Delete the character at the cursor.
    pub fn delete(&mut self) {
        let byte_idx = self.cursor_to_byte();
        if byte_idx >= self.text.len() {
            return;
        }
        // Find the end of the grapheme cluster at cursor.
        let cluster = self.text[byte_idx..]
            .graphemes(true)
            .next()
            .map(|g| g.len())
            .unwrap_or(1);
        let end = std::cmp::min(byte_idx + cluster, self.text.len());
        self.text.drain(byte_idx..end);
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.text.graphemes(true).count() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.text.graphemes(true).count();
    }

    /// Delete from cursor to the beginning of the line (Ctrl+U).
    pub fn delete_to_start(&mut self) {
        let byte_idx = self.cursor_to_byte();
        self.text.drain(..byte_idx);
        self.cursor = 0;
    }

    /// Delete from cursor to the end of the line (Ctrl+K).
    pub fn delete_to_end(&mut self) {
        let byte_idx = self.cursor_to_byte();
        self.text.drain(byte_idx..);
    }

    /// Delete the previous word (Ctrl+W).
    pub fn delete_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let byte_idx = self.cursor_to_byte();
        let prefix = self.text[..byte_idx].to_string();
        // Find the start of the word by scanning backwards through graphemes.
        let mut graphemes = prefix.graphemes(true).collect::<Vec<_>>();
        // Skip trailing whitespace.
        while let Some(g) = graphemes.last() {
            if g.chars().any(|c| !c.is_whitespace()) {
                break;
            }
            graphemes.pop();
        }
        // Pop the word characters.
        while let Some(g) = graphemes.last() {
            if g.chars().any(|c| c.is_whitespace()) {
                break;
            }
            graphemes.pop();
        }
        let new_byte = graphemes.iter().map(|g| g.len()).sum::<usize>();
        self.text.drain(new_byte..byte_idx);
        self.cursor = graphemes.len();
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Byte index of the cursor.
    pub fn cursor_byte(&self) -> usize {
        self.cursor_to_byte()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    fn cursor_to_byte(&self) -> usize {
        self.text
            .grapheme_indices(true)
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut buf = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        assert_eq!(buf.text(), "abc");
        assert_eq!(buf.cursor, 3);
    }

    #[test]
    fn test_backspace() {
        let mut buf = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.insert('c');
        buf.backspace();
        assert_eq!(buf.text(), "ab");
        assert_eq!(buf.cursor, 2);
    }

    #[test]
    fn test_backspace_at_start() {
        let mut buf = InputBuffer::new();
        buf.insert('a');
        buf.move_home();
        buf.backspace();
        assert_eq!(buf.text(), "a");
        assert_eq!(buf.cursor, 0);
    }

    #[test]
    fn test_delete_to_start() {
        let mut buf = InputBuffer::new();
        buf.insert('h');
        buf.insert('e');
        buf.insert('l');
        buf.insert('l');
        buf.insert('o');
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.delete_to_start();
        assert_eq!(buf.text(), "llo");
        assert_eq!(buf.cursor, 0);
    }

    #[test]
    fn test_delete_to_end() {
        let mut buf = InputBuffer::new();
        buf.insert('h');
        buf.insert('e');
        buf.insert('l');
        buf.insert('l');
        buf.insert('o');
        buf.move_home();
        buf.move_right();
        buf.move_right();
        buf.delete_to_end();
        assert_eq!(buf.text(), "he");
        assert_eq!(buf.cursor, 2);
    }

    #[test]
    fn test_delete_word() {
        let mut buf = InputBuffer::new();
        for ch in "hello world".chars() {
            buf.insert(ch);
        }
        buf.delete_word();
        assert_eq!(buf.text(), "hello ");
        assert_eq!(buf.cursor, 6);
        buf.delete_word();
        assert_eq!(buf.text(), "");
        assert_eq!(buf.cursor, 0);
    }

    #[test]
    fn test_unicode_grapheme() {
        let mut buf = InputBuffer::new();
        buf.insert('👋');
        buf.insert('a');
        assert_eq!(buf.text(), "👋a");
        assert_eq!(buf.cursor, 2);
        buf.backspace();
        assert_eq!(buf.text(), "👋");
        assert_eq!(buf.cursor, 1);
    }

    #[test]
    fn test_delete_word_with_unicode() {
        let mut buf = InputBuffer::new();
        for ch in "hello 世界".chars() {
            buf.insert(ch);
        }
        buf.delete_word();
        assert_eq!(buf.text(), "hello ");
    }

    #[test]
    fn test_clear() {
        let mut buf = InputBuffer::new();
        buf.insert('a');
        buf.insert('b');
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.cursor, 0);
    }

    #[test]
    fn test_move_home_end() {
        let mut buf = InputBuffer::new();
        for ch in "test".chars() {
            buf.insert(ch);
        }
        buf.move_home();
        assert_eq!(buf.cursor, 0);
        buf.move_end();
        assert_eq!(buf.cursor, 4);
    }
}

use regex::Regex;

/// Search state over a fixed snapshot of lines.
pub struct SearchState {
    query: Option<Regex>,
    /// (line_index, line_content)
    matches: Vec<(usize, String)>,
    current: Option<usize>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: None,
            matches: Vec::new(),
            current: None,
        }
    }

    pub fn set_query(&mut self, query: &str) {
        if query.is_empty() {
            self.clear();
            return;
        }
        self.query = Regex::new(&regex::escape(query)).ok();
        self.current = None;
        self.matches.clear();
    }

    /// Find matches in the given lines.
    pub fn execute(&mut self, lines: &[String]) {
        if self.query.is_none() {
            self.matches.clear();
            self.current = None;
            return;
        }
        let re = self.query.as_ref().unwrap();
        self.matches = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, line)| (i, line.clone()))
            .collect();
        self.current = if self.matches.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    pub fn next_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        let idx = match self.current {
            None => 0,
            Some(i) => (i + 1) % self.matches.len(),
        };
        self.current = Some(idx);
        Some(self.matches[idx].0)
    }

    pub fn prev_match(&mut self) -> Option<usize> {
        if self.matches.is_empty() {
            return None;
        }
        let idx = match self.current {
            None => self.matches.len() - 1,
            Some(i) => (i + self.matches.len() - 1) % self.matches.len(),
        };
        self.current = Some(idx);
        Some(self.matches[idx].0)
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_ref().map(|re| re.as_str())
    }

    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    pub fn current_match(&self) -> Option<(usize, &str)> {
        self.current
            .map(|i| (self.matches[i].0, self.matches[i].1.as_str()))
    }

    pub fn active(&self) -> bool {
        self.query.is_some() && !self.matches.is_empty()
    }

    pub fn clear(&mut self) {
        self.query = None;
        self.matches.clear();
        self.current = None;
    }

    /// Check if `line_idx` is a match.
    pub fn is_match(&self, line_idx: usize) -> bool {
        self.matches.iter().any(|(idx, _)| *idx == line_idx)
    }

    /// Check if `line_idx` is the currently focused match.
    pub fn is_current_match(&self, line_idx: usize) -> bool {
        self.current
            .map(|i| self.matches[i].0 == line_idx)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_search() {
        let mut search = SearchState::new();
        search.set_query("hello");
        search.execute(&[
            "hello world".to_string(),
            "goodbye".to_string(),
            "hello again".to_string(),
        ]);
        assert_eq!(search.match_count(), 2);
    }

    #[test]
    fn test_no_match() {
        let mut search = SearchState::new();
        search.set_query("xyz");
        search.execute(&["hello world".to_string()]);
        assert_eq!(search.match_count(), 0);
        assert!(!search.active());
    }

    #[test]
    fn test_empty_query() {
        let mut search = SearchState::new();
        search.set_query("");
        assert!(search.query().is_none());
    }

    #[test]
    fn test_next_prev_match() {
        let mut search = SearchState::new();
        search.set_query("foo");
        search.execute(&[
            "foo bar".to_string(),
            "baz".to_string(),
            "foo baz".to_string(),
        ]);
        // After execute, current is at first match. next_match advances.
        assert_eq!(search.next_match(), Some(2)); // advances to second match
        assert_eq!(search.next_match(), Some(0)); // wraps around
        assert_eq!(search.prev_match(), Some(2)); // goes back
    }

    #[test]
    fn test_is_match() {
        let mut search = SearchState::new();
        search.set_query("test");
        search.execute(&[
            "test line".to_string(),
            "no match".to_string(),
            "another test".to_string(),
        ]);
        assert!(search.is_match(0));
        assert!(!search.is_match(1));
        assert!(search.is_match(2));
    }

    #[test]
    fn test_is_current_match() {
        let mut search = SearchState::new();
        search.set_query("foo");
        search.execute(&["foo bar".to_string(), "foo baz".to_string()]);
        // After execute, current is at first match (line 0).
        assert!(search.is_current_match(0));
        assert!(!search.is_current_match(1));
        search.next_match(); // advances to second match (line 1)
        assert!(!search.is_current_match(0));
        assert!(search.is_current_match(1));
    }

    #[test]
    fn test_clear() {
        let mut search = SearchState::new();
        search.set_query("foo");
        search.execute(&["foo bar".to_string()]);
        search.clear();
        assert!(!search.active());
        assert_eq!(search.match_count(), 0);
    }

    #[test]
    fn test_special_chars_escaped() {
        let mut search = SearchState::new();
        search.set_query("foo.bar");
        search.execute(&[
            "fooXbar".to_string(),
            "foo.bar".to_string(),
        ]);
        // Should only match the literal "foo.bar", not the regex pattern.
        assert_eq!(search.match_count(), 1);
    }
}

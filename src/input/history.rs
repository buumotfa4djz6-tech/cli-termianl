use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Serializable history data.
#[derive(Debug, Serialize, Deserialize)]
struct HistoryFile {
    commands: Vec<String>,
}

/// Command history with search and persistence.
pub struct HistoryManager {
    entries: Vec<String>,
    max_size: usize,
    persist_path: Option<PathBuf>,
}

impl HistoryManager {
    pub fn new(max_size: usize, persist_path: Option<PathBuf>) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            persist_path,
        }
    }

    pub fn load(&mut self) -> Result<()> {
        let Some(path) = &self.persist_path else {
            return Ok(());
        };
        if !path.exists() {
            return Ok(());
        }
        let data: HistoryFile = serde_json::from_str(&fs::read_to_string(path)?)?;
        self.entries = data.commands;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = &self.persist_path else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = HistoryFile {
            commands: self.entries.clone(),
        };
        fs::write(path, serde_json::to_string_pretty(&data)?)?;
        Ok(())
    }

    pub fn add(&mut self, command: &str) {
        if command.trim().is_empty() {
            return;
        }
        self.entries.push(command.to_string());
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
        }
    }

    /// Search history, returning entries containing `query` (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&str> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .filter(|cmd| cmd.to_lowercase().contains(&q))
            .map(|s| s.as_str())
            .collect()
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn last(&self) -> Option<&str> {
        self.entries.last().map(|s| s.as_str())
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_manager() -> HistoryManager {
        HistoryManager::new(10, None)
    }

    #[test]
    fn test_add_and_search() {
        let mut hm = new_test_manager();
        hm.add("ls -la");
        hm.add("cargo build");
        hm.add("cargo test");
        let results = hm.search("cargo");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut hm = new_test_manager();
        hm.add("cargo build");
        let results = hm.search("CARGO");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "cargo build");
    }

    #[test]
    fn test_search_empty_query() {
        let mut hm = new_test_manager();
        hm.add("ls");
        let results = hm.search("");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_no_match() {
        let mut hm = new_test_manager();
        hm.add("cargo build");
        let results = hm.search("python");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_max_size() {
        let mut hm = HistoryManager::new(3, None);
        hm.add("cmd1");
        hm.add("cmd2");
        hm.add("cmd3");
        hm.add("cmd4");
        assert_eq!(hm.entries().len(), 3);
        assert_eq!(hm.entries()[0], "cmd2");
    }

    #[test]
    fn test_add_empty_ignored() {
        let mut hm = new_test_manager();
        hm.add("");
        hm.add("   ");
        assert!(hm.entries().is_empty());
    }

    #[test]
    fn test_last() {
        let mut hm = new_test_manager();
        assert!(hm.last().is_none());
        hm.add("first");
        hm.add("second");
        assert_eq!(hm.last(), Some("second"));
    }

    #[test]
    fn test_clear() {
        let mut hm = new_test_manager();
        hm.add("cmd1");
        hm.add("cmd2");
        hm.clear();
        assert!(hm.entries().is_empty());
    }
}

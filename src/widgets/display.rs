use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;

use crate::output::collapse::CollapseManager;

/// Styled output line.
#[derive(Clone)]
pub struct OutputLine {
    pub content: String,
    pub is_command: bool,
    pub timestamp: Option<String>,
}

impl OutputLine {
    pub fn new(content: String) -> Self {
        Self {
            content,
            is_command: false,
            timestamp: None,
        }
    }

    pub fn command(content: String, timestamp: Option<String>) -> Self {
        Self {
            content,
            is_command: true,
            timestamp,
        }
    }
}

/// Output display widget.
pub struct DisplayWidget {
    lines: Vec<OutputLine>,
    timestamp_format: String,
    timestamp_enabled: bool,
    collapse_manager: CollapseManager,
    /// Start line index of the current output block (for auto-collapse).
    block_start: usize,
}

impl DisplayWidget {
    pub fn new(timestamp_format: String, timestamp_enabled: bool) -> Self {
        Self {
            lines: Vec::new(),
            timestamp_format,
            timestamp_enabled,
            collapse_manager: CollapseManager::new(),
            block_start: 0,
        }
    }

    /// Add a plain output line.
    pub fn add_line(&mut self, content: String) {
        let ts = if self.timestamp_enabled {
            Some(Local::now().format(&self.timestamp_format).to_string())
        } else {
            None
        };
        self.lines.push(OutputLine {
            content,
            is_command: false,
            timestamp: ts,
        });
    }

    /// Add a command echo line. Creates a collapsible region for the previous
    /// output block before adding the new command.
    pub fn add_command(&mut self, content: String) {
        let end = self.lines.len();
        if end > self.block_start + 1 {
            // There are output lines between the previous command and this one.
            let header: String = self.lines[self.block_start]
                .content
                .chars()
                .take(40)
                .collect();
            self.collapse_manager
                .add_region(self.block_start, end, &header);
        }
        self.lines.push(OutputLine::command(content, None));
        self.block_start = self.lines.len();
    }

    /// Add a system message.
    pub fn add_message(&mut self, msg: &str) {
        self.add_line(format!("[{msg}]"));
    }

    pub fn lines(&self) -> &[OutputLine] {
        &self.lines
    }

    /// Raw text of all lines (for export and search).
    pub fn raw_lines(&self) -> Vec<String> {
        self.lines.iter().map(|l| l.content.clone()).collect()
    }

    /// Line count.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.collapse_manager = CollapseManager::new();
        self.block_start = 0;
    }

    pub fn collapse_manager(&mut self) -> &mut CollapseManager {
        &mut self.collapse_manager
    }

    /// Update timestamp settings without losing existing content.
    pub fn update_timestamp_settings(&mut self, format: String, enabled: bool) {
        self.timestamp_format = format;
        self.timestamp_enabled = enabled;
    }

    /// Export output to a timestamped file.
    pub fn export(&self) -> Result<PathBuf> {
        let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        dir.push("cli-terminal");
        dir.push("exports");
        fs::create_dir_all(&dir)?;

        let ts = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("output_{ts}.txt");
        let path = dir.join(filename);

        let mut content = String::new();
        for line in &self.lines {
            content.push_str(&line.content);
            content.push('\n');
        }
        fs::write(&path, content)?;
        Ok(path)
    }
}

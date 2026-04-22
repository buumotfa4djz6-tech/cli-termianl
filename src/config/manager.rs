use std::collections::HashMap;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use super::defaults::*;

/// Parsed and validated application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub commands: HashMap<String, String>,
    pub macros: HashMap<String, String>,
    pub templates: Vec<TemplateConfig>,
    pub highlights: Vec<HighlightRule>,
    pub timestamp_format: String,
    pub timestamp_enabled: bool,
    pub history_max_size: usize,
    pub history_persist: bool,
}

/// Raw deserializable config for loading from YAML.
#[derive(Debug, Deserialize, Serialize)]
pub struct RawConfig {
    #[serde(default = "default_commands")]
    pub commands: HashMap<String, String>,

    #[serde(default = "default_macros")]
    pub macros: HashMap<String, String>,

    #[serde(default)]
    pub templates: Vec<TemplateConfig>,

    #[serde(default = "default_highlights")]
    pub highlights: Vec<HighlightRule>,

    #[serde(default = "default_timestamp_cfg")]
    pub timestamp: TimestampConfig,

    #[serde(default = "default_history_cfg")]
    pub history: HistoryConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateConfig {
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub params: Vec<TemplateParam>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateParam {
    pub name: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HighlightRule {
    pub pattern: String,
    #[serde(
        deserialize_with = "deserialize_color_opt",
        serialize_with = "serialize_color_opt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fg: Option<Color>,
    #[serde(
        deserialize_with = "deserialize_color_opt",
        serialize_with = "serialize_color_opt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub bg: Option<Color>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub underline: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimestampConfig {
    #[serde(default = "default_timestamp_enabled")]
    pub enabled: bool,
    #[serde(default = "default_timestamp_format")]
    pub format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryConfig {
    #[serde(default = "default_history_max_size")]
    pub max_size: usize,
    #[serde(default = "default_history_persist")]
    pub persist: bool,
}

// Serde default functions — keep in this module since #[serde(default = "...")] resolves locally.
fn default_timestamp_enabled() -> bool {
    true
}

fn default_timestamp_format() -> String {
    "[%H:%M:%S]".to_string()
}

fn default_history_max_size() -> usize {
    1000
}

fn default_history_persist() -> bool {
    true
}

// ─── Color deserialization ───────────────────────────────────────────

fn deserialize_color_opt<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    s.map(|s| {
        parse_color(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {s}")))
    })
    .transpose()
}

fn parse_color(s: &str) -> Option<Color> {
    match s.to_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" | "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "dark_gray" => Some(Color::DarkGray),
        "white" => Some(Color::White),
        _ => None,
    }
}

fn serialize_color_opt<S>(color: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match color {
        Some(c) => serializer.serialize_str(&color_to_str(c)),
        None => serializer.serialize_none(),
    }
}

fn color_to_str(color: &Color) -> &'static str {
    match color {
        Color::Black => "black",
        Color::Red => "red",
        Color::Green => "green",
        Color::Yellow => "yellow",
        Color::Blue => "blue",
        Color::Magenta => "magenta",
        Color::Cyan => "cyan",
        Color::Gray => "gray",
        Color::DarkGray => "dark_gray",
        Color::White => "white",
        _ => "white",
    }
}

impl RawConfig {
    /// Merge `other` into `self`, with `other` fields taking priority.
    /// Only fields that are non-empty in `other` are overridden.
    pub fn merge_with(&mut self, other: Self) {
        if !other.commands.is_empty() {
            self.commands.extend(other.commands);
        }
        if !other.macros.is_empty() {
            self.macros.extend(other.macros);
        }
        if !other.templates.is_empty() {
            self.templates = other.templates;
        }
        if !other.highlights.is_empty() {
            self.highlights = other.highlights;
        }
        if other.timestamp.enabled || !other.timestamp.format.is_empty() {
            self.timestamp = other.timestamp;
        }
        if other.history.max_size > 0 {
            self.history = other.history;
        }
    }
}

impl AppConfig {
    pub fn from_raw(raw: RawConfig) -> Self {
        Self {
            commands: raw.commands,
            macros: raw.macros,
            templates: raw.templates,
            highlights: raw.highlights,
            timestamp_format: raw.timestamp.format,
            timestamp_enabled: raw.timestamp.enabled,
            history_max_size: raw.history.max_size,
            history_persist: raw.history.persist,
        }
    }
}

use std::collections::HashMap;

use super::manager::{HighlightRule, RawConfig};

// ─── Embedded default config (edit defaults.yaml, not Rust code) ─────

const DEFAULT_YAML: &str = include_str!("defaults.yaml");

/// Return the bundled default config as YAML string.
pub fn bundled_default_yaml() -> &'static str {
    DEFAULT_YAML
}

/// Parse the bundled default YAML into a RawConfig.
pub fn parse_default_config() -> RawConfig {
    serde_yaml::from_str(DEFAULT_YAML).expect("bundled defaults YAML should always parse")
}

/// Generate the default config YAML — just return the bundled text.
pub fn generate_default_config_yaml() -> String {
    DEFAULT_YAML.to_string()
}

// ─── Minimal Rust-only defaults (structural, no command data) ────────

pub fn default_commands() -> HashMap<String, String> {
    HashMap::new()
}

pub fn default_macros() -> HashMap<String, String> {
    HashMap::new()
}

pub fn default_highlights() -> Vec<HighlightRule> {
    vec![]
}

pub const fn default_timestamp_enabled() -> bool {
    true
}

pub fn default_timestamp_format() -> String {
    "[%H:%M:%S]".to_string()
}

pub const fn default_history_max_size() -> usize {
    1000
}

pub const fn default_history_persist() -> bool {
    true
}

pub fn default_timestamp_cfg() -> crate::config::manager::TimestampConfig {
    crate::config::manager::TimestampConfig {
        enabled: default_timestamp_enabled(),
        format: default_timestamp_format(),
    }
}

pub fn default_history_cfg() -> crate::config::manager::HistoryConfig {
    crate::config::manager::HistoryConfig {
        max_size: default_history_max_size(),
        persist: default_history_persist(),
    }
}

use regex::Regex;

use crate::config::manager::{TemplateConfig, TemplateParam};

/// A parsed command template.
pub struct Template {
    pub label: String,
    /// Raw command string with `{param}` placeholders.
    command: String,
    /// Ordered list of unique parameter names.
    pub params: Vec<TemplateParam>,
}

impl Template {
    /// Get the raw command string.
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn from_config(cfg: TemplateConfig) -> Self {
        // Deduplicate params by name, preserving order.
        let mut seen = std::collections::HashSet::new();
        let params: Vec<TemplateParam> = cfg
            .params
            .into_iter()
            .filter(|p| seen.insert(p.name.clone()))
            .collect();
        Self {
            label: cfg.label,
            command: cfg.command,
            params,
        }
    }

    /// Expand template with parameter values. Unmatched placeholders are left as-is.
    pub fn expand(&self, values: &[(String, String)]) -> String {
        let mut result = self.command.clone();
        for (name, value) in values {
            result = result.replace(&format!("{{{name}}}"), value);
        }
        result
    }

    /// Extract parameter names from the command string.
    pub fn extract_params(command: &str) -> Vec<String> {
        let re = Regex::new(r"\{(\w+)\}").unwrap();
        let mut seen = std::collections::HashSet::new();
        re.captures_iter(command)
            .filter_map(|cap| cap.get(1))
            .filter(|m| seen.insert(m.as_str().to_string()))
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

/// Manager for command templates.
pub struct TemplateManager {
    templates: Vec<Template>,
}

impl TemplateManager {
    pub fn new(configs: Vec<TemplateConfig>) -> Self {
        Self {
            templates: configs.into_iter().map(Template::from_config).collect(),
        }
    }

    pub fn templates(&self) -> &[Template] {
        &self.templates
    }

    pub fn by_label(&self, label: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.label == label)
    }

    pub fn update(&mut self, configs: Vec<TemplateConfig>) {
        self.templates = configs.into_iter().map(Template::from_config).collect();
    }
}

pub mod app;
pub mod config;
pub mod input;
pub mod output;
pub mod widgets;

#[cfg(test)]
mod test_defaults_yaml {
    use crate::config::defaults::{parse_default_config, generate_default_config_yaml};

    #[test]
    fn bundled_yaml_parses() {
        let cfg = parse_default_config();
        assert!(!cfg.templates.is_empty());
        assert!(!cfg.macros.is_empty());
        assert!(!cfg.commands.is_empty());
        assert!(!cfg.highlights.is_empty());
    }

    #[test]
    fn roundtrip_serialize() {
        let cfg = parse_default_config();
        let yaml = generate_default_config_yaml();
        let cfg2: crate::config::manager::RawConfig = serde_yaml::from_str(&yaml).expect("roundtrip");
        assert_eq!(cfg.templates.len(), cfg2.templates.len());
        assert_eq!(cfg.macros.len(), cfg2.macros.len());
    }

    #[test]
    fn yaml_contains_command_templates() {
        let yaml = generate_default_config_yaml();
        assert!(yaml.contains("mscontrol"));
        assert!(yaml.contains("get_rcm"));
        assert!(yaml.contains("inject"));
        assert!(yaml.contains("set_rcm"));
    }
}

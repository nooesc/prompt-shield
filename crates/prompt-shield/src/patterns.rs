use crate::config::{parse_config, Config};

const DEFAULT_CONFIG_TOML: &str = include_str!("../../../config/default.toml");

pub fn default_config() -> Config {
    parse_config(DEFAULT_CONFIG_TOML).expect("built-in default.toml must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::Action;

    #[test]
    fn default_config_loads() {
        let config = default_config();
        assert!(!config.patterns.is_empty());
    }

    #[test]
    fn default_config_has_all_categories() {
        let config = default_config();
        assert!(!config.patterns.instruction_override.is_empty());
        assert!(!config.patterns.role_playing.is_empty());
        assert!(!config.patterns.encoding_obfuscation.is_empty());
        assert!(!config.patterns.context_manipulation.is_empty());
    }

    #[test]
    fn default_severity_actions_are_set() {
        let config = default_config();
        assert_eq!(config.severity_actions.low, Action::Log);
        assert_eq!(config.severity_actions.medium, Action::Warn);
        assert_eq!(config.severity_actions.high, Action::Block);
    }

    #[test]
    fn default_config_pattern_counts() {
        let config = default_config();
        assert_eq!(config.patterns.instruction_override.len(), 23);
        assert!(config.patterns.role_playing.len() >= 23);
        assert!(config.patterns.encoding_obfuscation.len() >= 21);
        assert!(config.patterns.context_manipulation.len() >= 29);
    }
}

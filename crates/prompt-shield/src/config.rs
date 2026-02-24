use serde::Deserialize;

use crate::detection::{Action, Severity};
use crate::Category;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub severity_actions: SeverityActions,
    #[serde(default)]
    pub patterns: PatternSet,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeverityActions {
    #[serde(default = "default_low_action")]
    pub low: Action,
    #[serde(default = "default_medium_action")]
    pub medium: Action,
    #[serde(default = "default_high_action")]
    pub high: Action,
}

fn default_low_action() -> Action {
    Action::Log
}
fn default_medium_action() -> Action {
    Action::Warn
}
fn default_high_action() -> Action {
    Action::Block
}

impl Default for SeverityActions {
    fn default() -> Self {
        Self {
            low: default_low_action(),
            medium: default_medium_action(),
            high: default_high_action(),
        }
    }
}

impl SeverityActions {
    pub fn action_for(&self, severity: Severity) -> Action {
        match severity {
            Severity::Low => self.low,
            Severity::Medium => self.medium,
            Severity::High => self.high,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PatternSet {
    #[serde(default)]
    pub instruction_override: Vec<PatternEntry>,
    #[serde(default)]
    pub role_playing: Vec<PatternEntry>,
    #[serde(default)]
    pub encoding_obfuscation: Vec<PatternEntry>,
    #[serde(default)]
    pub context_manipulation: Vec<PatternEntry>,
}

impl PatternSet {
    pub fn iter_with_category(&self) -> impl Iterator<Item = (Category, &PatternEntry)> {
        self.instruction_override
            .iter()
            .map(|p| (Category::InstructionOverride, p))
            .chain(
                self.role_playing
                    .iter()
                    .map(|p| (Category::RolePlayingDan, p)),
            )
            .chain(
                self.encoding_obfuscation
                    .iter()
                    .map(|p| (Category::EncodingObfuscation, p)),
            )
            .chain(
                self.context_manipulation
                    .iter()
                    .map(|p| (Category::ContextManipulation, p)),
            )
    }

    pub fn is_empty(&self) -> bool {
        self.instruction_override.is_empty()
            && self.role_playing.is_empty()
            && self.encoding_obfuscation.is_empty()
            && self.context_manipulation.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PatternEntry {
    pub pattern: String,
    pub reason: String,
    #[serde(default = "default_severity")]
    pub severity: Severity,
}

fn default_severity() -> Severity {
    Severity::Medium
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to parse TOML config: {0}")]
    ParseError(#[from] toml::de::Error),
}

pub fn parse_config(toml_str: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(toml_str)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let toml_str = r#"
[severity_actions]
low = "ignore"
medium = "warn"
high = "block"
"#;
        let config = parse_config(toml_str).unwrap();
        assert_eq!(config.severity_actions.low, Action::Ignore);
        assert_eq!(config.severity_actions.medium, Action::Warn);
        assert_eq!(config.severity_actions.high, Action::Block);
        assert!(config.patterns.instruction_override.is_empty());
    }

    #[test]
    fn parse_config_with_patterns() {
        let toml_str = r#"
[severity_actions]
low = "log"
medium = "warn"
high = "block"

[[patterns.instruction_override]]
pattern = '(?i)\bignore\b'
reason = "test pattern"
severity = "high"
"#;
        let config = parse_config(toml_str).unwrap();
        assert_eq!(config.patterns.instruction_override.len(), 1);
        assert_eq!(
            config.patterns.instruction_override[0].reason,
            "test pattern"
        );
    }

    #[test]
    fn default_severity_actions() {
        let actions = SeverityActions::default();
        assert_eq!(actions.low, Action::Log);
        assert_eq!(actions.medium, Action::Warn);
        assert_eq!(actions.high, Action::Block);
    }

    #[test]
    fn invalid_toml_returns_error() {
        let result = parse_config("this is not valid toml [[[");
        assert!(result.is_err());
    }

    #[test]
    fn action_for_severity() {
        let actions = SeverityActions::default();
        assert_eq!(actions.action_for(Severity::Low), Action::Log);
        assert_eq!(actions.action_for(Severity::Medium), Action::Warn);
        assert_eq!(actions.action_for(Severity::High), Action::Block);
    }

    #[test]
    fn pattern_set_iter_with_category() {
        let toml_str = r#"
[[patterns.instruction_override]]
pattern = "test1"
reason = "reason1"
severity = "high"

[[patterns.role_playing]]
pattern = "test2"
reason = "reason2"
severity = "medium"
"#;
        let config = parse_config(toml_str).unwrap();
        let items: Vec<_> = config.patterns.iter_with_category().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, Category::InstructionOverride);
        assert_eq!(items[1].0, Category::RolePlayingDan);
    }
}

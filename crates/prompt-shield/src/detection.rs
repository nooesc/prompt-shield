use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            other => Err(format!("invalid severity: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    InstructionOverride,
    RolePlayingDan,
    EncodingObfuscation,
    ContextManipulation,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::InstructionOverride => "Instruction Override",
            Category::RolePlayingDan => "Role-Playing/DAN",
            Category::EncodingObfuscation => "Encoding/Obfuscation",
            Category::ContextManipulation => "Context Manipulation",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Ignore = 0,
    Log = 1,
    Warn = 2,
    Block = 3,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Ignore => "ignore",
            Action::Log => "log",
            Action::Warn => "warn",
            Action::Block => "block",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Detection {
    pub category: Category,
    pub severity: Severity,
    pub reason: String,
    pub matched_text: String,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
    pub detections: Vec<Detection>,
    pub highest_severity: Option<Severity>,
    pub action: Action,
    pub summary: String,
}

impl ScanResult {
    pub fn clean() -> Self {
        Self {
            detections: Vec::new(),
            highest_severity: None,
            action: Action::Ignore,
            summary: String::new(),
        }
    }

    pub fn has_detections(&self) -> bool {
        !self.detections.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn action_display() {
        assert_eq!(Action::Block.as_str(), "block");
        assert_eq!(Action::Warn.as_str(), "warn");
        assert_eq!(Action::Log.as_str(), "log");
        assert_eq!(Action::Ignore.as_str(), "ignore");
    }

    #[test]
    fn category_display() {
        assert_eq!(
            Category::InstructionOverride.as_str(),
            "Instruction Override"
        );
        assert_eq!(Category::RolePlayingDan.as_str(), "Role-Playing/DAN");
    }

    #[test]
    fn severity_from_str() {
        assert_eq!("high".parse::<Severity>().unwrap(), Severity::High);
        assert_eq!("medium".parse::<Severity>().unwrap(), Severity::Medium);
        assert_eq!("low".parse::<Severity>().unwrap(), Severity::Low);
        assert!("invalid".parse::<Severity>().is_err());
    }

    #[test]
    fn scan_result_clean() {
        let result = ScanResult::clean();
        assert!(!result.has_detections());
        assert_eq!(result.action, Action::Ignore);
        assert!(result.highest_severity.is_none());
    }
}

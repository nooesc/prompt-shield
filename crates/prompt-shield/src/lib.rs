pub mod config;
pub mod detection;

pub use config::{parse_config, Config, ConfigError, PatternEntry, PatternSet, SeverityActions};
pub use detection::{Action, Category, Detection, ScanResult, Severity};

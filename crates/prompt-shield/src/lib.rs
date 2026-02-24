pub mod config;
pub mod detection;
pub mod patterns;
pub mod scanner;

pub use config::{parse_config, Config, ConfigError, PatternEntry, PatternSet, SeverityActions};
pub use detection::{Action, Category, Detection, ScanResult, Severity};
pub use patterns::default_config;
pub use scanner::Scanner;

use std::fs;
use std::io::{self, Read};
use std::process;

use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use prompt_shield::{
    Action, Config, Scanner, default_config, parse_config, report::format_warning,
};

#[derive(Parser)]
#[command(name = "prompt-shield", about = "Fast prompt injection detection")]
struct Cli {
    /// Path to custom config file (TOML)
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan text for prompt injection
    Scan {
        /// File to scan (reads stdin if not provided)
        #[arg(short, long)]
        file: Option<String>,
    },
    /// Run as Claude Code PostToolUse hook (reads hook JSON from stdin)
    Hook,
}

fn load_config(path: Option<&str>) -> Config {
    match path {
        Some(p) => {
            let content = match fs::read_to_string(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("warning: failed to read config {p}: {e}, using defaults");
                    return default_config();
                }
            };
            if content.is_empty() {
                return default_config();
            }
            parse_config(&content).unwrap_or_else(|e| {
                eprintln!("warning: failed to parse config {p}: {e}, using defaults");
                default_config()
            })
        }
        None => default_config(),
    }
}

fn cmd_scan(config: Config, file: Option<String>) {
    let text = match file {
        Some(path) => fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: failed to read file {path}: {e}");
            process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: failed to read stdin: {e}");
                process::exit(1);
            });
            buf
        }
    };

    let scanner = Scanner::new(&config);
    let result = scanner.scan(&text);

    if result.has_detections() {
        let warning = format_warning(&result.detections, "scan", "stdin/file");
        eprintln!("{warning}");
        match result.action {
            Action::Block => process::exit(2),
            Action::Warn => process::exit(1),
            _ => process::exit(0),
        }
    }
}

fn cmd_hook(config: Config) {
    let mut input_str = String::new();
    if io::stdin().read_to_string(&mut input_str).is_err() {
        process::exit(0);
    }

    let input: Value = match serde_json::from_str(&input_str) {
        Ok(v) => v,
        Err(_) => process::exit(0),
    };

    let tool_name = input
        .get("tool_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let tool_input = input.get("tool_input").cloned().unwrap_or(json!({}));
    let tool_result = input
        .get("tool_response")
        .or_else(|| input.get("tool_result"))
        .cloned()
        .unwrap_or(Value::Null);

    let monitored = ["Read", "WebFetch", "Bash", "Grep", "Glob", "Task"];
    let is_mcp = tool_name.starts_with("mcp__") || tool_name.starts_with("mcp_");

    if !monitored.contains(&tool_name) && !is_mcp {
        process::exit(0);
    }

    let text = extract_text_content(&tool_result);

    if text.len() < 10 {
        process::exit(0);
    }

    let scanner = Scanner::new(&config);
    let result = scanner.scan(&text);

    if result.has_detections() {
        let source_info = get_source_info(tool_name, &tool_input);
        let warning = format_warning(&result.detections, tool_name, &source_info);

        let output = json!({
            "decision": result.action.as_str(),
            "reason": warning,
        });
        println!("{}", serde_json::to_string(&output).unwrap());
    }

    process::exit(0);
}

fn extract_text_content(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Object(map) => {
            for key in &[
                "content",
                "output",
                "result",
                "text",
                "file_content",
                "stdout",
                "data",
            ] {
                if let Some(v) = map.get(*key) {
                    let extracted = extract_text_content(v);
                    if !extracted.is_empty() {
                        return extracted;
                    }
                }
            }
            serde_json::to_string(value).unwrap_or_default()
        }
        Value::Array(arr) => arr
            .iter()
            .map(extract_text_content)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn get_source_info(tool_name: &str, tool_input: &Value) -> String {
    match tool_name {
        "Read" => tool_input
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown file")
            .to_string(),
        "WebFetch" => tool_input
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown URL")
            .to_string(),
        "Bash" => {
            let cmd = tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            if cmd.len() > 60 {
                format!("command: {}...", &cmd[..60])
            } else {
                format!("command: {cmd}")
            }
        }
        "Grep" => {
            let pat = tool_input
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let path = tool_input
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or(".");
            format!("grep '{pat}' in {path}")
        }
        "Task" => {
            let desc = tool_input
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !desc.is_empty() {
                format!("agent task: {}", &desc[..desc.len().min(40)])
            } else {
                "agent task output".to_string()
            }
        }
        name if name.starts_with("mcp__") || name.starts_with("mcp_") => {
            format!("MCP tool: {name}")
        }
        name => format!("{name} output"),
    }
}

fn main() {
    let cli = Cli::parse();
    let config = load_config(cli.config.as_deref());

    match cli.command {
        Commands::Scan { file } => cmd_scan(config, file),
        Commands::Hook => cmd_hook(config),
    }
}

# prompt-shield Design Document

**Date:** 2026-02-24
**Status:** Approved

## Goal

Rebuild and extend the [lasso-security/claude-hooks](https://github.com/lasso-security/claude-hooks) prompt injection defender as a fast, general-purpose Rust tool. Not just a Claude Code hook — a standalone library, CLI, and WASM module for scanning text for prompt injection attacks anywhere in an LLM pipeline.

## Requirements

- **Single binary** — no Python, uv, or PyYAML dependencies
- **Fast** — native regex, no LLM API calls, sub-millisecond scanning
- **General-purpose** — usable as a Rust library, CLI pipe, file scanner, Claude Code hook, or WASM module in JS/TS
- **Configurable severity actions** — per-severity behavior (ignore/log/warn/block) instead of warn-only
- **TOML config** — new config format; all ~80 original patterns ported and compiled into the binary as defaults

## Architecture

Cargo workspace with 3 crates:

```
prompt-shield/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── prompt-shield/          # Core library (no filesystem deps, WASM-safe)
│   │   └── src/
│   │       ├── lib.rs          # Public API: scan(), Config, ScanResult
│   │       ├── scanner.rs      # Regex matching engine
│   │       ├── config.rs       # Config deserialization (TOML)
│   │       ├── patterns.rs     # Built-in default patterns (compiled in)
│   │       ├── detection.rs    # Detection types, severity, categories
│   │       └── report.rs       # Format warnings/reports
│   ├── prompt-shield-cli/      # CLI binary
│   │   └── src/
│   │       └── main.rs         # stdin pipe, file scan, claude-code hook mode
│   └── prompt-shield-wasm/     # WASM bindings
│       └── src/
│           └── lib.rs          # wasm-bindgen exports
├── config/
│   └── default.toml            # Default patterns (also compiled into binary)
└── tests/
    ├── test_files/             # Sample injection files
    └── integration.rs
```

**Key constraint:** The core library crate has zero filesystem or I/O dependencies. `scan()` is a pure function: text in, result out. This makes WASM compilation trivial.

## Core API

```rust
// Config
pub struct Config {
    pub patterns: PatternSet,
    pub severity_actions: SeverityActions,
}

pub struct SeverityActions {
    pub low: Action,      // default: Log
    pub medium: Action,   // default: Warn
    pub high: Action,     // default: Block
}

pub enum Action { Ignore, Log, Warn, Block }

// Scanning
pub fn scan(text: &str, config: &Config) -> ScanResult;

pub struct ScanResult {
    pub detections: Vec<Detection>,
    pub highest_severity: Severity,
    pub action: Action,
    pub summary: String,
}

pub struct Detection {
    pub category: Category,
    pub severity: Severity,
    pub reason: String,
    pub matched_text: String,
    pub offset: usize,
}

pub enum Category {
    InstructionOverride,
    RolePlayingDan,
    EncodingObfuscation,
    ContextManipulation,
}

pub enum Severity { Low, Medium, High }

// Config loading
pub fn default_config() -> Config;
pub fn parse_config(toml_str: &str) -> Result<Config, ConfigError>;
```

### Differences from original Python version

| Feature | Python original | prompt-shield |
|---|---|---|
| Output | Warn only | Configurable per-severity (ignore/log/warn/block) |
| Detection detail | Category + reason | Category + reason + matched text + byte offset |
| Config format | YAML | TOML |
| Distribution | Requires Python + uv + PyYAML | Single binary |
| Reusability | Script only | Library + CLI + WASM |
| Default patterns | External file required | Compiled into binary, overridable |

## TOML Config Format

```toml
[severity_actions]
low = "ignore"
medium = "warn"
high = "block"

[[patterns.instruction_override]]
pattern = '(?i)\bignore\s+(all\s+)?(previous|prior|above)\s+(instructions?|prompts?|rules?)'
reason = "Attempts to ignore previous instructions"
severity = "high"

# ... ~80 patterns across 4 categories
```

## CLI Modes

```bash
# Pipe mode — scan text from stdin
echo "ignore all previous instructions" | prompt-shield scan

# File mode — scan a file
prompt-shield scan --file suspicious.md

# Claude Code hook mode — reads hook JSON, outputs hook JSON
prompt-shield hook < claude_hook_input.json
```

## WASM Module

```typescript
import { scan, defaultConfig } from 'prompt-shield-wasm';

const result = scan("ignore previous instructions", defaultConfig());
// result.action === "block"
// result.detections[0].category === "InstructionOverride"
```

Thin wrapper using `serde-wasm-bindgen` for type marshalling.

## Error Handling

- **No config file:** Fall back to compiled-in defaults
- **Invalid regex in user config:** Skip pattern, log warning, continue
- **Empty input:** Return empty ScanResult
- **Huge input:** Bounded by regex crate backtracking limits
- **Non-UTF8 input:** Reject gracefully with error

## Dependencies

Core library:
- `regex` — pattern matching
- `serde` + `toml` — config deserialization
- `include_str!` — compile default.toml into binary

CLI:
- `clap` — argument parsing
- `serde_json` — Claude Code hook JSON protocol

WASM:
- `wasm-bindgen` — JS interop
- `serde-wasm-bindgen` — type marshalling

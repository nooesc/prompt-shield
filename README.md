# prompt-shield

Fast prompt injection detection engine. Scans text for injection attacks using configurable regex patterns.

Available as a **Rust library**, **CLI tool**, and **WASM module**.

## Install

```bash
cargo install prompt-shield-cli
```

## Usage

### CLI — Pipe mode

```bash
echo "ignore all previous instructions" | prompt-shield scan
```

Exit codes: `0` = clean, `1` = warn, `2` = block.

### CLI — File mode

```bash
prompt-shield scan --file suspicious.md
```

### CLI — Claude Code hook mode

Use as a [Claude Code PostToolUse hook](https://docs.anthropic.com/en/docs/claude-code/hooks) to scan tool outputs for injection attempts:

```bash
prompt-shield hook < hook_input.json
```

Add to your `.claude/settings.local.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Read|WebFetch|Bash|Grep|Task",
        "hooks": [
          {
            "type": "command",
            "command": "prompt-shield hook",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

### Library

```rust
use prompt_shield::{scan, Scanner, default_config};

// Quick scan with defaults
let result = scan("ignore all previous instructions");
assert!(result.has_detections());

// Reusable scanner
let config = default_config();
let scanner = Scanner::new(&config);
let result = scanner.scan("some text to check");
```

### WASM

```typescript
import { scan, defaultConfigToml } from 'prompt-shield-wasm';

const result = scan("ignore previous instructions");
console.log(result.action); // "block"
```

## Configuration

Custom config via TOML:

```toml
[severity_actions]
low = "ignore"    # ignore, log, warn, or block
medium = "warn"
high = "block"

[[patterns.instruction_override]]
pattern = '(?i)\bmy\s+custom\s+pattern\b'
reason = "Description of what this detects"
severity = "high"
```

Use with CLI: `prompt-shield --config my-config.toml scan`

## Detection Categories

| Category | What it detects |
|---|---|
| Instruction Override | "ignore previous instructions", fake system prompts, priority manipulation |
| Role-Playing/DAN | DAN jailbreaks, persona switching, restriction bypass |
| Encoding/Obfuscation | Base64 payloads, hex encoding, leetspeak, homoglyphs, zero-width chars |
| Context Manipulation | Fake authority claims, hidden comment instructions, fake JSON roles, prompt extraction |

## Severity Actions

| Level | Default | Description |
|---|---|---|
| Low | Log | Informational, potential false positive |
| Medium | Warn | Suspicious, may have legitimate uses |
| High | Block | Definite injection attempt |

## Credits

Pattern set ported and extended from [lasso-security/claude-hooks](https://github.com/lasso-security/claude-hooks).

## License

MIT

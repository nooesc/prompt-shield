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

## Benchmarks

Compared against [lasso-security/claude-hooks](https://github.com/lasso-security/claude-hooks) (Python/UV), which prompt-shield's pattern set was ported from. Both tools use regex-based detection with ~100 patterns across the same 4 categories.

### Speed

Measured with [hyperfine](https://github.com/sharkdp/hyperfine) (20 runs, 3 warmup). macOS ARM64.

| File | Size | prompt-shield | claude-hooks | Speedup |
|------|------|--------------|-------------|---------|
| clean_small.txt | 698B | 13.9ms | 49.7ms | **3.6x** |
| clean_medium.txt | 10KB | 14.4ms | 58.2ms | **4.0x** |
| clean_large.txt | 100KB | 15.2ms | 119.5ms | **7.9x** |
| clean_xlarge.txt | 1MB | 30.3ms | 716.1ms | **23.6x** |
| inject_small.txt | 872B | 13.3ms | 50.2ms | **3.8x** |
| inject_medium.txt | 10KB | 14.0ms | 54.5ms | **3.9x** |
| inject_large.txt | 100KB | 15.2ms | 112.3ms | **7.4x** |
| inject_xlarge.txt | 1MB | 27.6ms | 731.1ms | **26.5x** |
| inject_dense.txt | 1.5KB | 13.7ms | 48.7ms | **3.5x** |

**3.5x faster on small files, 25x+ faster on large files.** The gap widens with file size due to Rust's compiled regex engine vs Python's interpreter overhead.

### Detection accuracy

Both tools produce identical results across all test files — same detections, same severity classifications, zero false positives on clean text.

## Credits

Pattern set ported and extended from [lasso-security/claude-hooks](https://github.com/lasso-security/claude-hooks).

## License

MIT

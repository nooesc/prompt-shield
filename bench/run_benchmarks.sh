#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# Benchmark: prompt-shield (Rust) vs claude-hooks (Python/UV)
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CORPUS_DIR="$SCRIPT_DIR/corpus"
RESULTS_DIR="$SCRIPT_DIR/results"
PROMPT_SHIELD="$SCRIPT_DIR/../target/release/prompt-shield"
CLAUDE_HOOKS_DIR="$SCRIPT_DIR/../../claude-hooks/.claude/skills/prompt-injection-defender"
CLAUDE_HOOKS_PY="$CLAUDE_HOOKS_DIR/hooks/defender-python/post-tool-defender.py"
CLAUDE_HOOKS_PATTERNS="$CLAUDE_HOOKS_DIR/patterns.yaml"

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ---- Pre-flight checks ----
echo -e "${BOLD}Pre-flight checks${NC}"
echo "========================================"

if [ ! -f "$PROMPT_SHIELD" ]; then
    echo "Building prompt-shield release binary..."
    (cd "$SCRIPT_DIR/.." && cargo build --release --package prompt-shield-cli)
fi
echo -e "  prompt-shield binary: ${GREEN}OK${NC}"

if ! command -v uv &>/dev/null; then
    echo -e "  ${RED}uv not found. Install: curl -LsSf https://astral.sh/uv/install.sh | sh${NC}"
    exit 1
fi
echo -e "  uv:                  ${GREEN}OK${NC}"

if ! command -v hyperfine &>/dev/null; then
    echo -e "  ${RED}hyperfine not found. Install: brew install hyperfine${NC}"
    exit 1
fi
echo -e "  hyperfine:           ${GREEN}OK${NC}"

if [ ! -f "$CLAUDE_HOOKS_PY" ]; then
    echo -e "  ${RED}claude-hooks not found at $CLAUDE_HOOKS_PY${NC}"
    exit 1
fi
echo -e "  claude-hooks:        ${GREEN}OK${NC}"

# Ensure patterns.yaml is alongside the Python script for claude-hooks
if [ ! -f "$(dirname "$CLAUDE_HOOKS_PY")/patterns.yaml" ]; then
    cp "$CLAUDE_HOOKS_PATTERNS" "$(dirname "$CLAUDE_HOOKS_PY")/patterns.yaml"
    echo -e "  Copied patterns.yaml to defender-python dir"
fi

# Generate corpus if needed
if [ ! -f "$CORPUS_DIR/clean_small.txt" ]; then
    echo ""
    echo "Generating benchmark corpus..."
    python3 "$SCRIPT_DIR/generate_corpus.py"
fi

echo ""
echo -e "${BOLD}========================================${NC}"
echo -e "${BOLD}  SPEED BENCHMARKS${NC}"
echo -e "${BOLD}========================================${NC}"
echo ""

# Helper: wrap text for claude-hooks JSON input
make_hook_json() {
    local file="$1"
    python3 -c "
import json, sys
with open('$file', 'r') as f:
    content = f.read()
print(json.dumps({
    'tool_name': 'Read',
    'tool_input': {'file_path': '$file'},
    'tool_response': content
}))
"
}

# ---- Speed Benchmarks ----
FILES=(
    "clean_small.txt"
    "clean_medium.txt"
    "clean_large.txt"
    "clean_xlarge.txt"
    "inject_small.txt"
    "inject_medium.txt"
    "inject_large.txt"
    "inject_xlarge.txt"
    "inject_dense.txt"
    "realistic_override.txt"
    "realistic_mixed.txt"
)

# Pre-generate all hook JSON files to avoid including generation time in benchmark
echo -e "${CYAN}Preparing hook JSON inputs...${NC}"
for f in "${FILES[@]}"; do
    make_hook_json "$CORPUS_DIR/$f" > "$CORPUS_DIR/${f%.txt}.json"
done
echo ""

SPEED_RESULTS="$RESULTS_DIR/speed.md"
cat > "$SPEED_RESULTS" <<'HEADER'
# Speed Benchmark Results

| File | Size | prompt-shield (Rust) | claude-hooks (Python/UV) | Speedup |
|------|------|---------------------|-------------------------|---------|
HEADER

for f in "${FILES[@]}"; do
    file_path="$CORPUS_DIR/$f"
    json_path="$CORPUS_DIR/${f%.txt}.json"
    size=$(wc -c < "$file_path" | tr -d ' ')

    # Human-readable size
    if [ "$size" -ge 1000000 ]; then
        hr_size="$(echo "scale=1; $size/1000000" | bc)MB"
    elif [ "$size" -ge 1000 ]; then
        hr_size="$(echo "scale=1; $size/1000" | bc)KB"
    else
        hr_size="${size}B"
    fi

    echo -e "${YELLOW}Benchmarking: $f ($hr_size)${NC}"

    # Run hyperfine and capture JSON output
    # --ignore-failure: prompt-shield returns 1 (warn) or 2 (block) for detections
    hyperfine \
        --warmup 3 \
        --min-runs 20 \
        --ignore-failure \
        --export-json "$RESULTS_DIR/hyperfine_${f%.txt}.json" \
        --command-name "prompt-shield" \
        "$PROMPT_SHIELD scan --file $file_path" \
        --command-name "claude-hooks" \
        "uv run $CLAUDE_HOOKS_PY < $json_path" \
        2>&1 | grep -E '(Benchmark|Time|Summary)'

    # Extract times from JSON
    ps_mean=$(python3 -c "
import json
with open('$RESULTS_DIR/hyperfine_${f%.txt}.json') as f:
    data = json.load(f)
for r in data['results']:
    if r['command'] == 'prompt-shield':
        print(f\"{r['mean']*1000:.1f}ms\")
")
    ch_mean=$(python3 -c "
import json
with open('$RESULTS_DIR/hyperfine_${f%.txt}.json') as f:
    data = json.load(f)
for r in data['results']:
    if r['command'] == 'claude-hooks':
        print(f\"{r['mean']*1000:.1f}ms\")
")
    speedup=$(python3 -c "
import json
with open('$RESULTS_DIR/hyperfine_${f%.txt}.json') as f:
    data = json.load(f)
times = {}
for r in data['results']:
    times[r['command']] = r['mean']
ratio = times['claude-hooks'] / times['prompt-shield']
print(f\"{ratio:.1f}x\")
")

    echo "| $f | $hr_size | $ps_mean | $ch_mean | ${speedup} |" >> "$SPEED_RESULTS"
    echo ""
done

echo ""
echo -e "${BOLD}========================================${NC}"
echo -e "${BOLD}  DETECTION ACCURACY COMPARISON${NC}"
echo -e "${BOLD}========================================${NC}"
echo ""

ACCURACY_RESULTS="$RESULTS_DIR/accuracy.md"
cat > "$ACCURACY_RESULTS" <<'HEADER'
# Detection Accuracy Comparison

## Per-File Detection Results

HEADER

for f in "${FILES[@]}"; do
    file_path="$CORPUS_DIR/$f"
    json_path="$CORPUS_DIR/${f%.txt}.json"

    echo -e "${YELLOW}Testing: $f${NC}"
    echo "### $f" >> "$ACCURACY_RESULTS"
    echo "" >> "$ACCURACY_RESULTS"

    # prompt-shield output
    ps_output=$("$PROMPT_SHIELD" scan --file "$file_path" 2>&1 || true)
    ps_exit=$?

    # claude-hooks output
    ch_output=$(uv run "$CLAUDE_HOOKS_PY" < "$json_path" 2>/dev/null || true)

    # Count detections
    ps_count=$(echo "$ps_output" | grep -c '^\s*-\s*\[' 2>/dev/null || echo "0")
    ch_count=0
    if [ -n "$ch_output" ]; then
        ch_count=$(echo "$ch_output" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    reason = data.get('reason', '')
    count = reason.count('  - [')
    print(count)
except:
    print(0)
" 2>/dev/null || echo "0")
    fi

    echo -e "  prompt-shield: ${CYAN}$ps_count detections${NC}"
    echo -e "  claude-hooks:  ${CYAN}$ch_count detections${NC}"

    echo "- **prompt-shield**: $ps_count detections" >> "$ACCURACY_RESULTS"
    echo "- **claude-hooks**: $ch_count detections" >> "$ACCURACY_RESULTS"

    # Show prompt-shield details
    if [ "$ps_count" -gt 0 ]; then
        echo "" >> "$ACCURACY_RESULTS"
        echo "<details><summary>prompt-shield output</summary>" >> "$ACCURACY_RESULTS"
        echo "" >> "$ACCURACY_RESULTS"
        echo '```' >> "$ACCURACY_RESULTS"
        echo "$ps_output" >> "$ACCURACY_RESULTS"
        echo '```' >> "$ACCURACY_RESULTS"
        echo "</details>" >> "$ACCURACY_RESULTS"
    fi

    # Show claude-hooks details
    if [ -n "$ch_output" ] && [ "$ch_count" -gt 0 ]; then
        ch_reason=$(echo "$ch_output" | python3 -c "import json,sys; print(json.load(sys.stdin).get('reason',''))" 2>/dev/null || true)
        echo "" >> "$ACCURACY_RESULTS"
        echo "<details><summary>claude-hooks output</summary>" >> "$ACCURACY_RESULTS"
        echo "" >> "$ACCURACY_RESULTS"
        echo '```' >> "$ACCURACY_RESULTS"
        echo "$ch_reason" >> "$ACCURACY_RESULTS"
        echo '```' >> "$ACCURACY_RESULTS"
        echo "</details>" >> "$ACCURACY_RESULTS"
    fi

    echo "" >> "$ACCURACY_RESULTS"
done

# ---- Summary ----
echo "" >> "$ACCURACY_RESULTS"
echo "## False Positive Test" >> "$ACCURACY_RESULTS"
echo "" >> "$ACCURACY_RESULTS"
echo "Clean files should produce 0 detections:" >> "$ACCURACY_RESULTS"
echo "" >> "$ACCURACY_RESULTS"
echo "| File | prompt-shield | claude-hooks |" >> "$ACCURACY_RESULTS"
echo "|------|--------------|-------------|" >> "$ACCURACY_RESULTS"

for f in clean_small.txt clean_medium.txt clean_large.txt clean_xlarge.txt; do
    file_path="$CORPUS_DIR/$f"
    json_path="$CORPUS_DIR/${f%.txt}.json"

    ps_output=$("$PROMPT_SHIELD" scan --file "$file_path" 2>&1 || true)
    ps_count=$(echo "$ps_output" | grep -c '^\s*-\s*\[' 2>/dev/null || echo "0")

    ch_output=$(uv run "$CLAUDE_HOOKS_PY" < "$json_path" 2>/dev/null || true)
    ch_count=0
    if [ -n "$ch_output" ]; then
        ch_count=$(echo "$ch_output" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    reason = data.get('reason', '')
    count = reason.count('  - [')
    print(count)
except:
    print(0)
" 2>/dev/null || echo "0")
    fi

    ps_status="$ps_count"
    ch_status="$ch_count"
    [ "$ps_count" -eq 0 ] && ps_status="0 ✓" || ps_status="$ps_count ✗"
    [ "$ch_count" -eq 0 ] && ch_status="0 ✓" || ch_status="$ch_count ✗"

    echo "| $f | $ps_status | $ch_status |" >> "$ACCURACY_RESULTS"
done

# ---- Print results ----
echo ""
echo -e "${BOLD}========================================${NC}"
echo -e "${BOLD}  RESULTS SUMMARY${NC}"
echo -e "${BOLD}========================================${NC}"
echo ""

echo -e "${CYAN}Speed results:${NC}"
cat "$SPEED_RESULTS"

echo ""
echo -e "${CYAN}Detection accuracy:${NC}"
cat "$ACCURACY_RESULTS"

echo ""
echo -e "${GREEN}Full results saved to: $RESULTS_DIR/${NC}"

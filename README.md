# jimmy-skill

`jimmy-skill` is a Rust CLI that talks directly to ChatJimmy, a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec. It is designed to be invoked by Claude Code skills, using a split-brain pattern: Claude reasons and plans, then dispatches work to Jimmy for cheap parallel generation, returning structured JSON. No authentication required — ChatJimmy is in open beta.

## How it works

Claude skills fan out work to `jimmy-skill` via `--parallel` mode: a JSON array of prompts goes in, an ordered JSON array of results comes out. Jimmy handles the HTTP concurrency internally. Claude evaluates the results and picks the best output. The result: Claude-quality final decisions at a fraction of the cost and time of running everything through the Claude API.

**Performance:** 1,800 generations in ~6 seconds, $0.00 cost. See [DEMO.md](DEMO.md) for a full benchmark.

## Requirements

- Rust (stable) and Cargo — install via [rustup.rs](https://rustup.rs)
- Internet access (calls `https://chatjimmy.ai/api/chat`)
- Claude Code (for the `.claude` skills)

## Build and Install

### Build the binary

```bash
git clone <repo-url>
cd jimmy-tool
cargo build --release
```

The binary is at `target/release/jimmy-skill`.

### Install system-wide

Copy the binary to a directory on your `$PATH`:

```bash
# macOS / Linux
cp target/release/jimmy-skill /usr/local/bin/jimmy-skill

# Or add the release dir to your PATH in ~/.zshrc / ~/.bashrc:
export PATH="$PATH:/path/to/jimmy-tool/target/release"
```

Verify it works:

```bash
jimmy-skill "Hello, Jimmy"
# {"response":"Hello! How can I assist you today?","tokens":{"prompt":13,"completion":10,"total":23},"elapsed_ms":412}
```

## Binary usage

### Single-prompt mode

```bash
jimmy-skill "<prompt>"
jimmy-skill "<prompt>" --system "<system prompt>"
jimmy-skill "<prompt>" --system @path/to/system.txt   # read system prompt from file

# Read prompt from stdin
echo "What is 2+2?" | jimmy-skill
```

**Output:**
```json
{
  "response": "4",
  "tokens": { "prompt": 12, "completion": 2, "total": 14 },
  "elapsed_ms": 389
}
```

**Error output** (also JSON, always to stdout; details to stderr):
```json
{
  "response": null,
  "tokens": { "prompt": 0, "completion": 0, "total": 0 },
  "elapsed_ms": 0,
  "error": "ChatJimmy API request failed",
  "error_type": "api"
}
```

### Parallel batch mode

Pipe a JSON array of `{ "prompt": "...", "system"?: "..." }` objects to stdin:

```bash
echo '[
  {"prompt": "Name a color"},
  {"prompt": "Name a fruit"},
  {"prompt": "Name a planet"}
]' | jimmy-skill --parallel
```

**Options:**
- `--max-concurrent N` — max simultaneous HTTP requests (default: 100)
- `--max-iterations N` — repeat each prompt N times per item (default: 25); useful for getting multiple variations cheaply

**Output:** JSON array, one item per input, in index order. Each item includes `index`, `results` (array of N responses), `tokens`, and `elapsed_ms`. Per-item failures include `error` and `error_type` without affecting other items.

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Usage error (bad arguments, empty prompt) |
| 2 | API / network error |
| 3 | Parse error |

## Install the Claude Code skills

The `.claude/skills/` directory contains Claude Code skills that use `jimmy-skill` as their execution engine. To use them, copy or symlink the skills directory into your project's `.claude/` folder, or into `~/.claude/` for global access.

### Option 1: Copy into a project

```bash
# From inside your project directory:
cp -r /path/to/jimmy-tool/.claude/skills ~/.claude/skills
# or merge into existing:
cp -r /path/to/jimmy-tool/.claude/skills/jimmy-* ~/.claude/skills/
```

### Option 2: Symlink for live updates

```bash
ln -s /path/to/jimmy-tool/.claude/skills/jimmy-candidates ~/.claude/skills/jimmy-candidates
ln -s /path/to/jimmy-tool/.claude/skills/jimmy-validate   ~/.claude/skills/jimmy-validate
# ... repeat for each skill
```

### Option 3: Global install (all skills at once)

```bash
mkdir -p ~/.claude/skills
for skill_dir in /path/to/jimmy-tool/.claude/skills/jimmy-*/; do
  skill_name=$(basename "$skill_dir")
  ln -sf "$skill_dir" ~/.claude/skills/"$skill_name"
done
```

After copying/linking, restart Claude Code or open a new session. The skills will appear in `/help` and can be invoked with `/jimmy-candidates`, `/jimmy-validate`, etc.

> **Prerequisite:** `jimmy-skill` must be on your `$PATH` for the skills to work. Skills invoke the binary via `jimmy-skill --parallel ...` in a Bash tool call.

## Available skills

| Skill | What it does |
|-------|-------------|
| `/jimmy-candidates` | Generate N candidate responses — pick the best |
| `/jimmy-validate` | Fan out N yes/no questions in parallel — get boolean results with a pass/fail summary |
| `/jimmy-transform` | Rewrite text in multiple styles (one-to-many) or apply one instruction across N inputs (many-to-one) |
| `/jimmy-summarize` | Summarize a text; set `max_iterations > 1` for multiple summary variants |
| `/jimmy-classify` | Majority-vote text classification via N ensemble calls |
| `/jimmy-montecarlo` | Measure prompt stability — run N identical calls and check agreement rate |
| `/jimmy-search` | Generate N candidates and test each against shell oracle commands; returns ranked results |
| `/jimmy-testdata` | Generate N structured test fixtures from a schema |
| `/jimmy-fuzz` | Generate adversarial payload variants for manual security testing |

Full parameter docs for each skill are in its `SKILL.md`. Example prompts for every skill are in [TRY_ME.md](TRY_ME.md).

## Quick examples

**Generate 8 wording candidates and pick the best:**
```
Use /jimmy-candidates with prompt: "Write a one-sentence error message for an invalid file path" and n: 8
```

**Run a 6-question code review checklist:**
```
Use /jimmy-validate with context: "<paste your function here>" and questions: [
  "Does this function handle null input?",
  "Is there error handling for network failures?",
  "Are all public methods documented?"
]
```

**Classify a commit message:**
```
Use /jimmy-classify with text: "fix: prevent crash when user email is null" and categories: ["bug fix", "feature", "chore", "docs"] and n: 7
```

**Test a prompt for stability before using it in production:**
```
Use /jimmy-montecarlo with prompt: "Classify as bug/feature/chore: fix null pointer in UserService" and n: 20 and threshold: 0.7
```

## The split-brain pattern

Jimmy is not Claude. An 8B model gets ~40-60% of nuanced technical questions right; Claude gets them right consistently. The pattern that works:

1. **Jimmy generates volume** — 10-100+ candidates, transforms, or validations in seconds, for free
2. **Claude evaluates** — spends its tokens only on judgment: pick the best, flag the failures, reason about the results
3. **Net result** — Claude-quality output at Jimmy speed and cost

Don't use Jimmy for reasoning. Use it for generation at scale, then hand off to Claude.

## Development

```bash
# Run tests
cargo test

# Check for lint errors
cargo clippy

# Build debug binary
cargo build
```

Tests cover JSON serialization, stats block parsing, CLI argument handling, and parallel output contracts. Integration tests mock the ChatJimmy HTTP endpoint via `wiremock`.

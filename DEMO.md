# jimmy-skill Demo: 1,800 LLM Generations in 6 Seconds

This document records a benchmark run of all four jimmy-skill Claude Code skills operating in parallel with `--max-iterations 50`. The goal: show how many tokens Jimmies can produce when jimmy-skill fans out requests at full blast, and compare the cost and speed to running the same workload through the Claude API.

## What Was Tested

All four skills ran simultaneously, each as a single `jimmy-skill --parallel --max-concurrent 20 --max-iterations 50` invocation:

| Skill | Items | x Iterations | Total Generations |
|-------|------:|:------------:|------------------:|
| **jimmy-candidates** | 10 prompts | x 50 | **500** |
| **jimmy-transform** | 10 styles | x 50 | **500** |
| **jimmy-validate** | 15 questions | x 50 | **750** |
| **jimmy-summarize** | 1 text | x 50 | **50** |
| **TOTAL** | **36** | | **1,800** |

## Results

### Token Volume

| Metric | Value |
|--------|------:|
| Total generations | **1,800** |
| Total tokens (input + output) | **224,344** |
| Input tokens | 124,250 |
| Output tokens | 100,094 |
| Errors | **0** |
| Useful response rate | **100%** |

### Timing

All four skills ran concurrently. The slowest one determines wall-clock time.

| Skill | Wall Clock | Latency p50 | Latency p95 | Min | Max |
|-------|----------:|------------:|------------:|----:|----:|
| candidates (500) | 5.46s | 98ms | 112ms | 82ms | 382ms |
| transform (500) | 6.17s | 101ms | 125ms | 81ms | 345ms |
| validate (750) | 5.60s | 100ms | 114ms | 85ms | 367ms |
| summarize (50) | 5.75s | 107ms | 116ms | 98ms | 292ms |
| **Total wall clock** | **~6.2s** | | | | |

The first iteration per item is the slowest (300-400ms, cold connection). Subsequent iterations drop to ~90-110ms thanks to HTTP connection pooling inside the `reqwest` client.

### Usefulness Metrics

"Useful" = non-empty response that is structurally valid for the skill's purpose.

| Skill | Useful | Total | Rate | Notes |
|-------|-------:|------:|-----:|-------|
| candidates | 500 | 500 | 100% | All returned tagline text |
| transform | 500 | 500 | 100% | All returned transformed text |
| validate | 750 | 750 | 100% | All parseable to YES/NO |
| summarize | 50 | 50 | 100% | All returned summary text |
| **Total** | **1,800** | **1,800** | **100%** | |

**Validate accuracy note:** While 100% of responses were parseable (started with YES or NO), only 40% of majority-vote answers matched the expected correct answer across the 15 questions. This is expected for an 8B model on nuanced technical questions — the split-brain pattern addresses this by having Claude evaluate Jimmy's output rather than trusting it directly.

### Sample Outputs

**Best candidate taglines** (from 500 generations):
1. "Faster inference at the edge, one token at a time."
2. "Effortless LLM scaling: 17K tokens/sec, 0 developer hassle"
3. "Unlock unparalleled speed: 17K tokens/sec, limitless LLM possibilities"

**Transform samples** (1 per style, from 50 variations each):
- **Haiku:** "Jimmy's skill born / Claude's thoughts in swift streams / Seventeen thousand"
- **Limerick:** "In Jimmy's skill, thoughts quickly flow / 17 thousand tokens, as the seconds go..."
- **Country song:** "Well, I woke up this mornin', my mind was racin' fast / Claude was thinkin', but Jimmy had the last word at last..."
- **Zen koan:** "Jimmy's brushstrokes: Claude's thoughts, 17K flowers."

**Best one-sentence summaries** (from 50 variations):
1. "Jimmy-Skill is a CLI that pairs with Claude for reasoning and planning, using a hardware-accelerated Llama 3.1 model to generate output at scale in structured JSON format."
2. "Jimmy-Skill is a fast and low-cost Rust CLI that pairs with Claude for reasoning and planning, leveraging a hardware-accelerated Llama 3.1 model to generate output at scale."

## Cost Comparison: Jimmy vs Claude API

### What this workload would cost through Claude

| | Jimmy (Llama 3.1 8B) | Claude 3.5 Haiku | Claude 3.5 Sonnet | Claude Opus 4 |
|---|---:|---:|---:|---:|
| **Output tokens** | 100,094 | 100,094 | 100,094 | 100,094 |
| **Input tokens** | 124,250 | 124,250 | 124,250 | 124,250 |
| **Output cost** | $0.00 | $0.10 | $1.50 | $7.51 |
| **Input cost** | $0.00 | $0.03 | $0.37 | $1.86 |
| **Total cost** | **$0.00** | **$0.13** | **$1.87** | **$9.37** |
| **Wall clock (est.)** | **~6s** | **~5-8 min** | **~10-15 min** | **~25-40 min** |

**Pricing used** (as of 2025):
- Haiku: $0.25/M input, $1.25/M output
- Sonnet: $3/M input, $15/M output
- Opus: $15/M input, $75/M output

### Time estimate methodology

Claude API calls are typically 1-3 seconds each for short completions. Even with batching:
- 1,800 calls at 1s/call serial = **30 minutes**
- With 10x parallelism via batch API = **~3 minutes** (best case)
- Jimmy did it in **6 seconds**

### The split-brain value proposition

The point is NOT that Jimmy is smarter than Claude. The validate demo proves it isn't — Jimmy got 40% of nuanced technical questions wrong. The point is:

1. **Jimmy generates volume** — 1,800 completions in 6 seconds, for free
2. **Claude evaluates** — spends its expensive tokens only on picking the best from Jimmy's output
3. **Net result** — you get Claude-quality final output at a fraction of the cost and time

Example workflow: generate 50 tagline candidates with Jimmy ($0.00, 5s), then ask Claude to pick the best 3 ($0.01, 2s). Total: $0.01 and 7 seconds. Doing the same entirely in Claude: $0.17 and ~2 minutes.

## Reproduce This Benchmark

### Prerequisites

1. Clone the repo and build the binary:
   ```bash
   git clone <repo-url>
   cd jimmy-tool
   cargo build --release
   ```

2. Verify the binary works:
   ```bash
   ./target/release/jimmy-skill "Hello, Jimmy"
   ```
   You should get a JSON response with `response`, `tokens`, and `elapsed_ms` fields.

### Run the benchmark

Run all four skills concurrently (they are independent processes):

```bash
# Create output directory
mkdir -p /tmp/jimmy-bench

# Run all 4 in parallel (background the first 3, foreground the last)
time ./target/release/jimmy-skill --parallel --max-concurrent 20 --max-iterations 50 << 'EOF' > /tmp/jimmy-bench/candidates.json &
[
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"},
  {"prompt": "Write ONE sentence: a tagline for jimmy-skill, a Rust CLI that fans out LLM calls at 17K tokens/sec"}
]
EOF

time ./target/release/jimmy-skill --parallel --max-concurrent 20 --max-iterations 50 << 'EOF' > /tmp/jimmy-bench/transform.json &
[
  {"prompt": "rewrite as a haiku\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a limerick\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite for a pirate audience\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a Shakespearean couplet\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a movie trailer voiceover\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a nature documentary narration\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as an infomercial pitch\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a fairy tale opening\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a country song verse\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."},
  {"prompt": "rewrite as a zen koan\n\nText to transform:\njimmy-skill: Claude thinks, Jimmy generates, 17K tok/s.\n\nTransformed text:", "system": "Output ONLY the transformed text."}
]
EOF

time ./target/release/jimmy-skill --parallel --max-concurrent 20 --max-iterations 50 << 'EOF' > /tmp/jimmy-bench/validate.json &
[
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is Rust a good language for a high-concurrency CLI tool?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Does this tool need a database?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is 17,000 tokens per second fast for an 8B model?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is the split-brain pattern a valid architecture?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Should it output XML instead of JSON?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is semaphore-based concurrency appropriate for fan-out?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Would auth improve this tool during open beta?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is tokio the correct async runtime for reqwest?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is JSON the right output format for machine-consumed CLI output?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Should this CLI include colored terminal output?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Can an 8B model reliably answer yes/no factual questions?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is fan-out parallelism useful when the server handles concurrent requests?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is Python better than Rust here?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Should errors go to stderr to protect JSON stdout?", "system": "Answer YES or NO, then one sentence of reasoning."},
  {"prompt": "Context:\njimmy-skill is a Rust CLI calling a local Llama 3.1 8B at 17K tok/s with parallel fan-out.\n\nQuestion: Is clap the best CLI argument parser for Rust?", "system": "Answer YES or NO, then one sentence of reasoning."}
]
EOF

time ./target/release/jimmy-skill --parallel --max-concurrent 20 --max-iterations 50 << 'EOF' > /tmp/jimmy-bench/summarize.json
[
  {"prompt": "jimmy-skill is a Rust CLI that serves as the fast, cheap worker in a Claude + Jimmy split-brain pattern. Claude reasons and plans, Jimmy (hardware-accelerated Llama 3.1 8B at 17K tokens/sec) generates at scale. The tool supports single-prompt mode and parallel batch mode with semaphore-based concurrency. It outputs structured JSON consumed by Claude Code skills.", "system": "Produce a concise summary in exactly ONE sentence."}
]
EOF

# Wait for background jobs
wait

echo "Done. Outputs in /tmp/jimmy-bench/"
```

### Analyze the results

```bash
python3 << 'PYEOF'
import json, glob

total_tokens = 0
total_results = 0
total_errors = 0

for f in sorted(glob.glob("/tmp/jimmy-bench/*.json")):
    data = json.load(open(f))
    name = f.split("/")[-1].replace(".json", "")
    results = 0
    tokens = 0
    errors = 0
    for item in data:
        for r in item["results"]:
            results += 1
            tokens += r.get("tokens", {}).get("total", 0)
            if r.get("response") is None:
                errors += 1
    total_tokens += tokens
    total_results += results
    total_errors += errors
    print(f"{name:>12}: {results:>5} results, {tokens:>7,} tokens, {errors} errors")

print(f"{'TOTAL':>12}: {total_results:>5} results, {total_tokens:>7,} tokens, {total_errors} errors")
print()

# Claude cost comparison
output_tokens = total_tokens * 0.45  # rough output ratio
input_tokens = total_tokens * 0.55
print("Claude API cost for same workload:")
for name, inp, outp in [("Haiku", 0.25, 1.25), ("Sonnet", 3, 15), ("Opus", 15, 75)]:
    cost = (input_tokens / 1e6 * inp) + (output_tokens / 1e6 * outp)
    print(f"  {name}: ${cost:.2f}")
print(f"  Jimmy: $0.00")
PYEOF
```

### Key metrics to record

After running, record these numbers for your system:

| Metric | Your Value |
|--------|-----------|
| Total generations | |
| Total tokens | |
| Wall clock (slowest skill) | |
| Errors | |
| Useful response rate (%) | |
| Tokens per second (total_tokens / wall_clock) | |

### Tuning parameters

- `--max-concurrent`: Controls how many HTTP requests are in-flight simultaneously. Higher = faster but more load on the LLM server. Start at 10, increase to 20-50 if the server handles it.
- `--max-iterations`: How many times each prompt is repeated. This is the primary volume knob. Each iteration reuses the HTTP connection (fast) and produces a unique response (temperature > 0 on the server).
- **Item count**: Number of JSON items in the input array. Each item runs `max_iterations` times. More items = more parallelism across different prompts.

**Scaling formula:** `total_generations = items x max_iterations`

This benchmark used `36 items x 50 iterations = 1,800 generations`.


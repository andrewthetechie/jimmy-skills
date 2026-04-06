# Skill: jimmy-montecarlo

You are executing the jimmy-montecarlo skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for running the same prompt many times to measure how consistently it performs.

This skill sends ONE prompt to Jimmy N times (via `--max-iterations N`) and computes variance metrics: agreement rate (how often the top response appears), unique response count, response distribution (per-unique frequency), and length statistics (mean/stddev/min/max). A stability verdict ("stable"/"unstable") is derived from the agreement rate vs a configurable threshold. The skill tells you whether your prompt is reliable before you invest Claude reasoning on it.

Use it when you need:
- Statistical confidence that a prompt performs consistently before production use
- A quick "is this prompt good enough?" answer before spending Sonnet/Opus tokens
- Quantitative comparison of two prompt variants (higher agreement_rate = more stable)
- Regression testing after system prompt changes (run before and after, compare verdict)

## Inputs

You will be provided:
- `prompt`: string — the prompt to test for stability (required, non-empty)
- `n`: integer — number of samples; minimum 10 recommended (required, >= 1)
- `system` (optional): string — system prompt for every Jimmy call; keep under 400 chars
- `threshold` (optional, default 0.7): float 0.0–1.0 — agreement rate cutoff for "stable" verdict
- `max_concurrent` (optional integer, default 100): maximum simultaneous HTTP requests

## Step 1: Validate inputs

Check each input before proceeding. For any validation failure, output the bare JSON error object and stop immediately. Do not output markdown fences.

- `prompt` missing or empty string: `{"error": "prompt is required", "error_type": "usage"}`
- `n` missing, zero, or negative: `{"error": "n must be a positive integer", "error_type": "usage"}`
- `threshold` provided and outside 0.0–1.0: `{"error": "threshold must be between 0.0 and 1.0", "error_type": "usage"}`
- `max_concurrent` provided and less than 1: `{"error": "max_concurrent must be >= 1", "error_type": "usage"}`

If `n` is between 1 and 9 inclusive: do NOT error. Continue with execution but set a `_low_n_warning` flag internally — the warning will be added to the output in Step 5.

## Step 2: Construct the JSON array and call jimmy-skill --parallel

jimmy-montecarlo sends ONE prompt item with `--max-iterations N`. This causes the binary to send the identical prompt N times and return all N results under `output[0].results[0..N-1]`.

**System prompt construction:**
- No `system` param provided: use `"You are a helpful assistant."` (keep minimal — category lists etc. go in the prompt, not system)
- With `system` param: use the caller's system text directly (do not prepend extra text)

**Bash invocation (ONE Bash tool call):**

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations N << 'JIMMY_INPUT'
[{"prompt": "PROMPT_TEXT", "system": "SYSTEM_TEXT"}]
JIMMY_INPUT
```

Replace `N` with the value of `n`. Replace `PROMPT_TEXT` with the actual prompt. Replace `MAX_CONCURRENT` with the provided value (default 100). Replace `SYSTEM_TEXT` with the resolved system prompt. Escape any `"` characters in prompt or system text as `\"` in the JSON. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` characters.

This is ONE Bash tool call. The binary handles all concurrency internally.

IMPORTANT: This output shape differs from jimmy-classify. All N responses are under `output[0].results[0..N-1]`, NOT under `output[0..N-1].results[0]`. Read `output[0].results[i].response` for i in 0..N-1 — not `output[i].results[0].response`.

## Step 3: Collect results

Wait for the single Bash call to complete. stdout is a JSON array. Parse it. The array has exactly one item (index 0). `output[0].results` contains N response objects.

Extract all N responses:
```
raw_responses = [output[0].results[i].response for i in 0..N-1]
```

If `output[0].results[i].response` is null for some i (API or network error): treat that response as an empty string `""` for normalization and counting purposes. It contributes to the unique count and response_distribution as `""`. Its length for null responses is 0 (counted in length stats).

## Step 4: Compute metrics

**Normalization function** (apply to responses before counting uniqueness and agreement — NOT for length stats):
```
normalize(s):
  1. If s is null: return ""
  2. Lowercase
  3. Trim leading and trailing whitespace
  4. Strip trailing punctuation characters (.,!?;:)
  5. Trim again
```

**Step-by-step computation:**

```
# Normalize all responses for counting
normalized = [normalize(r) for r in raw_responses]  (length N)

# Unique count and response_distribution
response_distribution = {}  (normalized string -> count)
for each s in normalized:
  response_distribution[s] = (response_distribution[s] or 0) + 1
unique_responses = len(keys in response_distribution)

# Agreement rate
top_response_value = key in response_distribution with highest count
  (if tie: pick lexicographically first among tied keys)
top_count = response_distribution[top_response_value]
agreement_rate = top_count / N  (divide by N, not by unique count or successful count)
Round to 4 decimal places.

# Length stats — computed on RAW (unnormalized) responses, not normalized
lengths = [len(r) if r is not null else 0 for r in raw_responses]
length_mean = sum(lengths) / N
length_stddev = sqrt(sum((l - length_mean)^2 for l in lengths) / N)  (population stddev)
length_min = min(lengths)
length_max = max(lengths)
Round mean and stddev to 2 decimal places.

# Verdict
effective_threshold = threshold if provided else 0.7
verdict = "stable" if agreement_rate >= effective_threshold else "unstable"
```

## Step 5: Assemble and return output

Assemble the output object using exactly these fields:

```json
{
  "prompt": "<the original prompt string from input>",
  "samples": N,
  "metrics": {
    "unique_responses": <unique_responses>,
    "agreement_rate": <agreement_rate>,
    "top_response": { "value": "<top_response_value (normalized)>", "count": <top_count> },
    "length": { "mean": <length_mean>, "stddev": <length_stddev>, "min": <length_min>, "max": <length_max> },
    "response_distribution": { "<response_key>": <count>, ... }
  },
  "verdict": "<stable or unstable>",
  "raw_responses": [<array of N original response strings as returned by Jimmy, null preserved as null>]
}
```

If `_low_n_warning` flag is set (n was 1–9): add a `"warning"` field to the top-level object (alongside `prompt`, `samples`, etc.):
```json
"warning": "n < 10 produces statistically unreliable variance estimates; recommend n >= 10"
```

Output the JSON object only. No markdown fences, no commentary — bare JSON starting with `{` and ending with `}`.

Full example output for N=20, threshold=0.7 (default):

```json
{
  "prompt": "Classify this as bug/feature/chore: Fix null pointer in UserService",
  "samples": 20,
  "metrics": {
    "unique_responses": 3,
    "agreement_rate": 0.75,
    "top_response": { "value": "bug", "count": 15 },
    "length": { "mean": 4.2, "stddev": 1.8, "min": 3, "max": 9 },
    "response_distribution": { "bug": 15, "feature": 3, "chore": 2 }
  },
  "verdict": "stable",
  "raw_responses": ["bug", "Bug", "bug", "feature", "bug", "bug", "chore", "bug", "bug", "bug", "bug", "feature", "bug", "bug", "feature", "bug", "chore", "bug", "bug", "bug"]
}
```

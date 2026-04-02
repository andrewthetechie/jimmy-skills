# jimmy-candidates

Generate N candidate responses from a prompt using the jimmy-skill binary (ChatJimmy / Llama 3.1 8B at ~17K tokens/sec).

## When to use this skill

Use jimmy-candidates when you need fast, cheap generation of multiple variations — then apply your own reasoning to pick or refine the best one. Good for:

- **Wording candidates** — "Give me 5 ways to phrase this error message"
- **Draft content** — "Generate 3 README intros, I'll pick the best"
- **Parallel exploration** — "Try 4 different approaches to this function signature"
- **Cheap validation** — "Generate 10 yes/no answers to check consistency"
- **Creative variation** — "Write 5 taglines for this product"

Not suited for: long reasoning chains, tool use, precise instruction-following, or anything requiring Claude-level intelligence. Jimmy is fast and cheap — use it for volume.

## Important: no --parallel flag

`jimmy-skill` handles ONE prompt and returns ONE result. There is no `--parallel` flag. Parallelism is achieved by the skill issuing N separate Bash tool calls in one response — Claude Code runs them simultaneously. See jimmy-candidates.md for details.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `prompt`  | yes      | The prompt to send to Jimmy (same prompt sent to all N calls) |
| `n`       | yes      | Number of candidates to generate (integer >= 1; practical max ~10) |
| `system`  | no       | Optional system prompt shared across all candidates |

## Output

JSON array of N items. Each item has the shape of `JimmyOutput` plus an `index` field (0-based):

```json
[
  { "index": 0, "response": "...", "tokens": { "prompt": 12, "completion": 9, "total": 21 }, "elapsed_ms": 843 },
  { "index": 1, "response": null, "tokens": { "prompt": 0, "completion": 0, "total": 0 }, "elapsed_ms": 45, "error": "Request timed out", "error_type": "timeout" }
]
```

Per-item failures set `response: null`, `error`, and `error_type` without affecting other items. Array always has exactly N items.

`error_type` values: `"timeout"`, `"network"`, `"api"`, `"parse"`, `"usage"`.

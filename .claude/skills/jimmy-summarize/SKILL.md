# jimmy-summarize

Summarize a text using the jimmy-skill binary (ChatJimmy / Llama 3.1 8B at ~17K tokens/sec).

## When to use this skill

Use jimmy-summarize when you need fast, cheap summarization of a single text. Good for:

- **Condense a long document** before passing to Claude for reasoning
- **Get multiple summary variants** (set max_iterations > 1) to compare styles
- **Pre-process text** to fit within context limits
- **Cheap first-pass summaries at scale** before applying Claude's judgment

Not suited for: multi-document summarization, structured extraction, precise formatting requirements, or anything requiring Claude-level reasoning about the content.

## How parallelism works here

`jimmy-skill --parallel` handles all concurrency internally. The skill constructs a 1-item JSON array and pipes it to the binary in a single Bash call. `max_iterations` makes Jimmy run the same prompt multiple times to produce multiple summaries cheaply — ideal for getting variation without multiple Bash calls.

See jimmy-summarize.md for full details.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `text` | yes | The text to summarize |
| `system` | no | Optional system prompt appended to the skill default for style/length guidance |
| `max_iterations` | no | Number of summaries to generate. Default: 1. Jimmy runs at ~17K tokens/sec — set max_iterations > 1 to get multiple summaries cheaply. |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 10. |

## Output

The full parallel array from the binary, passed through as-is:

```json
[
  {
    "index": 0,
    "results": [
      { "response": "...", "tokens": { "prompt": N, "completion": N, "total": N }, "elapsed_ms": N }
    ]
  }
]
```

For `max_iterations=1` (default), there is one result in `results`. For `max_iterations > 1`, `results` has multiple entries — one per iteration. Per-item failures set `response: null` with `error` and `error_type` fields.

---
name: jimmy-candidates
description: Use when you need multiple wording or draft candidates from a fast cheap LLM, parallel variations of one prompt, scaffolding options to pick from, or bulk generation before spending Claude tokens on volume.
---

# jimmy-candidates

Generate **N** candidate responses via `jimmy-skill`. You pick/refine; Jimmy supplies volume.

**REQUIRED:** Read [jimmy-cli](references/jimmy-cli.md) for invocation rules.

## Not for

Long reasoning, tool use, precise instruction-following, or Claude-level judgment.

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `prompt` | yes | — | Same prompt for every candidate |
| `n` | yes | — | Candidates (≥1; practical max ~20) |
| `system` | no | — | Shared `--system` |
| `max_concurrent` | no | 10 | Cap simultaneous HTTP |
| `max_iterations` | no | 1 | Prefer raising `n`; leave at 1 unless you want nested results |

## Steps

1. **Validate** — empty `prompt` → `{"error":"prompt is required","error_type":"usage"}`. Non-positive `n` / `max_concurrent` / `max_iterations` → matching usage error. Stop.
2. **Call once** — N identical JSON items; pass `--max-iterations` explicitly (CLI default is 25):

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations MAX_ITERATIONS [--system "SYSTEM"] << 'JIMMY_INPUT'
[{"prompt":"PROMPT"},{"prompt":"PROMPT"}]
JIMMY_INPUT
```

3. **Reshape** — for each item `I`, take `results[0]` into:
   - success: `{ "index", "response", "tokens", "elapsed_ms" }`
   - failure: same + `"response": null`, `error`, `error_type`
   Array length must equal `n`.
4. **Return** — bare JSON array only (no markdown fences, no commentary).

## Example

`prompt`: "Write a one-sentence error for an invalid file path", `n`: 5 → five flat candidate objects.

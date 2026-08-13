---
name: jimmy-map
description: Explicit generic Jimmy map over many inputs with a caller-supplied prompt template.
---

# Jimmy map

Apply one prompt template to many inputs while preserving input identity and every iteration result. Use this explicit primitive when no specialized `jimmy-*` skill owns the task.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Follow [the mapping protocol](references/mapping.md) for substitution and optional JSON parsing.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `inputs` | yes | — | Non-empty strings or `{ id, input, system? }` objects; values and IDs are strings; IDs are unique |
| `prompt_template` | yes | — | Non-empty string containing `{input}`; may also contain `{id}` and `{index}` |
| `system` | no | — | Shared system instruction prepended to per-input `system` |
| `iterations` | no | 1 | Positive responses per input |
| `output` | no | `raw` | `raw` or `json` |
| `max_concurrent` | no | 50 | Positive integer |

## Process

1. **Validate.** Enforce every input contract and template placeholder. Return a bare `usage` error object before calling Jimmy on the first invalid input.
2. **Map.** Substitute each input as data in input order and JSON-serialize exactly one CLI item per input.
3. **Batch.** Invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations ITERATIONS`.
4. **Shape.** Apply the mapping protocol to every iteration, preserving API and optional JSON parse failures.
5. **Return.** Emit only `{ "results", "summary" }`. Completion requires `results.length = inputs.length` and exactly `iterations` outputs per input.

---
name: jimmy-fuzz
description: Generate inert adversarial payload strings with Jimmy for authorized, human-controlled security testing.
---

# Jimmy fuzz

Generate `n` inert payload strings for each requested attack category.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Use [the payload protocol](references/payloads.md) for prompts, severities, and output shaping.

## Safety boundary

Keep generated payloads as in-memory data and return them for human-controlled testing. Never execute them, interpolate them into shell commands, write them to disk, or send them to a live target. Use `jimmy-search` only when the caller has supplied an authorized, sandboxed oracle.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `attack_surface` | yes | — | Non-empty description |
| `attack_types` | yes | — | Non-empty array of normalization-distinct category strings; custom categories are allowed |
| `n` | yes | — | Positive integer per category |
| `max_concurrent` | no | 10 | Positive integer for the whole batch |

## Process

1. **Validate.** Enforce every input contract. Return a bare `usage` error object and stop on the first invalid input. Validation is complete when the batch cardinality is `n × attack_types.length`.
2. **Batch.** Build all category/payload items in category order, then item order. JSON-serialize the full array and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`. Do not make one call per category.
3. **Shape.** Apply the payload protocol. The global index is `(category_position × n) + item_position`; preserve API failures at their original indices.
4. **Return.** Emit only `{ "warning", "payloads", "summary" }`. Completion requires `payloads.length = n × attack_types.length`, summary counts that reconcile to that length, and a warning that the agent has not executed the inert strings.

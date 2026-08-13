---
name: jimmy-prompt-ab
description: Compare two or more prompt variants across repeated Jimmy trials. Use for labeled prompt evaluation, regression checks, or stability and latency comparisons before deploying a prompt.
---

# Jimmy prompt A/B

Run every prompt variant against every case and repetition, then rank variants with deterministic quality, reliability, agreement, and latency metrics.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Follow [the experiment protocol](references/experiment.md) for trial construction, scoring, aggregation, and winner selection.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `variants` | yes | — | At least two `{ name, prompt, system? }` objects; unique names; each prompt contains `{input}` |
| `cases` | yes | — | Non-empty strings or `{ id, input, expected? }` objects; IDs and values are non-empty strings; IDs are unique |
| `scoring` | no | `exact` | `exact`, `contains`, `regex`, or `none` |
| `repetitions` | no | 5 | Positive trials per variant/case pair |
| `system` | no | — | Shared system instruction prepended to variant `system` |
| `max_concurrent` | no | 100 | Positive integer |

## Process

1. **Validate.** Enforce every input contract. Require a non-empty string `expected` on every case unless `scoring` is `none`; compile expected regexes before any call. Return a bare `usage` error object on the first invalid input.
2. **Matrix.** Build trials in variant, case, repetition order. Replace `{input}` as data; expected answers never enter Jimmy prompts.
3. **Batch.** JSON-serialize the complete matrix and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
4. **Evaluate.** Score and aggregate every trial with the experiment protocol, preserving API failures at their matrix indices.
5. **Return.** Emit only `{ "winner", "variants", "trials", "summary" }`. Completion requires `trials.length = variants.length × cases.length × repetitions` and reconciled per-variant counts.

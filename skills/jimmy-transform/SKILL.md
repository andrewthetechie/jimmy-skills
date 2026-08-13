---
name: jimmy-transform
description: Transform text in parallel with Jimmy, either one input across many instructions or many inputs under one instruction, with optional regex or length enforcement.
---

# Jimmy transform

Transform text in one of two modes:

- **one-to-many:** `input` + `instructions[]`
- **many-to-one:** `inputs[]` + `instruction`

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. When `validate` is present, follow [the validation and retry protocol](references/validate-retry.md).

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| mode pair | yes | — | Exactly one complete pair above; every string and array is non-empty |
| `system` | no | — | Appended to the writer system instruction |
| `max_concurrent` | no | 10 | Positive integer |
| `validate` | no | — | `pattern`, `length`, or `both`; see the retry protocol |
| `max_retries` | no | 2 | Non-negative integer; used only with `validate` |

## Process

1. **Validate.** Accept exactly one complete mode pair and enforce every input contract. Validate the optional constraint before any call. Return a bare `usage` error object naming the invalid field and stop on the first failure. Validation is complete when mode and output cardinality `n` are unambiguous.
2. **Prompt.** For each index, build `INSTRUCTION\n\nText to transform:\nINPUT\n\nTransformed text:`. Start the system instruction with `You are a skilled writer. Follow the transformation instruction precisely.` and append caller `system` on a new line.
3. **Batch.** JSON-serialize exactly `n` items and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
4. **Enforce.** When configured, validate and retry failed constraints with the retry protocol. Preserve original API failures without retrying them.
5. **Return.** Emit only an array of `{ "index", "input", "instruction", "result", "tokens", "elapsed_ms" }` successes or equivalent failures with `result: null`, `error`, and `error_type`. Completion requires exactly `n` items in original order.

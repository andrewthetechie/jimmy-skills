---
name: jimmy-validate
description: Validate independent binary claims against supplied context with Jimmy and return parsed booleans plus aggregate counts.
---

# Jimmy validate

Fan out independent yes/no questions, parse each answer, and aggregate pass/fail/error counts. Use a stronger model when a question needs multi-step reasoning or comparison.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Parse responses with [the boolean protocol](references/parse.md).

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `questions` | yes | — | Non-empty array of strings or `{ question, context? }` objects; each question is non-empty |
| `context` | no | — | Shared context; per-question context overrides it |
| `system` | no | — | Appended to the validator system instruction |
| `max_concurrent` | no | 10 | Positive integer |

## Process

1. **Validate.** Enforce every input contract. Return a bare `usage` error naming the first invalid question index and stop before calling Jimmy.
2. **Prompt.** Use `You are a validator. Answer YES or NO, then give one sentence of reasoning.` as the base system instruction and append caller `system` on a new line. Build each user prompt as `Context:\nCONTEXT\n\nQuestion: QUESTION`, omitting the context block when absent.
3. **Batch.** JSON-serialize one item per question and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
4. **Parse.** Apply the boolean protocol to every `results[0]`, preserving API and parse failures at their original indices.
5. **Return.** Emit only `{ "results", "summary": { "total", "passed", "failed", "errors" } }`. Completion requires `results.length = questions.length` and `passed + failed + errors = total`.

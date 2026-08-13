---
name: jimmy-summarize
description: Summarize supplied text with Jimmy for cheap compression or several summary variants before higher-level analysis.
---

# Jimmy summarize

Generate one or more summaries of the same text. The invoking agent owns any analytical synthesis across documents or claims.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `text` | yes | — | Non-empty text to summarize |
| `system` | no | — | Appended to the default summary instruction |
| `max_iterations` | no | 1 | Positive number of variants |

## Process

1. **Validate.** Reject empty `text` or `max_iterations < 1` with a bare `usage` error object. Stop before calling Jimmy.
2. **Prompt.** Start with `You are a summarizer. Produce a concise summary of the following text in 2-3 sentences.` Append caller `system` on a new line when supplied.
3. **Batch.** JSON-serialize one item containing `text` and the merged system instruction. Invoke parallel mode once with `--max-concurrent 1 --max-iterations MAX_ITERATIONS`.
4. **Return.** Emit the CLI array unchanged. Completion requires one outer item whose `results` array contains exactly `max_iterations` success-or-failure entries.

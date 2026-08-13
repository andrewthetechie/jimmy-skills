---
name: jimmy-classify
description: Classify text into exactly one supplied category with a Jimmy ensemble vote. Use for cheap routing or tagging among semantically distinct labels.
---

# Jimmy classify

Vary the classification prompt across `n` calls, then return the majority label and confidence.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Use [the voting protocol](references/vote.md) to build prompts and tally responses.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `text` | yes | — | Non-empty string |
| `categories` | yes | — | At least two non-empty, normalization-distinct labels |
| `n` | yes | — | Integer at least 3; 7 is a useful baseline |
| `system` | no | — | Appended to the classifier system instruction |
| `max_concurrent` | no | 100 | Positive integer |

Use a stronger model for multi-label or reasoning-heavy decisions.

## Process

1. **Validate.** Enforce every input contract, including category uniqueness after the normalization in the voting protocol. Return a bare `usage` error object and stop on the first invalid input. Validation is complete when every vote has at least two unambiguous labels to choose from.
2. **Diversify.** Build exactly `n` items by cycling the seven prompt templates. Use `You are a text classifier. Be concise.` as the base system instruction and append caller `system` on a new line when supplied.
3. **Batch.** JSON-serialize the items and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`. The call is complete when the CLI returns one ordered item per vote.
4. **Vote.** Extract and tally every response using the voting protocol; retain null/parse failures in `errors` and `raw_responses`.
5. **Return.** Emit only `{ "classification", "confidence", "votes", "total_votes", "raw_responses", "errors"? }`. Completion requires `total_votes + errors.length = n`.

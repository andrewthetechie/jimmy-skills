---
name: jimmy-candidates
description: Generate independent draft candidates with Jimmy when one prompt benefits from many cheap responses that a stronger model will select or refine.
---

# Jimmy candidates

Fan out one prompt into `n` independent drafts. Jimmy supplies volume; the invoking agent owns judgment and final selection.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `prompt` | yes | — | Non-empty string sent to every item |
| `n` | yes | — | Positive integer; practical maximum about 20 |
| `system` | no | — | Shared system instruction |
| `max_concurrent` | no | 10 | Positive integer |

## Process

1. **Validate.** Reject an empty `prompt`, `n < 1`, or `max_concurrent < 1` with a bare `usage` error object. Stop before calling Jimmy. Validation is complete when all four inputs satisfy the table.
2. **Batch.** JSON-serialize exactly `n` items containing the same `prompt` and optional `system`. Invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`. The call is complete when the CLI returns an array that satisfies its contract.
3. **Shape.** Map each `items[i].results[0]` to:
   - success: `{ "index", "response", "tokens", "elapsed_ms" }`
   - failure: the same fields with `"response": null`, plus `error` and `error_type`
4. **Return.** Emit only the JSON array. Completion requires exactly `n` output items in input order, including failures.

---
name: jimmy-montecarlo
description: Measure Jimmy prompt stability by repeating an identical prompt and computing exact-response agreement before relying on its output shape or wording.
---

# Jimmy Monte Carlo

Repeat one prompt `n` times and measure normalized exact-response agreement. Semantic equivalence and factual accuracy remain with the invoking agent.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Compute the result with [the metrics protocol](references/metrics.md).

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `prompt` | yes | — | Non-empty string, unchanged across samples |
| `n` | yes | — | Positive integer; values below 10 require a warning |
| `threshold` | no | 0.7 | Number in `[0, 1]` |
| `system` | no | `You are a helpful assistant.` | Replaces the default when supplied |

## Process

1. **Validate.** Enforce every input contract. Return a bare `usage` error object and stop on invalid input; retain a low-sample warning when `n < 10`.
2. **Repeat.** JSON-serialize one item and invoke parallel mode once with `--max-concurrent 1 --max-iterations N`. Read all samples from `output[0].results`; the call is complete only when that array contains exactly `n` entries.
3. **Measure.** Apply the metrics protocol to all successes and failures. API failures reduce reliability and can never contribute to agreement.
4. **Return.** Emit only `{ "prompt", "samples", "metrics", "verdict", "raw_responses", "warning"? }`. Completion requires `raw_responses.length = n` and metric counts that sum to `n`.

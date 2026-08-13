---
name: jimmy-montecarlo
description: Use when you need to measure prompt stability, agreement rate across repeated identical Jimmy calls, variance before production prompts, or a stable/unstable verdict with response distribution.
---

# jimmy-montecarlo

Run **one** prompt **N** times (`--max-iterations N`); compute agreement metrics and a stability verdict.

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Metrics: [metrics](references/metrics.md).

## Not for

Generating diverse candidates (use jimmy-candidates) or classification voting (use jimmy-classify).

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `prompt` | yes | — | Identical prompt every sample |
| `n` | yes | — | Samples (≥1; warn if `<10`) |
| `threshold` | no | 0.7 | Agreement cutoff for `stable` |
| `system` | no | `You are a helpful assistant.` | Caller replaces default if set |
| `max_concurrent` | no | 100 | |

## Steps

1. **Validate** — empty prompt / bad `n` / threshold outside 0–1 → usage error. If `1 ≤ n ≤ 9`, continue but set low-n warning for output.
2. **Call once** — **one** JSON item; set `--max-iterations` to `n` (this is the diversity lever):

```bash
jimmy-skill --parallel --max-concurrent MAX --max-iterations N << 'JIMMY_INPUT'
[{"prompt":"PROMPT","system":"SYSTEM"}]
JIMMY_INPUT
```

Read `output[0].results[0..N-1]` — **not** `output[i].results[0]`.

3. **Metrics** — [metrics](references/metrics.md).
4. **Return** — bare JSON object only (`prompt`, `samples`, `metrics`, `verdict`, `raw_responses`, optional `warning`).

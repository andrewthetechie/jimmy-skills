# Monte Carlo metrics (jimmy-montecarlo)

`raw_responses[i] = output[0].results[i].response` (preserve nulls in output; treat null as `""` for counting/length).

## normalize(s) — for uniqueness/agreement only

null → `""`; lowercase; trim; strip trailing `.,!?;:`; trim again.

## Compute

- `response_distribution`: counts of normalized strings
- `unique_responses`: distinct keys
- `top_response`: highest count; tie → lexicographically first key; `agreement_rate = top_count / N` (4 decimals)
- Length stats on **raw** strings (null length 0): mean, population stddev, min, max (mean/stddev 2 decimals)
- `verdict`: `stable` if `agreement_rate >= threshold` else `unstable` (default threshold 0.7)

If n was 1–9: add `"warning": "n < 10 produces statistically unreliable variance estimates; recommend n >= 10"`.

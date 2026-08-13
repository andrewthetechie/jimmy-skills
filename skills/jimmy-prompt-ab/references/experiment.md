# Prompt experiment protocol

## Construct trials

Normalize a string case to `{ id: zero_based_index, input: string }`. For each variant, substitute every literal `{input}` in its prompt with the case input. Merge systems as `SHARED\nVARIANT` when both exist. Expected values remain evaluator-only data.

Flatten trials in variant, case, repetition order. Retain that coordinate map while sending one Jimmy batch with `--max-iterations 1`.

## Score

Normalize exact values by lowercasing, trimming, and stripping trailing `.,!?;:`. Score successful responses as follows:

- `exact`: normalized response equals normalized expected value.
- `contains`: lowercased response contains the lowercased expected value.
- `regex`: expected value is a regex searched over the full response.
- `none`: `pass` is null and quality ranking is omitted.

API failures have null response and pass. Retain every raw response, token count, elapsed time, and error at its trial coordinate.

## Aggregate each variant

- `trials`, `successes`, `api_errors`, and `reliability_rate = successes / trials`
- `scored`, `passed`, and `pass_rate = passed / scored` when scoring is active; use zero when `scored = 0`
- Per case, normalize successful responses as for exact scoring and find the modal count; break modal-response ties lexicographically
- `agreement_rate = sum(case modal counts) / successes`, or zero without successes
- `mean_elapsed_ms` and nearest-rank `p95_elapsed_ms` over successes; both are null without successes
- Sum prompt, completion, and total tokens over all trials

Round rates to four decimals and mean latency to two decimals; p95 is an observed integer latency. Rank by pass rate when scoring is active, then reliability, agreement, lower mean latency, and lexicographic variant name. With `scoring: none`, start at reliability. A null mean latency sorts last. `winner` is the winning variant name, or null when every variant has zero successes. Keep the `variants` metrics array in input order.

Each trial record contains zero-based `index` and `repetition`, `variant`, `case_id`, `response`, `pass`, `tokens`, `elapsed_ms`, and optional error fields. Each variant record contains the metrics named above. `summary` is `{ "variants", "cases", "repetitions", "total_trials", "successful_trials", "api_errors", "scoring" }`. Aggregation is complete when every flattened trial belongs to exactly one variant and case and `successful_trials + api_errors = total_trials`.

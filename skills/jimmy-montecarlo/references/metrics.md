# Monte Carlo metrics protocol

Set `raw_responses[i] = output[0].results[i].response`, preserving nulls. Count a null as an API failure and exclude it from response agreement.

## Normalize successful responses

Lowercase, trim whitespace, strip trailing `.,!?;:`, then trim again. Use normalized strings only for uniqueness and agreement; use raw strings for length metrics.

## Compute

Let `N` be requested samples, `S` non-null responses, and `E = N - S` failures.

- `samples`: `N`
- `successful_samples`: `S`
- `api_errors`: `E`
- `reliability_rate`: `S / N`
- `response_distribution`: counts of normalized successful strings
- `unique_responses`: number of distribution keys
- `top_response`: highest-count normalized response; break ties lexicographically; null when `S = 0`
- `agreement_rate`: `top_count / S`; use `0` when `S = 0`
- `length`: mean, population standard deviation, minimum, and maximum over raw successful response character counts; all zero when `S = 0`

Round rates to four decimals and mean/standard deviation to two decimals.

`verdict` is `stable` only when `E = 0` and `agreement_rate >= threshold`; otherwise it is `unstable`. When `N < 10`, add `"warning": "n < 10 produces statistically unreliable variance estimates; recommend n >= 10"`.

Metrics are complete when `successful_samples + api_errors = samples` and the response-distribution counts sum to `successful_samples`.

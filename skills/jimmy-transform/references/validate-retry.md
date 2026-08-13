# Transform validation and retry protocol

This branch applies only when `validate` is present.

## Validate the constraint

- `type: pattern` requires a non-empty, compilable regex in `pattern`.
- `type: length` requires at least one non-negative integer bound: `min_length` or `max_length`.
- `type: both` requires both the regex and at least one length bound.
- When both length bounds exist, require `min_length <= max_length`.

Reject any other type with a `usage` error before calling Jimmy.

## Check a response

- `pattern`: regex search over the full response.
- `length`: Unicode character count satisfies every supplied bound.
- `both`: both checks pass.

Produce a concrete failure reason containing the failed pattern or measured length and bound.

## Retry in rounds

1. Keep only original successes that fail the constraint; original API failures are final.
2. For each retry round up to `max_retries`, build one batch containing all currently constraint-failing items with their original prompt and system instruction. Invoke `jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations 1` once for the round.
3. Map retry results back to original indices. Accept passes, retain constraint failures for the next round, and make retry API failures final.
4. After the last round, emit remaining constraint failures with `result: null`, `error_type: "validation"`, and `error: "validation failed: REASON"`.

Retries are additional attempts: `max_retries: 2` permits the initial batch plus two retry batches. Return tokens and elapsed time from the accepted attempt. Retry processing is complete when every original index is accepted or final and the output cardinality is unchanged.

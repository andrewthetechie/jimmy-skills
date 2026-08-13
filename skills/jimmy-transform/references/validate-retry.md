# Validate + retry (jimmy-transform)

Skip entirely if `validate` absent.

## Check

- `pattern`: regex search against full result string
- `length`: `min_length` / `max_length` bounds (whichever present)
- `both`: pattern AND length must pass

## Retry

On fail only: re-call **single-item** (not `--parallel`) with same prompt/system for that index, `--max-iterations 1`. Up to `max_retries` additional attempts (default 2 → 3 total calls). Sequential per failing item.

Exhausted → `result: null`, `error_type: "validation"`, `error: "validation failed: ..."`.

Do not retry original API nulls. Silent retries (no narration).

## Output items

Success: `{ index, input, instruction, result, tokens, elapsed_ms }`  
API fail: `{ index, input, instruction, result: null, error, error_type }`  
Validation fail: same as API fail with `error_type: "validation"`.

`result` is raw response text — do not strip. Array length = N. No summary object.

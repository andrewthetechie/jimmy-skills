# Boolean response protocol

For question index `i`, read `parallel_output[i].results[0]`.

An API failure becomes `{ "index", "question", "pass": null, "error", "error_type", "raw": null }`.

## Parse a successful response

Recognize ASCII case-insensitive `YES`, `TRUE`, `NO`, or `FALSE` only as whole words (not adjacent to a letter, digit, or underscore). Map `YES`/`TRUE` to true and `NO`/`FALSE` to false. Reject `1`, `0`, and synonyms.

1. Trim leading whitespace. If a recognized token starts at position zero, use it.
2. Otherwise inspect the first 50 characters and use the recognized token with the earliest start position.
3. If no token matches, emit a parse failure.

For a match, `explanation` is the trimmed text after the token and `raw` is the complete unmodified response.

## Shape and reconcile

- Success: `{ "index", "question", "pass", "explanation", "raw" }`
- Parse failure: `{ "index", "question", "pass": null, "error": "Could not parse YES/NO from response", "error_type": "parse", "raw" }`

The plain question string is used even when its input was an object. Summary counts true, false, and null as `passed`, `failed`, and `errors`; parsing is complete when those counts sum to the input length.

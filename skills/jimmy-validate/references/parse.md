# Boolean parse (jimmy-validate)

For each index `I`, `raw = parallel_output[I].results[0].response`.

If `raw` is null → error item: `pass: null`, `error` from CLI, `error_type` from CLI, `raw: null`.

## Two-pass parse

**Pass 1 — prefix:** strip leading whitespace. If starts with YES/TRUE/NO/FALSE (case-insensitive): set `pass` (YES/TRUE→true, NO/FALSE→false), `explanation` = trim(text after token).

**Pass 2 — scan:** first 50 chars; first whole-word YES/TRUE/NO/FALSE. Same mapping. No match → parse error item.

Reject bare `1`/`0` and other tokens.

## Item shapes

Success: `{ "index", "question", "pass", "explanation", "raw" }`  
Parse fail: `{ "index", "question", "pass": null, "error": "Could not parse YES/NO from response", "error_type": "parse", "raw" }`

`question` is the plain question string. Results length must equal N.

## Summary

`total=N`, `passed`/`failed`/`errors` = counts where `pass` is true / false / null.

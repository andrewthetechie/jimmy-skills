# Classification voting protocol

Join categories as comma-separated `{CATS}`. For vote index `i`, use template `(i mod 7) + 1` and substitute `{TEXT}` and `{CATS}` as data:

```text
1: Text: {TEXT}\nCategories: {CATS}\n\nClassify as exactly one category. Reply with ONLY the category name.
2: Text: {TEXT}\nWhich of these categories best describes the text? {CATS}\nRespond with only the category name.
3: Text: {TEXT}\nLabel this text as one of the following: {CATS}\nOutput only the label.
4: Categorize: {TEXT}\nOptions: {CATS}\nAnswer with just the category.
5: Text: {TEXT}\nSelect the most appropriate category: {CATS}\nOne word or phrase only.
6: Pick the best category for the following text.\nText: {TEXT}\nCategories: {CATS}\nReply with the category name only.
7: What category does this text belong to?\nText: {TEXT}\nChoose one: {CATS}\nRespond with the category name.
```

## Normalize

Lowercase, replace `-` and `_` with spaces, strip leading/trailing `.,!?:;`, collapse whitespace, then trim. Reject input categories that normalize to the same string.

## Extract one label

Keep original labels for output. A whole-word match is not adjacent to a letter, digit, or underscore.

1. If normalized response exactly equals one normalized category, select it.
2. Otherwise collect categories that whole-word-match at the response prefix; select the longest normalized match, breaking equal-length ties by input order.
3. Otherwise scan the first 100 characters. Select the earliest whole-word category match, then the longest match, then input order.
4. No match is a parse error for that vote; continue tallying the others.

This ordering makes overlapping labels such as `bug` and `bug fix` deterministic.

## Tally

- `votes` contains every original category with its integer count.
- `raw_responses` contains all `n` response strings or nulls in vote order.
- `errors` records API and parse failures with their vote indices.
- `total_votes` is the sum of category counts.
- With zero valid votes, return `classification: null` and `confidence: 0`.
- Otherwise select the greatest count; break a tied winning count by lexicographic original label. `confidence = winner_count / total_votes`.

Tallying is complete when `total_votes + errors.length = n`.

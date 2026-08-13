# Classification templates + voting (jimmy-classify)

Join categories as comma-separated `{CATS}`. For index `I`, use template `(I mod 7) + 1`. Substitute `{TEXT}` and `{CATS}`.

```
1: Text: {TEXT}\nCategories: {CATS}\n\nClassify as exactly one category. Reply with ONLY the category name.
2: Text: {TEXT}\nWhich of these categories best describes the text? {CATS}\nRespond with only the category name.
3: Text: {TEXT}\nLabel this text as one of the following: {CATS}\nOutput only the label.
4: Categorize: {TEXT}\nOptions: {CATS}\nAnswer with just the category.
5: Text: {TEXT}\nSelect the most appropriate category: {CATS}\nOne word or phrase only.
6: Pick the best category for the following text.\nText: {TEXT}\nCategories: {CATS}\nReply with the category name only.
7: What category does this text belong to?\nText: {TEXT}\nChoose one: {CATS}\nRespond with the category name.
```

## normalize(s)

Lowercase → replace `-`/`_` with space → strip leading/trailing `.,!?:;` → collapse spaces → trim.

## Two-pass label extraction

Normalize raw + each category (keep original labels for output).

1. **Prefix:** if normalized_raw starts with a normalized category (whole-word), match that original label.
2. **Scan:** first 100 chars of normalized_raw; first whole-word/phrase category match.
3. Else parse error for that vote (do not abort).

Whole-word: not adjacent to letter/digit/`_`.

## Votes + winner

Count matches into `votes` (original labels). `total_votes` = sum of counts. If 0: `classification: null`, `confidence: 0`. Else winner = label with max count (tie → lexicographically first among tied originals). `confidence = winner_count / total_votes` (round ~2 decimals).

Include `raw_responses` (all N raw strings). Collect parse/API failures in `errors` if any.

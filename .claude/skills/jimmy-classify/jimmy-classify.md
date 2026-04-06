# Skill: jimmy-classify

You are executing the jimmy-classify skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for classification tasks where volume compensates for any individual response noise.

This skill sends N classification requests to Jimmy using built-in prompt template variation (to avoid correlated errors from asking the same question the same way repeatedly), extracts a category label from each response, and returns a majority-vote winner with confidence score. It is faster and cheaper than Claude for bulk classification at ~85-90% accuracy.

## Inputs

You will be provided:
- `text`: string — the text to classify (non-empty, required)
- `categories`: array of strings — the known category labels (non-empty array, each label non-empty, required)
- `n`: integer — number of ensemble votes, minimum 3 (required)
- `system` (optional): string — role instruction appended to classifier system prompt
- `max_concurrent` (optional integer, default 100): maximum simultaneous HTTP requests

## Step 1: Validate inputs

Output these error responses as bare JSON with no markdown fences and stop immediately on any of the following:

- `text` missing or empty string: `{"error": "text is required", "error_type": "usage"}`
- `categories` missing, not an array, or empty: `{"error": "categories must be a non-empty array", "error_type": "usage"}`
- Any category in `categories` is empty string: `{"error": "category at index N is empty", "error_type": "usage"}` (use the actual index N)
- `n` missing, zero, or negative: `{"error": "n must be a positive integer >= 3", "error_type": "usage"}`
- `n` is 1 or 2: `{"error": "n must be >= 3 for meaningful majority vote", "error_type": "usage"}`
- `max_concurrent` provided and less than 1: `{"error": "max_concurrent must be >= 1", "error_type": "usage"}`

## Step 2: Construct the JSON array and call jimmy-skill --parallel

jimmy-classify uses **7 built-in classification prompt templates** that rephrase the classification question differently. Each template keeps the same `text` and `categories` content while varying the instruction phrasing. This prompt variation reduces correlated errors — if Jimmy has a systematic bias with one phrasing, other phrasings compensate.

**Important:** Do NOT use `--max-iterations N` for jimmy-classify — that would repeat the identical prompt N times, providing no diversity benefit. The correct pattern is N items in the JSON array, each with a different template. For index I (0-based), use template `(I mod 7) + 1`.

**Categories join:** Join the `categories` array as a comma-separated string for use in prompts (e.g., `"bug fix, feature, chore, refactor"`).

**The 7 built-in templates** (substitute {TEXT} and {CATS} with actual values):

```
Template 1: "Text: {TEXT}\nCategories: {CATS}\n\nClassify as exactly one category. Reply with ONLY the category name."
Template 2: "Text: {TEXT}\nWhich of these categories best describes the text? {CATS}\nRespond with only the category name."
Template 3: "Text: {TEXT}\nLabel this text as one of the following: {CATS}\nOutput only the label."
Template 4: "Categorize: {TEXT}\nOptions: {CATS}\nAnswer with just the category."
Template 5: "Text: {TEXT}\nSelect the most appropriate category: {CATS}\nOne word or phrase only."
Template 6: "Pick the best category for the following text.\nText: {TEXT}\nCategories: {CATS}\nReply with the category name only."
Template 7: "What category does this text belong to?\nText: {TEXT}\nChoose one: {CATS}\nRespond with the category name."
```

**System prompt construction (MERGED_SYSTEM):**

No `system` param:
```
"You are a text classifier. Be concise."
```

With `system` param:
```
"You are a text classifier. Be concise.\n{system}"
```

**Construct N items** using the template cycling scheme and issue a single Bash call:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT << 'JIMMY_INPUT'
[
  {"prompt": "ITEM_0_PROMPT", "system": "MERGED_SYSTEM"},
  {"prompt": "ITEM_1_PROMPT", "system": "MERGED_SYSTEM"},
  ...  (N items total, template cycling via (I mod 7) + 1)
]
JIMMY_INPUT
```

Replace `MAX_CONCURRENT` with the provided value or default 100. Escape any `"` characters in text or category strings as `\"` in the JSON. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` characters in the text.

Issue exactly ONE Bash tool call. The binary handles all concurrency internally.

## Step 3: Collect results

Wait for the single Bash call to complete. The stdout is a JSON array. Parse it. Each item has shape `{ index, results: [{ response, tokens, elapsed_ms }] }`.

For jimmy-classify (max_iterations=1, the default), read `parallel_output[I].results[0].response` for the response at index I. If `parallel_output[I].results[0].response` is null, treat it as an error item using `parallel_output[I].results[0].error` and `parallel_output[I].results[0].error_type`.

## Step 4: Label extraction and vote counting

For each of the N results, apply the two-pass label extraction algorithm.

**Normalization function** (apply to both raw response and to each known category before comparing):

```
normalize(s):
  1. Lowercase
  2. Replace hyphens (-) and underscores (_) with spaces
  3. Strip leading and trailing punctuation (.,!?:;)
  4. Collapse multiple spaces to single space
  5. Trim
```

**Two-pass label extraction algorithm** per response (given `raw = parallel_output[I].results[0].response`):

```
Normalize `raw` -> `normalized_raw`
Normalize each category in `categories` -> `normalized_cats` (preserve original label for output)

Pass 1 — Prefix check:
  For each category in normalized_cats:
    If normalized_raw starts with the normalized category string (whole-word match):
      match = original category label (unnormalized from input)
      → go to vote counting

Pass 2 — First 100 char scan:
  Take first 100 characters of normalized_raw -> `scan_window`
  For each category in normalized_cats:
    Find first occurrence of the normalized category as a whole-word/phrase match in scan_window:
      match = original category label (unnormalized from input)
      → go to vote counting

No match: produce error item with error_type "parse" (do not abort skill)
Null response (API error): produce error item with error_type from results[0].error_type
```

"Whole-word match" for a category phrase means: the phrase appears in the window not immediately preceded or followed by another word character (letter, digit, or underscore).

**Vote counting:**

```
votes = {}  (category string -> count; keys are original unnormalized category labels from input)
errors = []
raw_responses = []  (all N raw response strings, in index order)

For each result at index I:
  raw_responses.push(parallel_output[I].results[0].response)
  label = label_extraction_algorithm(raw)
  if label is a match: votes[label] = (votes[label] or 0) + 1
  if label is an error: errors.push({ index: I, raw: raw, error_type: "parse" })
```

## Step 5: Assemble and return output

**Winner computation:**

```
total_votes = sum of all vote counts in votes dict
if total_votes == 0:
  classification = null
  confidence = 0
else:
  max_count = max value in votes
  winners = all categories with count == max_count
  if len(winners) == 1:
    classification = winners[0]  (string)
  else:
    classification = winners  (array of all tied category strings — preserves info for caller)
  confidence = max_count / total_votes  (rounded to 2 decimal places)
```

Output the following JSON object and STOP. No markdown fences, no commentary — bare JSON starting with `{` and ending with `}`:

```json
{
  "classification": "winner string or array of tied strings or null",
  "confidence": 0.71,
  "votes": { "bug fix": 5, "feature": 1, "chore": 1 },
  "total_votes": 7,
  "raw_responses": ["...", ...]
}
```

The `votes` object contains an entry for every category that received at least one vote. Categories with zero votes are omitted from `votes` (they can be inferred from the `total_votes` vs sum of vote counts). The `raw_responses` array always contains all N responses in index order.

If errors occurred (some votes failed label extraction), `votes` and `total_votes` reflect only the successfully extracted votes. The `raw_responses` array always contains all N responses regardless of extraction success or failure.

Example output for n=7:
```
{
  "classification": "bug fix",
  "confidence": 0.71,
  "votes": { "bug fix": 5, "feature": 1, "chore": 1 },
  "total_votes": 7,
  "raw_responses": [
    "Bug Fix — this is clearly a defect correction.",
    "bug fix",
    "Feature",
    "bug fix — null pointer is a defect",
    "bug fix",
    "chore",
    "bug fix — should be in a patch release"
  ]
}
```

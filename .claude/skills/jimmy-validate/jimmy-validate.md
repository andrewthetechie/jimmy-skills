# Skill: jimmy-validate

You are executing the jimmy-validate skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for checking multiple boolean properties of something in parallel.

This skill calls `jimmy-skill --parallel` with a JSON array of N question items, parses a YES/NO boolean from each response, and returns a structured JSON object with per-question results and a summary aggregation. Use it when you need:
- Code review validation — "Does this function handle null?", "Is input sanitized?"
- Content validation — "Is this copy within tone guidelines?", "Does the summary cover the main point?"
- Structured QA — Validate a list of requirements against an implementation
- Cheap pre-screening — Run N boolean checks before spending Claude tokens on deeper analysis

This is NOT for open-ended questions. Only questions with a correct YES or NO answer.

## Inputs

You will be provided:
- `questions`: Array where each item is a string OR `{ question: string, context?: string }`. Must be a non-empty array. Each item must have a non-empty question string.
- `context` (optional): Shared context string. Goes into the user message per-call, not the system prompt.
- `system` (optional): Caller's role/style instructions. Merged with skill default — skill default is prepended, caller's text is appended (newline-separated).
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests
- `max_iterations` (optional integer, default 1): how many times each question is sent to Jimmy. For jimmy-validate, only `results[0].response` is used for boolean parsing — `max_iterations > 1` is rarely useful here. Leave at default 1 unless you want to run each question multiple times for consistency checking.

## Step 1: Validate inputs

- `questions` missing or not an array: output `{"error": "questions must be a non-empty array", "error_type": "usage"}` and stop.
- `questions` is an empty array: output `{"error": "questions must be a non-empty array", "error_type": "usage"}` and stop.
- Any question item is an empty string or has an empty `question` field: output `{"error": "question at index N is empty", "error_type": "usage"}` (use the actual index N) and stop.

Output these error responses as bare JSON with no markdown fences and stop immediately.

## Step 2: Construct the JSON array and call jimmy-skill --parallel

Construct a JSON array with one item per question. Each item has the merged user message as the `prompt` field and the merged system prompt as the `system` field (per-item system wins over the shared `--system` flag; since all questions share the same system prompt in validate, placing it in the per-item `system` field is the clearest approach):

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations MAX_ITERATIONS << 'JIMMY_INPUT'
[
  {"prompt": "QUESTION_0_USER_MSG", "system": "MERGED_SYSTEM"},
  {"prompt": "QUESTION_1_USER_MSG", "system": "MERGED_SYSTEM"},
  ...
]
JIMMY_INPUT
```

**System prompt construction (`MERGED_SYSTEM`):**

No caller `system` param:
```
"You are a validator. Answer YES or NO, then give one sentence of reasoning."
```

With caller `system` param:
```
"You are a validator. Answer YES or NO, then give one sentence of reasoning.\n{caller_system}"
```

**User message construction for each question (`QUESTION_I_USER_MSG`):**

For each question at index I:

If question is a plain string:
- If `context` param is provided: `user_message = "Context:\n{context}\n\nQuestion: {question}"`
- Else: `user_message = "Question: {question}"`

If question is an object `{ question, context? }`:
- If `object.context` is provided (overrides shared context): `user_message = "Context:\n{object.context}\n\nQuestion: {object.question}"`
- Else if `context` param is provided: `user_message = "Context:\n{context}\n\nQuestion: {object.question}"`
- Else: `user_message = "Question: {object.question}"`

Escape any `"` characters in prompt or system text as `\"` in the JSON. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` characters in the text.

Issue exactly ONE Bash tool call. The binary handles concurrency internally.

## Step 3: Collect results

Wait for the single Bash call to complete. The stdout is a JSON array. Parse it. Each item has shape `{ index, results: [{ response, tokens, elapsed_ms }] }`.

Extract the response text for question at index I:
- `parallel_output[I].results[0].response` — this is the raw text to parse in Step 4
- If `parallel_output[I].results[0].response` is null: treat it as an error item using `parallel_output[I].results[0].error` and `parallel_output[I].results[0].error_type`

## Step 4: Assemble output object

For each result at position I (0-based), apply the boolean parsing algorithm to the `response` field:

**Boolean parsing algorithm (two-pass):**

```
Given: raw = parallel_output[I].results[0].response

Pass 1 — Prefix check:
  Strip leading whitespace from raw.
  If raw starts with YES (case-insensitive): token = "YES", pass = true, explanation = trim(text after "YES")
  If raw starts with TRUE (case-insensitive): token = "TRUE", pass = true, explanation = trim(text after "TRUE")
  If raw starts with NO (case-insensitive): token = "NO", pass = false, explanation = trim(text after "NO")
  If raw starts with FALSE (case-insensitive): token = "FALSE", pass = false, explanation = trim(text after "FALSE")
  → If match found: construct success item (below) and continue to next question

Pass 2 — First-50-char scan:
  If no prefix match, take first 50 characters of raw.
  Find first occurrence of YES, TRUE, NO, or FALSE as a word/token (case-insensitive).
  → If found: extract token, map to pass value as above, explanation = trim(raw after the token position)
  → If not found: construct error item (below)

Token mapping:
  YES, TRUE → pass: true
  NO, FALSE → pass: false
  1, 0, or any other value → NOT accepted; treat as no-match
```

**Success item shape:**
```json
{ "index": I, "question": "{question_string}", "pass": true_or_false, "explanation": "...", "raw": "..." }
```
- `question` is the plain question string (use `object.question` if input was an object)
- `explanation` is the text after the YES/NO/TRUE/FALSE token, trimmed (may be empty string if none)
- `raw` is the full response text from `parallel_output[I].results[0].response`

**Error item shape (parse failure):**
```json
{ "index": I, "question": "{question_string}", "pass": null, "error": "Could not parse YES/NO from response", "error_type": "parse", "raw": "..." }
```

**Error item shape (API/network failure — results[0].response is null):**
```json
{ "index": I, "question": "{question_string}", "pass": null, "error": "jimmy-skill error: {results[0].error}", "error_type": "{results[0].error_type}", "raw": null }
```

**Summary computation:**
```
total = N (number of questions)
passed = count of items where pass === true
failed = count of items where pass === false
errors = count of items where pass === null
```

The results array must contain exactly N items regardless of failures.

## Step 5: Return

Output the JSON object only. No markdown fences, no commentary, no explanation — bare JSON starting with `{` and ending with `}`.

Example output for N=3:
```
{
  "results": [
    { "index": 0, "question": "Is the function pure?", "pass": true, "explanation": "it has no side effects.", "raw": "YES it has no side effects." },
    { "index": 1, "question": "Does it handle null?", "pass": false, "explanation": "there is no null check.", "raw": "NO there is no null check." },
    { "index": 2, "question": "Is it documented?", "pass": null, "error": "Could not parse YES/NO from response", "error_type": "parse", "raw": "It depends." }
  ],
  "summary": { "total": 3, "passed": 1, "failed": 1, "errors": 1 }
}
```

# Skill: jimmy-testdata

You are executing the jimmy-testdata skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for generating many fixtures in parallel.

This skill calls `jimmy-skill --parallel` with a JSON array of N fixture prompts (one per fixture) and returns all results as a `{ fixtures, summary }` object. Use it when you need:
- Unit test setup — structured objects that conform to a schema for test suites
- Database seeding — realistic fixture data for development environments
- API contract testing — request/response payloads matching a known schema
- Edge-case coverage — boundary and adversarial values per field for stress testing

NOT for: relational data with foreign-key consistency, deeply nested schemas (keep schemas to 5-10 fields), or exact controlled values you need to specify manually.

## Inputs

You will be provided:
- `schema`: object — maps field names to type strings. Must be non-empty. Supported types: `string`, `int`, `float`, `bool`, `email`, `url`, `date`, `uuid`.
- `n`: integer — number of fixtures to generate (must be >= 1; practical max ~20)
- `edge_case` (optional boolean, default `false`): when `true`, prompts Jimmy to generate boundary/adversarial values for each field based on its type
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests passed to `--max-concurrent`

## Step 1: Validate inputs

Validate all inputs before proceeding. Output bare JSON errors with no markdown fences and stop immediately on any failure.

- `schema` missing or not an object: output `{"error": "schema must be a non-empty object", "error_type": "usage"}` and stop.
- `schema` is an empty object (zero keys): output `{"error": "schema must be a non-empty object", "error_type": "usage"}` and stop.
- Any schema type value is not one of `string`, `int`, `float`, `bool`, `email`, `url`, `date`, `uuid`: output `{"error": "unsupported type '{TYPE}' for field '{FIELD}' — supported types: string, int, float, bool, email, url, date, uuid", "error_type": "usage"}` (use the actual type and field name) and stop.
- `n` missing or not a positive integer: output `{"error": "n must be a positive integer", "error_type": "usage"}` and stop.
- `max_concurrent` provided and less than 1: output `{"error": "max_concurrent must be >= 1", "error_type": "usage"}` and stop.

## Step 2: Construct the fixture prompts and call jimmy-skill --parallel

**System prompt (same for all items):**

```
You are a data generator. Generate exactly one JSON object. Output only the raw JSON — no markdown fences, no explanation, no trailing text.
```

**Normal mode fixture prompt (when `edge_case` is false or not provided):**

Build a per-fixture user message by iterating the schema fields:

```
Generate a single JSON object with these fields and types:
{field_name}: {type}
{field_name}: {type}
...

Return only the JSON object. No explanation, no markdown fences, no extra text.
```

**Edge-case mode fixture prompt (when `edge_case` is true, per D-07):**

Build a per-fixture user message with adversarial per-field instructions:

```
Generate a single JSON object with adversarial/boundary values for these fields:
{field_name} ({type}): use a boundary or adversarial value for this type
...

Type-specific guidance:
- string: empty string "", very long string (500+ chars), or string with special chars like \n " \t <>
- int: 0, -1, max safe integer (9007199254740991), or -9007199254740991
- float: 0.0, -0.0, very large (1e308), very small (1e-308), or NaN-adjacent
- bool: true and false both valid; prefer whichever is the less expected value
- email: invalid format (missing @, missing domain, spaces inside), empty string
- url: missing scheme, spaces in path, non-ASCII hostname
- date: "0000-01-01", "9999-12-31", invalid format like "31-13-2024", empty string
- uuid: all zeros, invalid format (missing hyphens), empty string

Return only the JSON object. No explanation, no markdown fences, no extra text.
```

All N fixture prompts are identical (same schema, same mode). Construct a JSON array of N items. Issue ONE Bash tool call using quoted heredoc:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT << 'JIMMY_INPUT'
[
  {"prompt": "FIXTURE_PROMPT", "system": "SYSTEM_PROMPT"},
  {"prompt": "FIXTURE_PROMPT", "system": "SYSTEM_PROMPT"},
  ...  (N items total, all identical)
]
JIMMY_INPUT
```

Replace `MAX_CONCURRENT` with the provided value or default 10. Escape any `"` in prompt or system text as `\"` to keep the JSON valid. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` characters in the prompt text. This is ONE Bash tool call.

**IMPORTANT (DATA-02):** Never batch multiple fixtures into one Jimmy prompt. One prompt per fixture — N fixtures = N array items = N parallel calls. This is mandatory — batching degrades JSON reliability with 8B models and a single bad call can lose all N fixtures instead of just one.

## Step 3: Collect results

Wait for the single Bash call to complete. Parse the JSON array output. Each item has shape `{ index, results: [{ response, tokens, elapsed_ms }] }`.

For each item at index I:
- `raw = parallel_output[I].results[0].response`
- If `raw` is null: API error — note `parallel_output[I].results[0].error` and `.error_type`. Proceed to Step 4 for error item construction.
- If `raw` is a string: attempt JSON parse. Proceed to Step 4.

## Step 4: Assemble output array

For each item at index I, construct one of the following shapes:

**API error (raw is null):**
```json
{
  "index": I,
  "fixture": null,
  "error": "jimmy-skill error: {results[0].error}",
  "error_type": "{results[0].error_type}",
  "raw": null
}
```

**Parse attempt (raw is a string):**

Attempt to parse `raw` as JSON. In practice: trim `raw`, check if it starts with `{`, and attempt JSON parsing.

**Parse succeeds — success fixture item:**
```json
{
  "index": I,
  "fixture": { ...parsed JSON object... }
}
```

**Parse fails (D-08) — no retry:**
```json
{
  "index": I,
  "fixture": null,
  "error": "invalid JSON",
  "error_type": "parse",
  "raw": "...Jimmy's raw response..."
}
```

**Summary computation:**
```
total = N
succeeded = count of items where fixture != null
failed = count of items where fixture == null (parse errors + API errors)
```

The `fixtures` array must contain exactly N items. Order matches the original index (0 to N-1).

## Step 5: Return

Output the JSON object only. No markdown fences, no commentary — bare JSON starting with `{` and ending with `}`.

Example output for `n=3`, `schema: { "username": "string", "age": "int", "active": "bool" }`, normal mode:

```
{
  "fixtures": [
    { "index": 0, "fixture": { "username": "alice", "age": 32, "active": true } },
    { "index": 1, "fixture": { "username": "bob", "age": 19, "active": false } },
    { "index": 2, "fixture": null, "error": "invalid JSON", "error_type": "parse", "raw": "Sure! Here is your object: {\"username\": \"carol\",..." }
  ],
  "summary": { "total": 3, "succeeded": 2, "failed": 1 }
}
```

# jimmy-testdata

Generate N synthetic test fixtures matching a user-defined schema — one Jimmy call per fixture, with optional edge-case mode for boundary and adversarial values.

## When to use this skill

Use jimmy-testdata when you need structured test data that conforms to a schema and want to generate many fixtures quickly. Good for:

- **Unit test setup** — "Give me 10 User objects with valid field values for my test suite"
- **Database seeding** — "Generate 20 Product records for a development database"
- **Edge-case coverage** — "Generate adversarial User inputs: empty names, max-int ages, invalid emails"
- **API contract testing** — "Generate 15 request payloads matching my request schema"

Not suited for: generating large nested object graphs (keep schemas shallow — 5-10 fields), generating exact values you need to control manually, or generating relational data with foreign-key consistency constraints.

Note on reliability: Llama 3.1 8B (Jimmy) can produce invalid JSON, especially for complex schemas. jimmy-testdata requests one fixture per call (not batched) to minimize blast radius — a single bad call loses one fixture, not all N. Expect a ~5-15% parse failure rate on complex schemas; re-run the skill or accept partial results.

## How parallelism works here

All N fixtures are generated in one `--parallel` call. Each array item contains one fixture prompt — Jimmy generates one JSON object per item. Parse failures produce error items; the skill never retries failed fixtures. See jimmy-testdata.md for full details.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `schema` | yes | Object mapping field names to types: `{ "field_name": "type" }`. Supported types: `string`, `int`, `float`, `bool`, `email`, `url`, `date`, `uuid`. |
| `n` | yes | Number of fixtures to generate (integer >= 1; practical max ~20). Each fixture is one Jimmy call. |
| `edge_case` | no | Boolean, default `false`. When `true`, prompts Jimmy to generate boundary/adversarial values for each field based on its type (e.g., empty strings, max integers, invalid emails). |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 10. |

## Output

JSON object with `fixtures` array and `summary`. Each fixture item has an `index` field. Successful items have `fixture` (the parsed JSON object). Failed items have `fixture: null`, `error`, `error_type`, and `raw` (Jimmy's raw response for debugging). When `edge_case` is `true`, the same output shape is returned but fixture values contain adversarial boundary values.

```json
{
  "fixtures": [
    { "index": 0, "fixture": { "username": "alice", "age": 32, "active": true } },
    { "index": 1, "fixture": { "username": "bob", "age": 19, "active": false } },
    { "index": 2, "fixture": null, "error": "invalid JSON", "error_type": "parse", "raw": "Sure! Here is your object: {\"username\": \"carol\",..." }
  ],
  "summary": { "total": 3, "succeeded": 2, "failed": 1 }
}
```

Per-item parse failures set `fixture: null`, `error`, and `error_type: "parse"` without affecting other items. `fixtures` always has exactly N items.

Note: No automatic retry on parse failure. Re-invoke the skill with a lower `n` if partial results are insufficient.

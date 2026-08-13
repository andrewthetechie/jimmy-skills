---
name: jimmy-testdata
description: Generate independent JSON test fixtures from a flat field schema with Jimmy, including boundary-value fixtures.
---

# Jimmy test data

Generate `n` independent JSON objects, then parse and validate each against the requested flat schema.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Use [the fixture protocol](references/fixtures.md) for prompting, schema checks, and output shaping.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `schema` | yes | — | Non-empty `{ field: type }` object with non-empty field names |
| `n` | yes | — | Positive fixture count |
| `edge_case` | no | false | Ask for boundary or malformed values while preserving JSON types |
| `max_concurrent` | no | 10 | Positive integer |

Supported types are `string`, `int`, `float`, `bool`, `email`, `url`, `date`, and `uuid`. Each fixture is independent; the invoking agent verifies relationships between objects.

## Process

1. **Validate.** Enforce every input contract and reject unsupported types with a bare `usage` error object. Stop before calling Jimmy.
2. **Batch.** Build one prompt per fixture using the fixture protocol. JSON-serialize exactly `n` items and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
3. **Verify.** Parse and schema-check every response using the fixture protocol. Preserve raw text and record parse/schema failures as final.
4. **Return.** Emit only `{ "fixtures", "summary" }`. Completion requires `fixtures.length = n` and `summary.succeeded + summary.failed = n`.

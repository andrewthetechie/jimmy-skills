---
name: jimmy-testdata
description: Use when you need synthetic JSON fixtures from a field schema, parallel test data generation, database seed objects, or edge-case boundary values for stress testing.
---

# jimmy-testdata

Generate **N** JSON fixtures matching a schema (one Jimmy call per fixture).

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Prompts + parse: [fixtures](references/fixtures.md).

## Not for

Relational FK consistency, deeply nested schemas (>~10 fields), or hand-specified exact values.

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `schema` | yes | — | `{ field: type }` — types: string, int, float, bool, email, url, date, uuid |
| `n` | yes | — | Fixture count |
| `edge_case` | no | false | Boundary/adversarial values |
| `max_concurrent` | no | 10 | |

## Steps

1. **Validate** — empty schema / unsupported type / bad `n` → usage error. Stop.
2. **Build prompts** — [fixtures](references/fixtures.md). **Never** batch multiple fixtures into one prompt — N items in the array.
3. **Call once** — N identical items; `--max-iterations 1` explicitly.
4. **Parse** — trim; parse JSON object; no retry on parse fail.
5. **Return** — bare JSON `{ "fixtures": [...], "summary": { "total", "succeeded", "failed" } }` only.

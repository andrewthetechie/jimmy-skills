---
name: jimmy-extract
description: Extract a validated flat JSON object from each supplied text with repeated Jimmy attempts and consensus. Use for bulk entity, metadata, field, or record extraction.
---

# Jimmy extract

Generate repeated structured extractions per input, reject malformed attempts, and select the modal valid object.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Follow [the extraction protocol](references/extraction.md) for prompting, schema validation, consensus, and output shaping.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `items` | yes | — | Non-empty strings or `{ id, text }` objects; values and IDs are strings; IDs are unique |
| `schema` | yes | — | Non-empty flat `{ field: type }` object |
| `attempts` | no | 3 | Positive attempts per item; an odd value reduces ties |
| `instruction` | no | — | Domain-specific extraction guidance |
| `system` | no | — | Appended to the extractor system instruction |
| `max_concurrent` | no | 50 | Positive integer |

Supported types are `string`, `int`, `float`, `bool`, `string[]`, `int[]`, `float[]`, and `bool[]`; `null` represents a missing value for any field.

## Process

1. **Validate.** Enforce every input contract and supported schema type. Return a bare `usage` error object before calling Jimmy on the first invalid input.
2. **Matrix.** Build exactly `items.length × attempts` prompt items in input then attempt order.
3. **Batch.** JSON-serialize the matrix and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
4. **Consensus.** Strictly parse, schema-check, canonicalize, and select an extraction for each input using the extraction protocol.
5. **Return.** Emit only `{ "extractions", "summary" }`. Completion requires one extraction record per input and attempt/error counts that reconcile to the matrix size.

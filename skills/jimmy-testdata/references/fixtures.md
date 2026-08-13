# Fixture protocol

## Prompt

Use this system instruction for every item:

```text
You are a data generator. Generate and output exactly one raw JSON object.
```

The user prompt lists every `field: type` pair and requests exactly those fields. In normal mode, ask for representative values. In `edge_case` mode, ask for boundary or malformed values while preserving the declared JSON type:

- `string`: empty, very long, or special characters
- `int`: zero, negative one, or safe-integer boundaries
- `float`: zero or finite extremes
- `bool`: the less expected value
- `email`, `url`, `date`, `uuid`: malformed or empty strings

## Parse and schema-check

Parse the complete trimmed response as one JSON object. Require exactly the schema's key set, then check JSON types:

| Declared type | Required JSON value |
|---|---|
| `string`, `email`, `url`, `date`, `uuid` | string |
| `int` | number with no fractional component |
| `float` | finite number |
| `bool` | boolean |

Content formats are generation hints, not validators; this permits intentionally malformed strings in edge-case mode.

## Shape

- Success: `{ "index", "fixture": object }`
- JSON parse failure: `{ "index", "fixture": null, "error": "invalid JSON", "error_type": "parse", "raw" }`
- Key/type mismatch: `{ "index", "fixture": null, "error": "schema mismatch: REASON", "error_type": "validation", "raw" }`
- API failure: `{ "index", "fixture": null, "error", "error_type", "raw": null }`

Do not retry or salvage failures. Summary is `{ "total", "succeeded", "failed" }`; completion requires `succeeded + failed = total`.

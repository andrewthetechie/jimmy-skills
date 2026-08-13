# Fixture prompts + parse (jimmy-testdata)

## System (all items)

```
You are a data generator. Generate exactly one JSON object. Output only the raw JSON — no markdown fences, no explanation, no trailing text.
```

## Normal user prompt

List each `field: type`, ask for one JSON object only.

## Edge-case user prompt (`edge_case: true`)

Ask for adversarial/boundary values. Guidance:

- string: empty, very long, special chars
- int: 0, -1, ±max safe integer
- float: 0, extremes
- bool: less expected value
- email/url/date/uuid: invalid or empty forms

## Parse

`raw = results[0].response`. Null → API error item. Else trim + JSON parse starting with `{`.

Success: `{ index, fixture: <object> }`  
Parse fail (no retry): `{ index, fixture: null, error: "invalid JSON", error_type: "parse", raw }`  
API fail: `{ index, fixture: null, error, error_type, raw: null }`

`summary`: total=N, succeeded = fixture≠null, failed = fixture==null.

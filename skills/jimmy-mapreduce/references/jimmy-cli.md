# `jimmy-skill` batch contract

`jimmy-skill` must be installed on `PATH`. Parallel mode accepts a JSON array of `{ "prompt": string, "system"?: string }` objects on stdin and returns one ordered output item per input item.

## Serialize, invoke, verify

1. **Serialize.** Encode the complete input array with a JSON-aware serializer. Preserve prompts and system instructions as string values, including quotes, backslashes, newlines, and control characters. A quoted heredoc supplies shell-safe transport; JSON encoding happens first.
2. **Invoke.** Make one batch call after the owning skill has validated positive concurrency and iteration values:

   ```bash
   jimmy-skill --parallel --max-concurrent MAX --max-iterations ITERATIONS <<'JIMMY_INPUT'
   SERIALIZED_JSON_ARRAY
   JIMMY_INPUT
   ```

   Put shared system text on each JSON item when practical. The `--system` flag is also supported, and a per-item `system` overrides it.
3. **Verify.** Parse stdout as JSON. On a nonzero exit, top-level error object, malformed JSON, or output shape/cardinality mismatch, return a bare error object immediately.

## Successful output

The outer array preserves input order and `index`. Each item has exactly `ITERATIONS` result objects:

```json
[
  {
    "index": 0,
    "results": [
      {
        "response": "...",
        "tokens": { "prompt": 1, "completion": 1, "total": 2 },
        "elapsed_ms": 400
      }
    ]
  }
]
```

An individual request failure stays in place with `response: null`, zero token counts, `elapsed_ms`, `error`, and `error_type` (`timeout`, `network`, `api`, `parse`, or `usage`). Other items continue.

Batch verification is complete when the outer length equals the input length, every `index` equals its input position, and every `results.length` equals `ITERATIONS`.

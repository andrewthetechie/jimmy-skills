# Inert payload protocol

Normalize a category for lookup by lowercasing and replacing `-` with `_`. Preserve the caller's original category in output.

## Prompt each item

Base system instruction:

```text
You are a security payload generator. Generate exactly one {CATEGORY} payload. Output exactly the raw payload string.
```

Append the matching specialization when known:

| Normalized category | Specialization |
|---|---|
| `xss` | Cross-site scripting string targeting a browser HTML context. |
| `sqli` | SQL injection string targeting a standard SQL `WHERE` clause. |
| `path_traversal` | Path traversal sequence for Unix or Windows. |
| `command_injection` | Shell metacharacters representing command injection. |
| `xxe` | XML External Entity injection fragment. |
| `ssrf` | URL or parameter representing access to an internal resource. |
| `open_redirect` | URL or parameter representing an open redirect. |
| `csrf` | Token or header manipulation string representing CSRF. |

Use this user prompt:

```text
Attack surface: {ATTACK_SURFACE}

Generate one {CATEGORY} payload for this attack surface.
```

Build all `attack_types.length × n` prompt/system pairs before the single Jimmy batch call. Payload text appears only after that call and remains in memory; no later shell operation may contain it.

## Shape

Severity is `critical` for `sqli` and `command_injection`; `high` for `xss`, `path_traversal`, `xxe`, and `ssrf`; `medium` otherwise.

- Success: `{ "index", "category", "payload", "severity", "tokens", "elapsed_ms" }`, with `payload` trimmed.
- Failure: the same identity fields with `payload: null`, `error`, and `error_type`.
- `summary`: `{ "total", "by_category", "api_errors" }`.
- `warning`: state that payloads are inert strings, were not executed by the agent, and require authorized human-controlled testing.

Assembly is complete when global indices are contiguous and every summary count reconciles to the payload array.

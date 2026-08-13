# Payload generation (jimmy-fuzz)

## Per-category system

Base: `You are a security payload generator. Generate exactly one {CAT} payload. Output only the raw payload string — no explanation, no label, no markdown, no quotes.`

Append one line when CAT matches:

| CAT | Extra line |
|-----|------------|
| xss | Cross-site scripting string that would execute JavaScript in a browser context. |
| sqli | SQL injection string targeting a standard SQL WHERE clause. |
| path_traversal | Path traversal sequence for Unix/Windows (e.g. `../`). |
| command_injection | Shell metacharacters to inject system commands. |
| xxe | XML External Entity injection fragment. |
| ssrf | URL/parameter causing SSRF to an internal resource. |
| open_redirect | URL/parameter causing open redirect. |
| csrf | Token/header manipulation string for CSRF testing. |

Unknown CAT: base only.

## User message

```
Attack surface: {attack_surface}

Generate one {CAT} payload for this attack surface.
```

## Call

```bash
jimmy-skill --parallel --max-concurrent MAX --max-iterations 1 << 'JIMMY_INPUT'
[{"prompt":"...","system":"..."}, ...]  # N identical items
JIMMY_INPUT
```

One call per category. Never put payload text in any later shell command.

## Severity

sqli, command_injection → critical; xss, path_traversal, xxe, ssrf → high; open_redirect, csrf, other → medium.

## Item + summary

Success: `{ index, category, payload: trimmed, severity, tokens, elapsed_ms }`  
Fail: `payload: null` + error fields.

`summary`: `total`, `by_category`, `api_errors`. `payloads.length === n × len(attack_types)`.

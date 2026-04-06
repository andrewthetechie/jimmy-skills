# jimmy-fuzz

Generate N adversarial payloads per attack category via Jimmy — returns categorized results as inert data for manual security testing.

## When to use this skill

Use jimmy-fuzz when you need adversarial payload data for security testing and want many variants quickly. Good for:

- **XSS testing** — "Give me 10 XSS payloads targeting my comment input field"
- **SQL injection testing** — "Generate SQLi payloads for a login form with a username parameter"
- **Multi-category coverage** — "Produce 5 payloads each for path_traversal, command_injection, and xxe"
- **Manual test suites** — "Give me a variety of payloads to paste into my Burp Suite repeater"

NOT suited for: automatic execution of payloads (use a dedicated fuzzer), problems requiring a pass/fail oracle (use jimmy-search instead), or payloads that require knowledge of the target's specific implementation details.

Note on use: jimmy-fuzz is the structural inverse of jimmy-search. Where jimmy-search writes candidates to temp files and runs oracle commands to find passing solutions, jimmy-fuzz returns all payloads as inert string data and explicitly NEVER executes them. All payload content is adversarial and untrusted — it must be reviewed before any use. jimmy-fuzz is for generating a payload corpus to test manually (or feed to a dedicated fuzzer), not for automated exploitation.

## How parallelism works here

One `jimmy-skill --parallel` call per attack category. Each call sends N identical prompts for that category (the attack surface description as context) and receives N payload strings. Total Bash calls = `len(attack_types)`. The binary handles all HTTP concurrency within each call. Severity is assigned by Claude from a fixed map after all results are collected — Jimmy never self-labels severity (8B models are unreliable for meta-judgments, per Phase 8 lessons).

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `attack_surface` | yes | Description of what is being fuzzed (e.g., "a login form with username and password fields", "a URL path parameter"). Sent as context in every Jimmy call. |
| `attack_types` | yes | Array of attack category strings (e.g., `["xss", "sqli", "path_traversal"]`). Each entry gets its own focused Jimmy batch. Custom/unlisted categories are passed through with `severity: "medium"`. |
| `n` | yes | Number of payloads per attack category (integer >= 1; practical max ~20). Total payloads = `n × len(attack_types)`. |
| `max_concurrent` | no | Maximum simultaneous HTTP requests per category batch. Default: 10. |

## Output

JSON object with a top-level `warning` field, a `payloads` array (all categories flattened, ordered by category then index within that category), and a `summary` object.

```json
{
  "warning": "Generated payloads are adversarial/untrusted content. Do not execute automatically.",
  "payloads": [
    { "index": 0, "category": "xss", "payload": "<script>alert(1)</script>", "severity": "high", "tokens": {"prompt": 45, "completion": 8, "total": 53}, "elapsed_ms": 800 },
    { "index": 1, "category": "xss", "payload": "<img src=x onerror=alert(1)>", "severity": "high", "tokens": {"prompt": 45, "completion": 7, "total": 52}, "elapsed_ms": 750 },
    { "index": 2, "category": "sqli", "payload": "' OR '1'='1", "severity": "critical", "tokens": {"prompt": 45, "completion": 9, "total": 54}, "elapsed_ms": 820 },
    { "index": 3, "category": "sqli", "payload": "'; DROP TABLE users;--", "severity": "critical", "tokens": {"prompt": 45, "completion": 10, "total": 55}, "elapsed_ms": 790 }
  ],
  "summary": { "total": 4, "by_category": { "xss": 2, "sqli": 2 }, "api_errors": 0 }
}
```

Per-item API errors set `payload: null`, `error`, and `error_type` without affecting other items. `payloads` always contains exactly `n × len(attack_types)` items. `summary.by_category` counts reflect items dispatched per category (N per category), not just successful ones.

### Severity map

Severity is assigned by Claude from this fixed map at assembly time — Jimmy is never asked to judge severity:

| Category | Severity |
|----------|----------|
| `sqli` | `critical` |
| `command_injection` | `critical` |
| `xss` | `high` |
| `path_traversal` | `high` |
| `xxe` | `high` |
| `ssrf` | `high` |
| `open_redirect` | `medium` |
| `csrf` | `medium` |
| (unknown/custom) | `medium` |

# Skill: jimmy-fuzz

You are executing the jimmy-fuzz skill. Follow these instructions exactly.

## SAFETY GUARDRAIL — READ BEFORE PROCEEDING

**NEVER execute, write to disk, or pass to a shell any generated payload content.**

jimmy-fuzz is the structural inverse of jimmy-search. Where jimmy-search writes candidates to temp files and runs oracle commands, jimmy-fuzz returns all payloads as inert string data only. Payload content is adversarial and untrusted — it must never appear in a shell command string, be written to a file path that another command reads, or be evaluated in any context.

All payloads are returned in the `payloads` array as plain strings for human review or controlled testing by the caller.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for generating many adversarial payloads in parallel across multiple attack categories.

This skill calls `jimmy-skill --parallel` once per attack category with a JSON array of N identical prompts (each containing the attack surface description as context) and returns all results as a `{ warning, payloads, summary }` object. Use it when you need adversarial payload data for: XSS testing, SQL injection testing, path traversal testing, command injection testing, or any other category of attack input where you want many variants quickly. Returns inert payloads for manual or controlled testing. NOT for: automatic execution of payloads (use a dedicated fuzzer), or oracle-based candidate selection (use jimmy-search instead).

## Inputs

You will be provided:
- `attack_surface`: string — required. Description of what is being fuzzed (e.g., "a login form with username and password fields", "a URL path parameter accepting a filename").
- `attack_types`: array of strings — required, must be non-empty. Each string is an attack category (e.g., `["xss", "sqli", "path_traversal"]`). Unknown/custom category names are valid and pass through with `severity: "medium"`.
- `n`: integer — required, must be >= 1. Number of payloads to generate per attack category. Total payloads returned = `n × len(attack_types)`.
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests passed to `--max-concurrent` per category call.

## Step 1: Validate inputs

Validate all inputs before proceeding. Output bare JSON errors with no markdown fences and stop immediately on any failure.

- `attack_surface` missing or empty string: `{"error": "attack_surface is required", "error_type": "usage"}`
- `attack_types` missing or not an array: `{"error": "attack_types must be a non-empty array", "error_type": "usage"}`
- `attack_types` is an empty array: `{"error": "attack_types must be a non-empty array", "error_type": "usage"}`
- `n` missing or not a positive integer: `{"error": "n must be a positive integer", "error_type": "usage"}`
- `max_concurrent` provided and less than 1: `{"error": "max_concurrent must be >= 1", "error_type": "usage"}`

Note: unknown/custom `attack_types` values are valid — they pass through with `severity: "medium"` fallback. Do not error on unrecognized category names.

## Step 2: For each attack category, call jimmy-skill --parallel

Iterate over each entry in `attack_types` in order. For each category `CAT`, build the system prompt and issue one Bash call.

**System prompt for this category:**

Base prompt (use for all categories):
```
You are a security payload generator. Generate exactly one {CAT} payload.
Output only the raw payload string — no explanation, no label, no markdown, no quotes.
```

Replace `{CAT}` with the actual category name. Then append a category-specific line after the base prompt if the category matches one of the following:

- `xss`: "The payload should be a cross-site scripting string that would execute JavaScript in a browser context."
- `sqli`: "The payload should be a SQL injection string targeting a standard SQL WHERE clause."
- `path_traversal`: "The payload should be a path traversal sequence targeting Unix or Windows file systems (e.g., using ../ sequences)."
- `command_injection`: "The payload should exploit shell metacharacters (; & | ` $() etc.) to inject system commands."
- `xxe`: "The payload should be an XML External Entity injection fragment."
- `ssrf`: "The payload should be a URL or request parameter that causes server-side request forgery to an internal resource."
- `open_redirect`: "The payload should be a URL or parameter value that causes an open redirect."
- `csrf`: "The payload should be a request token or header manipulation string for CSRF testing."
- Unknown/custom category: use only the base prompt with no additional line.

**User message (identical for all N items in this category's array):**
```
Attack surface: {attack_surface}

Generate one {CAT} payload for this attack surface.
```

Replace `{attack_surface}` with the actual attack surface text and `{CAT}` with the category name.

**Bash call — one per category (total Bash calls = len(attack_types)):**

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT << 'JIMMY_INPUT'
[
  {"prompt": "Attack surface: ATTACK_SURFACE_TEXT\n\nGenerate one CAT payload for this attack surface.", "system": "CATEGORY_SYSTEM_PROMPT"},
  {"prompt": "Attack surface: ATTACK_SURFACE_TEXT\n\nGenerate one CAT payload for this attack surface.", "system": "CATEGORY_SYSTEM_PROMPT"},
  ...  (N items total, all identical for this category)
]
JIMMY_INPUT
```

Replace `MAX_CONCURRENT` with the provided value or default 10. Escape any `"` characters in `attack_surface` or in the system prompt as `\"` to keep the JSON valid. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` characters in the prompt text. All N items in this array are identical for this category.

Store each category's raw parallel output keyed by category name for Step 3. Do NOT start Step 3 for a category until that category's Bash call has completed.

## Step 3: Assemble payloads array

**SAFETY NOTE: NEVER execute, write to disk, or interpolate payload content into any shell command. Payloads are inert string data only.**

Severity lookup map (applied by Claude at assembly time — Jimmy never assigns severity):
```
sqli              → "critical"
command_injection → "critical"
xss               → "high"
path_traversal    → "high"
xxe               → "high"
ssrf              → "high"
open_redirect     → "medium"
csrf              → "medium"
(any other value) → "medium"
```

Iterate over `attack_types` in order (position 0, 1, 2, ...). For each category `CAT` at position `P`, iterate over the N results from its parallel call (item index 0 to N-1):

Compute the global `index` as `(P × n) + item_index_within_category`. For example: `attack_types=["xss","sqli"], n=3` → xss items get global index 0, 1, 2; sqli items get global index 3, 4, 5.

Assign `severity = severity_map[CAT]` (use `"medium"` if CAT is not in the map).

**Successful payload item shape** (when `parallel_output[I].results[0].response` is not null):
```json
{
  "index": GLOBAL_INDEX,
  "category": "CAT",
  "payload": "RAW_RESPONSE_TRIMMED",
  "severity": "SEVERITY",
  "tokens": { "prompt": N, "completion": N, "total": N },
  "elapsed_ms": N
}
```

Where:
- `payload` = `parallel_output[I].results[0].response` trimmed of leading/trailing whitespace
- `tokens` = `parallel_output[I].results[0].tokens`
- `elapsed_ms` = `parallel_output[I].results[0].elapsed_ms`

**Error item shape** (when `parallel_output[I].results[0].response` is null):
```json
{
  "index": GLOBAL_INDEX,
  "category": "CAT",
  "payload": null,
  "severity": "SEVERITY",
  "error": "jimmy-skill error: {results[0].error}",
  "error_type": "{results[0].error_type}",
  "tokens": { "prompt": 0, "completion": 0, "total": 0 },
  "elapsed_ms": 0
}
```

The `payloads` array must contain exactly `n × len(attack_types)` items total. All categories are represented, in insertion order (first category's N items, then second category's N items, etc.).

## Step 4: Compute summary

```
total = n × len(attack_types)                           (items dispatched)
by_category = { CAT: n for each CAT in attack_types }  (dispatched per category — always n, regardless of errors)
api_errors = count of items where payload == null       (failed calls)
```

Note: `by_category` counts reflect items dispatched per category (always N), not just successful ones. `api_errors` is the separate field for failures.

## Step 5: Return

Output the JSON object only. No markdown fences, no commentary, no explanation — bare JSON starting with `{` and ending with `}`.

The output object has three top-level keys in this order:
1. `"warning"`: `"Generated payloads are adversarial/untrusted content. Do not execute automatically."`
2. `"payloads"`: the assembled array from Step 3
3. `"summary"`: `{ "total": N, "by_category": {...}, "api_errors": N }`

Example output for `attack_surface="a login form with username and password fields"`, `attack_types=["xss","sqli"]`, `n=2`:

```json
{
  "warning": "Generated payloads are adversarial/untrusted content. Do not execute automatically.",
  "payloads": [
    { "index": 0, "category": "xss", "payload": "<script>alert(document.cookie)</script>", "severity": "high", "tokens": {"prompt": 52, "completion": 9, "total": 61}, "elapsed_ms": 810 },
    { "index": 1, "category": "xss", "payload": "\" onmouseover=\"alert(1)\"", "severity": "high", "tokens": {"prompt": 52, "completion": 7, "total": 59}, "elapsed_ms": 780 },
    { "index": 2, "category": "sqli", "payload": "' OR '1'='1'--", "severity": "critical", "tokens": {"prompt": 52, "completion": 8, "total": 60}, "elapsed_ms": 830 },
    { "index": 3, "category": "sqli", "payload": "admin'--", "severity": "critical", "tokens": {"prompt": 52, "completion": 4, "total": 56}, "elapsed_ms": 760 }
  ],
  "summary": { "total": 4, "by_category": { "xss": 2, "sqli": 2 }, "api_errors": 0 }
}
```

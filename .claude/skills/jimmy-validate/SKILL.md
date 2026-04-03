# jimmy-validate

Fan out N yes/no questions to Jimmy in parallel and get back a boolean result set with summary aggregation.

## When to use this skill

Use jimmy-validate when you need to check multiple boolean properties of something quickly and cheaply. Good for:

- **Code review** — "Does this function handle null?", "Is this input sanitized?", "Are all exports documented?"
- **Content validation** — "Is this copy within tone guidelines?", "Does the summary cover the main point?"
- **Structured QA** — Validate a list of requirements against an implementation
- **Cheap pre-screening** — Run N boolean checks before spending Claude tokens on deeper analysis

Not suited for: questions requiring multi-step reasoning, comparisons between options, or answers longer than one sentence.

## How parallelism works here

`jimmy-skill --parallel` handles all concurrency internally. The skill constructs one JSON item per question and pipes the array in a single Bash call. See jimmy-validate.md for full details.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `questions` | yes | Array of yes/no questions. Each item is a string OR `{ question: string, context?: string }` for per-question context override. |
| `context` | no | Shared context string applied to every question (e.g., "The following code is a Python function that processes user input"). Placed in the user message, not the system prompt. Per-question context in an object item overrides this. |
| `system` | no | Optional role/style instructions merged with the skill default. Keep short — this is for tone/domain guidance only, not reference material. Max ~500 chars recommended. |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 10. |
| `max_iterations` | no | How many times each question is sent to Jimmy per item. Default: 1. For validate, leave at 1 — only results[0] is used for boolean parsing. |

## Output

JSON object with `results` array and `summary` aggregation:

```json
{
  "results": [
    { "index": 0, "question": "Is the function pure?", "pass": true, "explanation": "it has no side effects.", "raw": "YES it has no side effects." },
    { "index": 1, "question": "Does it handle null?", "pass": false, "explanation": "there is no null check.", "raw": "NO there is no null check." },
    { "index": 2, "question": "Is it documented?", "pass": null, "error": "Could not parse YES/NO from response", "error_type": "parse", "raw": "It depends." }
  ],
  "summary": { "total": 3, "passed": 1, "failed": 1, "errors": 1 }
}
```

Per-item parse failures set `pass: null`, `error`, and `error_type: "parse"` without affecting other items. Results always has exactly N items. Summary counts: `passed` = items where `pass === true`, `failed` = items where `pass === false`, `errors` = items where `pass === null`.

**Note on the `system` param:** The `system` param is for style instructions only — keep it short. Use question-level `context` for reference material to avoid hitting Jimmy's 28K system prompt cap.

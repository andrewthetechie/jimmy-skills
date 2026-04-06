# jimmy-search

Generate N candidate solutions via Jimmy and test each against user-provided shell commands — returns candidates ranked by test pass rate.

## When to use this skill

Use jimmy-search when you have a problem that admits a testable oracle and benefits from generating many candidate solutions cheaply. Good for:

- **Regex generation** — "Find a regex that matches all my valid inputs but rejects these invalid ones"
- **SQL query generation** — "Generate a query that passes my correctness checks"
- **Shell command generation** — "Find a one-liner that processes this file correctly"
- **Custom solution search** — Any deterministic problem where you can describe pass/fail via a shell exit code

Not suited for: problems without a computable oracle (subjective quality judgments), problems where correctness cannot be expressed as a shell exit code, or problems requiring structured multi-step reasoning from Jimmy.

**Portability note:** Test commands must be POSIX-compatible. Use `grep -E` (extended regex, POSIX) not `grep -P` (Perl regex, GNU only — not available on macOS).

## How parallelism works here

`jimmy-skill --parallel` generates all N candidates in one call. The skill constructs a JSON array of N identical prompt objects and pipes it to the binary in a single Bash call. The binary fans out HTTP requests up to `max_concurrent` at a time and returns an ordered JSON array. Oracle execution (write-test-record) runs sequentially per candidate after generation completes. See jimmy-search.md for full details.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `problem` | yes | Problem description. Sent as the user message to every Jimmy call. Be specific about what the solution should do. |
| `test_cases` | yes | Array of `{ "command": "<shell command with $CANDIDATE_FILE>", "expected_exit": 0 }` objects. Each candidate is tested against all test cases. Pass = command exits with `expected_exit`. |
| `n` | yes | Number of candidates to generate (integer >= 1; practical max ~20). |
| `type` | no | Hint for candidate type: `"regex"`, `"sql"`, `"shell"`, or omit for generic. Specializes Jimmy's system prompt for cleaner output. |
| `system` | no | Additional system prompt instructions appended after the skill default. Keep short — use `problem` for reference material. |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 10. |

## Output

JSON object with `candidates` array (sorted descending by `pass_rate`) and `summary`:

```json
{
  "candidates": [
    {
      "index": 0,
      "response": "^[a-z]+$",
      "pass_rate": 1.0,
      "tests_passed": 2,
      "tests_total": 2,
      "test_results": [
        { "command": "grep -qE '^[a-z]+$' /tmp/jimmy_candidate_0", "expected_exit": 0, "exit_code": 0, "pass": true },
        { "command": "bash -c 'wc -l < /tmp/jimmy_candidate_0 | grep -q ^1$'", "expected_exit": 0, "exit_code": 0, "pass": true }
      ],
      "tokens": { "prompt": 45, "completion": 8, "total": 53 },
      "elapsed_ms": 812
    },
    {
      "index": 2,
      "response": null,
      "pass_rate": null,
      "tests_passed": 0,
      "tests_total": 0,
      "test_results": [],
      "error": "jimmy-skill error: Request timed out",
      "error_type": "timeout"
    }
  ],
  "summary": {
    "total_candidates": 5,
    "candidates_tested": 5,
    "api_errors": 0,
    "perfect_pass_rate": 2,
    "any_passing": 3
  }
}
```

`test_results[].command` in the output uses the literal temp file path (e.g., `/tmp/jimmy_candidate_0`), NOT the `$CANDIDATE_FILE` placeholder.

Per-item API errors set `response: null`, `pass_rate: null`, and `test_results: []` without affecting other candidates. `candidates` always has exactly N items. API error items sort last (treated as `pass_rate: -1` for ordering).

`summary` fields:
- `total_candidates`: N (total generated)
- `candidates_tested`: N minus api_errors
- `api_errors`: count of candidates where response is null
- `perfect_pass_rate`: count of candidates where pass_rate == 1.0
- `any_passing`: count of candidates where pass_rate > 0 (and pass_rate is not null)

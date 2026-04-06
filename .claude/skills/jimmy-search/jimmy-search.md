# Skill: jimmy-search

You are executing the jimmy-search skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for generating many candidate solutions and testing each against a deterministic oracle.

This skill generates N candidate solutions in parallel via Jimmy, writes each candidate to a temp file, runs user-provided shell test commands against that file, and returns candidates ranked by test pass rate. Use it when you need:
- Regex generation — generate candidates, test each against match/reject cases
- SQL query generation — generate queries, test each against a correctness check
- Shell command generation — generate one-liners, test each against expected output
- Any problem with a testable oracle — if you can express pass/fail as a shell exit code, jimmy-search can search for a solution

NOT for: subjective evaluation, problems without a shell-testable oracle, or problems requiring multi-step reasoning from Jimmy.

## Inputs

You will be provided:
- `problem`: string — problem description sent as the user message to every Jimmy call
- `test_cases`: array of `{ "command": "<shell command with $CANDIDATE_FILE>", "expected_exit": 0 }` objects — must be non-empty
- `n`: integer — number of candidates to generate (must be >= 1; practical max ~20)
- `type` (optional): `"regex"`, `"sql"`, `"shell"`, or omit for generic — specializes Jimmy's system prompt
- `system` (optional): string — additional instructions appended after the skill default system prompt
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests

## Step 1: Validate inputs

Validate all inputs before proceeding. Output these error responses as bare JSON with no markdown fences and stop immediately on any failure.

- `problem` missing or empty string: `{"error": "problem is required", "error_type": "usage"}`
- `test_cases` missing or not an array: `{"error": "test_cases must be a non-empty array", "error_type": "usage"}`
- `test_cases` is an empty array: `{"error": "test_cases must be a non-empty array", "error_type": "usage"}`
- Any test_case at index N is missing the `command` field: `{"error": "test_case at index N is missing command", "error_type": "usage"}` (use the actual index)
- `n` missing or not a positive integer: `{"error": "n must be a positive integer", "error_type": "usage"}`
- `max_concurrent` provided and less than 1: `{"error": "max_concurrent must be >= 1", "error_type": "usage"}`

## Step 2: Construct the JSON array and call jimmy-skill --parallel

**Build the system prompt based on `type`:**

- `type: "regex"`: `"You are a solution generator. Output only the regex pattern — no delimiters, no flags, no explanation."`
- `type: "sql"`: `"You are a solution generator. Output only the SQL query — no backticks, no explanation."`
- `type: "shell"`: `"You are a solution generator. Output only the shell command — no explanation."`
- No `type` or unknown value: `"You are a solution generator. Generate exactly one solution that fits the problem description. Output only the solution — no explanation, no preamble, no markdown fences, no commentary."`

If `system` is provided, append it with a newline after the default: `"{default_system}\n{system}"`

**Construct N identical items** using `problem` as the user message. Issue ONE Bash tool call using a quoted heredoc:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT << 'JIMMY_INPUT'
[
  {"prompt": "PROBLEM_TEXT", "system": "SYSTEM_PROMPT"},
  {"prompt": "PROBLEM_TEXT", "system": "SYSTEM_PROMPT"},
  ...  (N items total, all identical)
]
JIMMY_INPUT
```

Replace `MAX_CONCURRENT` with the provided value or default 10. Escape any `"` in `problem` or `system` text as `\"` to keep the JSON valid. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` in the problem text.

The binary handles all concurrency internally. This is ONE Bash tool call.

## Step 3: Oracle execution — write-test-record per candidate

Wait for the single Bash call to complete. Parse the JSON array output.

For each candidate at index I (0-based):

1. Extract candidate text: `parallel_output[I].results[0].response`
2. If `response` is null: this is an API error candidate. Skip oracle execution. Record error item (see Step 4 error shape). Continue to next candidate.
3. Write candidate text to a temp file using `printf '%s'` (NEVER `echo` — `echo` may interpret escape sequences). Issue a Bash tool call:
   ```
   printf '%s' "CANDIDATE_TEXT" > /tmp/jimmy_candidate_I
   ```
   Replace `CANDIDATE_TEXT` with the actual response text and `I` with the candidate index.
4. For each test case T at index J in `test_cases`:
   - Perform **textual replacement**: take `T.command` and replace the string `$CANDIDATE_FILE` with the literal path `/tmp/jimmy_candidate_I` (where I is the candidate index). This replacement is done by Claude — the shell never sees `$CANDIDATE_FILE` as a variable. Example: `"grep -qE '^[a-z]+$' $CANDIDATE_FILE"` becomes `"grep -qE '^[a-z]+$' /tmp/jimmy_candidate_0"`.
   - Issue a Bash tool call: `bash -c "SUBSTITUTED_CMD"` where `SUBSTITUTED_CMD` is the command with the literal path substituted in.
   - Record exit code from the Bash call result.
   - `pass = (exit_code == T.expected_exit)`
   - Record: `{ "command": SUBSTITUTED_CMD, "expected_exit": T.expected_exit, "exit_code": exit_code, "pass": pass }`
5. `tests_passed = count of test_results where pass == true`
6. `pass_rate = tests_passed / len(test_cases)` (float, 0.0 to 1.0)
7. Assemble candidate result object (see Step 4 success shape).

**CRITICAL SECURITY NOTE:** Jimmy's output is untrusted. It is ONLY written to a temp file and referenced by file path. It is NEVER interpolated into any shell command string. The `$CANDIDATE_FILE` placeholder is always replaced by Claude with a safe literal path before any Bash call — the shell never sees `$CANDIDATE_FILE` as a variable. If a candidate contains `;`, `&&`, `$(...)`, or any shell metacharacters, they are inert inside the file — only the safe literal path `/tmp/jimmy_candidate_I` appears in the executed command.

## Step 4: Assemble ranked output

Construct the output object with `candidates` sorted descending by `pass_rate`. API error items (where `pass_rate` is null) sort last — treat `pass_rate: null` as -1 for sorting. Include ALL candidates regardless of pass_rate.

**Success candidate shape:**
```json
{
  "index": 0,
  "response": "^[a-z]+$",
  "pass_rate": 1.0,
  "tests_passed": 2,
  "tests_total": 2,
  "test_results": [
    { "command": "grep -qE '^[a-z]+$' /tmp/jimmy_candidate_0", "expected_exit": 0, "exit_code": 0, "pass": true }
  ],
  "tokens": { "prompt": 45, "completion": 8, "total": 53 },
  "elapsed_ms": 812
}
```

`tokens` and `elapsed_ms` come from `parallel_output[I].results[0].tokens` and `parallel_output[I].results[0].elapsed_ms`.

**API error candidate shape (response was null — oracle skipped):**
```json
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
```

`error` = `"jimmy-skill error: "` + `parallel_output[I].results[0].error`
`error_type` = `parallel_output[I].results[0].error_type`

**Summary computation:**
```
total_candidates = N
api_errors = count of candidates where response == null
candidates_tested = N - api_errors
perfect_pass_rate = count of candidates where pass_rate == 1.0
any_passing = count of candidates where pass_rate > 0 (and pass_rate is not null)
```

## Step 5: Return

Output the JSON object only. No markdown fences, no commentary, no explanation — bare JSON starting with `{` and ending with `}`.

Example output for n=3, 2 test cases:
```
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
        { "command": "bash -c 'echo hello | grep -qE ^[a-z]+$'", "expected_exit": 0, "exit_code": 0, "pass": true }
      ],
      "tokens": { "prompt": 45, "completion": 8, "total": 53 },
      "elapsed_ms": 812
    },
    {
      "index": 1,
      "response": "[a-z]+",
      "pass_rate": 0.5,
      "tests_passed": 1,
      "tests_total": 2,
      "test_results": [
        { "command": "grep -qE '[a-z]+' /tmp/jimmy_candidate_1", "expected_exit": 0, "exit_code": 0, "pass": true },
        { "command": "bash -c 'echo hello | grep -qE ^[a-z]+$'", "expected_exit": 0, "exit_code": 1, "pass": false }
      ],
      "tokens": { "prompt": 45, "completion": 6, "total": 51 },
      "elapsed_ms": 755
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
    "total_candidates": 3,
    "candidates_tested": 2,
    "api_errors": 1,
    "perfect_pass_rate": 1,
    "any_passing": 2
  }
}
```

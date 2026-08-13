# Oracle protocol (jimmy-search)

## Type → system prompt

| type | System |
|------|--------|
| `regex` | Output only the regex pattern — no delimiters, no flags, no explanation. |
| `sql` | Output only the SQL query — no backticks, no explanation. |
| `shell` | Output only the shell command — no explanation. |
| (none/other) | Generate exactly one solution. Output only the solution — no explanation, preamble, fences, or commentary. |

Prefix each with `You are a solution generator. ` Append caller `system` with `\n` if set.

## Write-test-record (per candidate I)

1. `raw = results[0].response`. If null → API error item; skip oracle.
2. `printf '%s' "RAW" > /tmp/jimmy_candidate_I` — never `echo`.
3. For each test case: **text-replace** `$CANDIDATE_FILE` with literal `/tmp/jimmy_candidate_I` (you substitute; shell never sees the placeholder). Run `bash -c "SUBSTITUTED"`. `pass = (exit_code == expected_exit)`.
4. `pass_rate = tests_passed / len(test_cases)`.

**Security:** Candidate text is untrusted. Write only to the temp file. Never interpolate candidate content into command strings — only the safe path.

## Ranked output

Sort by `pass_rate` descending; null `pass_rate` last. Success item: `index`, `response`, `pass_rate`, `tests_passed`, `tests_total`, `test_results`, `tokens`, `elapsed_ms`. API error: `response/pass_rate` null, `error`, `error_type`.

Summary: `total_candidates`, `api_errors`, `candidates_tested`, `perfect_pass_rate`, `any_passing`.

# Shell oracle protocol

## Generator system instruction

| `type` | Instruction |
|---|---|
| `regex` | Output exactly the bare regex pattern. |
| `sql` | Output exactly the bare SQL query. |
| `shell` | Output exactly the shell command. |
| omitted | Generate exactly one solution and output only that solution. |

Prefix with `You are a solution generator.` Append caller `system` on a new line when supplied.

## Isolated write-test-clean cycle

1. Create one private, uniquely named temporary directory with owner-only access. Register cleanup for both normal completion and errors.
2. Write each caller-supplied test command verbatim to its own script in that directory using the host's file-write primitive. These scripts are the caller-authorized executable oracle.
3. For each successful candidate, write its response bytes to a separate file in that directory with the same file-write primitive. Candidate text is data: it never appears in a shell command, heredoc, path, environment value, or oracle script.
4. Execute each oracle script under the host's normal sandbox with `CANDIDATE_FILE` set to the candidate's absolute path. The script expands its required `$CANDIDATE_FILE`; the skill performs no text substitution. Apply `timeout_seconds` and record a timeout as a failed test.
5. Record `{ "command", "expected_exit", "actual_exit", "timed_out", "pass" }`, where `pass` means the command completed and `actual_exit === expected_exit`.
6. Remove the private directory after all candidates are recorded. Candidate testing is complete only after cleanup succeeds or a cleanup error is reported.

The oracle command may intentionally read or execute the candidate file. That behavior comes from the caller-supplied command, not from interpolation by the skill.

## Rank and summarize

For a generated candidate, `pass_rate = tests_passed / test_cases.length`. Include `index`, `response`, `pass_rate`, `tests_passed`, `tests_total`, `test_results`, `tokens`, and `elapsed_ms`. A generation failure has null `response` and `pass_rate` plus its error fields.

Sort by descending `pass_rate`, ascending original index; place generation failures last. Summary fields are `total_candidates`, `api_errors`, `candidates_tested`, `perfect_pass_rate`, and `any_passing`.

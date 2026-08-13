---
name: jimmy-search
description: Search a solution space by generating Jimmy candidates and ranking them with caller-supplied shell oracles when correctness is executable as exit-code tests.
---

# Jimmy search

Generate `n` candidate solutions, test each through the same oracle suite, and rank by pass rate.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Follow [the oracle protocol](references/oracle.md) for isolation, testing, cleanup, and ranking.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `problem` | yes | — | Non-empty solution prompt |
| `n` | yes | — | Positive integer |
| `test_cases` | yes | — | Non-empty array of `{ command, expected_exit }`; each command contains `$CANDIDATE_FILE` |
| `type` | no | — | `regex`, `sql`, `shell`, or omitted |
| `system` | no | — | Appended to the type-specific generator instruction |
| `max_concurrent` | no | 10 | Positive integer |
| `timeout_seconds` | no | 10 | Positive per-oracle-command timeout |

The caller-supplied commands are the executable correctness boundary. Run them under the host's normal permission and sandbox policy.

## Process

1. **Validate.** Enforce every input contract; require a non-empty command string and integer `expected_exit` for every case. Return a bare `usage` error object and stop on the first invalid input.
2. **Generate.** Build `n` identical problem items with the type-specific system instruction. JSON-serialize them and invoke parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
3. **Oracle.** Test every successful candidate with every test case using the oracle protocol. Preserve generation failures without running tests. Oracle work is complete when every candidate is either an API failure or has exactly `test_cases.length` recorded outcomes and all temporary files are removed.
4. **Return.** Emit only `{ "candidates", "summary" }`, sorted by descending `pass_rate`, then ascending original index; API failures sort last. Completion requires `candidates.length = n` and summary counts that reconcile to `n`.

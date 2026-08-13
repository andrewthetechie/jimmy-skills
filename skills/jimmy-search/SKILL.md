---
name: jimmy-search
description: Use when you need to generate candidate solutions and rank them with shell oracle tests, regex/SQL/shell search via pass_rate, or write-test-record selection against expected exit codes.
---

# jimmy-search

Generate N candidates, write each to a temp file, run oracle commands, rank by `pass_rate`.

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Oracle protocol: [oracle](references/oracle.md).

## Not for

Open-ended generation without a test oracle (use jimmy-candidates) or executing adversarial payloads (use jimmy-fuzz for inert data only).

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `problem` | yes | — | What to solve |
| `n` | yes | — | Candidates |
| `test_cases` | yes | — | `[{ command, expected_exit }]` — command must contain `$CANDIDATE_FILE` |
| `type` | no | — | `regex` / `sql` / `shell` shapes system prompt |
| `system` | no | — | Appended after type default |
| `max_concurrent` | no | 10 | |

## Steps

1. **Validate** — empty problem/n/test_cases → usage error. Each command must include `$CANDIDATE_FILE`. Stop.
2. **System by type** — see [oracle](references/oracle.md).
3. **Call once** — N identical `{prompt: problem, system}` items; `--max-iterations 1` explicitly.
4. **Oracle** — for each candidate follow [oracle](references/oracle.md) write-test-record (never interpolate candidate text into shell).
5. **Return** — bare JSON `{ candidates: [...sorted by pass_rate desc], summary: {...} }` only.

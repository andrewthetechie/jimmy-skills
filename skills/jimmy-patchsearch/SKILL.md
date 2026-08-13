---
name: jimmy-patchsearch
description: Generate unified-diff patch candidates with Jimmy and rank them by applying and testing each in an isolated Git worktree. Use for small bugs or refactors with executable tests and bounded file scope.
---

# Jimmy patch search

Generate patch candidates from a committed snapshot, test them away from the user's working tree, and return a ranked evidence bundle. The invoking agent reviews the winner; this skill leaves the current worktree unchanged.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Follow [the worktree protocol](references/worktrees.md) for context capture, patch validation, isolation, testing, cleanup, and ranking.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `task` | yes | — | Non-empty bug or refactor description |
| `context_files` | yes | — | Non-empty repo-relative files read from `base_ref` |
| `test_commands` | yes | — | Non-empty caller-authorized shell commands |
| `n` | no | 5 | Positive candidate count; practical maximum 10 |
| `base_ref` | no | `HEAD` | Commit or tree-ish used for prompt context and worktrees |
| `allowed_paths` | no | `context_files` | Repo-relative paths candidates may change |
| `system` | no | — | Appended to the patch-generator system instruction |
| `max_concurrent` | no | 10 | Positive integer |
| `timeout_seconds` | no | 120 | Positive timeout per test command |
| `max_context_chars` | no | 24000 | Positive cap for combined base-file context |

## Process

1. **Preflight.** Verify the Git repository, `base_ref`, context files, allowed paths, test commands, and numeric inputs. Read context from `base_ref`, so uncommitted working-tree changes never silently enter evaluation.
2. **Generate.** Build `n` identical patch prompts, JSON-serialize them, and invoke Jimmy parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
3. **Isolate.** Validate and evaluate every generated diff with the worktree protocol. Preserve generation/patch failures and run every test for every applied patch.
4. **Rank.** Sort applied candidates by descending pass rate, then ascending changed lines and original index; place invalid/unapplied candidates last.
5. **Return.** Emit only `{ "candidates", "winner", "summary" }`. Completion requires `candidates.length = n`, reconciled test counts, successful cleanup, and no mutation of the user's worktree.

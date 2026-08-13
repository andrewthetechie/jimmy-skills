# Isolated patch worktree protocol

## Capture a committed snapshot

Resolve `base_ref` to a commit. Validate every context and allowed path as a relative path inside the repository, free of `..`, and present in the base tree when used as context. An allowed path ending in `/` authorizes descendants; every other allowed path is an exact file match. Read prompt context with `git show BASE_REF:PATH`; the user's current worktree is read-only and excluded. Reject combined context above `max_context_chars` and ask the caller to narrow `context_files`.

Prompt Jimmy with the task, allowed paths, and complete context-file contents. Use this base system instruction and append caller `system`:

```text
You are a patch generator. Output exactly one unified Git diff that applies to the supplied base files and changes only allowed paths.
```

## Validate candidate text

Treat each response as untrusted data. Require a textual unified diff with at least one file change. Reject absolute paths, traversal, paths outside `allowed_paths`, binary patches, submodule changes, and malformed headers before invoking Git.

## Evaluate in isolation

1. Create one private temporary root and caller-command test scripts using the host file-write primitive.
2. For each candidate, add a detached worktree at `base_ref` under that root. Write candidate bytes to a patch file outside the worktree without shell interpolation.
3. Run `git apply --check`, then `git apply` inside the temporary worktree. Record failures and skip tests for unapplied patches.
4. Execute every caller-authorized test script from the worktree root with `timeout_seconds`. Record command, expected exit zero, actual exit, timeout, and pass. Candidate text never enters a command.
5. Count changed lines from the diff's added/deleted content, excluding file headers.
6. Remove every created worktree with `git worktree remove --force`, prune worktree metadata, and remove the private root. These operations target only paths created by this run.

## Rank and return

An applied candidate has `pass_rate = passed / test_commands.length`. Sort applied candidates by descending pass rate, ascending changed lines, then original index. Generation, validation, and apply failures follow in original order. `winner` is the first perfect candidate or null.

Each result retains index, patch text or null, stage (`generation`, `validation`, `apply`, `test`), applied flag, changed lines, test results, pass rate, tokens, elapsed time, and optional error fields. Summary fields are total, generated, valid, applied, perfect, failed, and cleanup status.

The skill returns evidence only. Applying a winning patch to the user's worktree requires a separate explicit request.

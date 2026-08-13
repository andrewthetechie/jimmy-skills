---
name: jimmy-fuzz
description: Use when you need inert adversarial payload variants for manual security testing across XSS, SQLi, path traversal, or other attack categories — never for automatic execution.
---

# jimmy-fuzz

Generate N payloads per attack category as **inert strings** for human/controlled testing.

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Assembly: [payloads](references/payloads.md).

## Safety

**Never execute, write to disk, or interpolate payload content into any shell command.** Return strings only. Inverse of jimmy-search (which sandboxes candidates in temp files for oracles).

## Not for

Automatic fuzzing, oracle selection (use jimmy-search), or exploiting live systems.

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `attack_surface` | yes | — | What is being tested |
| `attack_types` | yes | — | Non-empty category strings (custom OK) |
| `n` | yes | — | Payloads per category |
| `max_concurrent` | no | 10 | Per category call |

## Steps

1. **Validate** — empty surface/types/n → usage error. Unknown categories allowed (severity medium).
2. **Per category** — one parallel call with N identical items; `--max-iterations 1`. System/user templates in [payloads](references/payloads.md). Wait for each category before the next.
3. **Assemble** — inert payload objects only; severity map in reference. Global index = `(category_pos × n) + i`.
4. **Return** — bare JSON `{ warning, payloads, summary }` only. `warning` must state payloads are inert and must not be executed by the agent.

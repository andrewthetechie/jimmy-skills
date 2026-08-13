---
name: jimmy-transform
description: Use when you need parallel tone or format rewrites, one text with many instructions, one instruction across many texts, or batch style edits with optional structural validation retries.
---

# jimmy-transform

Rewrite text in parallel: **one-to-many** (`input` + `instructions[]`) or **many-to-one** (`inputs[]` + `instruction`).

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Validation retries: [validate-retry](references/validate-retry.md).

## Not for

Creative generation from scratch (use jimmy-candidates) or multi-step analysis.

## Inputs

| Param | Mode | Notes |
|-------|------|-------|
| `input` + `instructions` | one-to-many | One text, N instructions |
| `inputs` + `instruction` | many-to-one | N texts, one instruction |
| `system` | optional | Appended to writer role |
| `max_concurrent` | optional, default 10 | |
| `max_iterations` | optional, default 1 | Only `results[0]` used in output |
| `validate` | optional | `pattern` / `length` / `both` — see reference |
| `max_retries` | optional, default 2 | Only when `validate` set |

Mutually exclusive mode pairs — never mix both modes.

## Steps

1. **Validate mode** — exact error strings from prior skill behavior: wrong combo / empty fields → bare usage JSON. Stop.
2. **Build N user messages** — `"{instr}\n\nText to transform:\n{text}\n\nTransformed text:"`. System default: `You are a skilled writer. Follow the transformation instruction precisely.` Append caller `system` if set.
3. **Call once** with explicit `--max-iterations`:

```bash
jimmy-skill --parallel --max-concurrent MAX --max-iterations MAX_IT << 'JIMMY_INPUT'
[{"prompt":"...","system":"MERGED"}, ...]
JIMMY_INPUT
```

4. **Optional validate/retry** — [validate-retry](references/validate-retry.md).
5. **Return** — bare JSON array length N. Each item: `{ index, input, instruction, result, tokens, elapsed_ms }` or failure with `result: null`, `error`, `error_type`. No summary object. No fences.

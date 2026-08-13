---
name: jimmy-validate
description: Use when you need parallel yes/no checks, cheap boolean QA against code or copy, requirement checklists, or a pass/fail pre-screen before deeper analysis.
---

# jimmy-validate

Fan out N yes/no questions to Jimmy; parse booleans; return results + summary.

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Parsing: [parse](references/parse.md).

## Not for

Open-ended questions, multi-step reasoning, or comparisons between options.

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `questions` | yes | — | Strings or `{question, context?}` objects |
| `context` | no | — | Shared context; per-question `context` overrides |
| `system` | no | — | Appended to validator role |
| `max_concurrent` | no | 10 | |
| `max_iterations` | no | 1 | Leave at 1 (only `results[0]` is parsed) |

## Steps

1. **Validate** — empty questions → usage error. Stop.
2. **Build prompts** — system default: `You are a validator. Answer YES or NO, then give one sentence of reasoning.` Append caller `system` if set. User msg: `Context:\n...\n\nQuestion: ...` or `Question: ...`.
3. **Call once** — one JSON item per question; `--max-iterations 1` (or caller value) explicitly:

```bash
jimmy-skill --parallel --max-concurrent MAX --max-iterations MAX_IT << 'JIMMY_INPUT'
[{"prompt":"...","system":"MERGED"}, ...]
JIMMY_INPUT
```

4. **Parse** — apply [parse](references/parse.md) to each `results[0].response`.
5. **Return** — bare JSON `{ "results": [...], "summary": { "total", "passed", "failed", "errors" } }` only.

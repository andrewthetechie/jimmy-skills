---
name: jimmy-classify
description: Use when you need fast cheap single-label text classification, ensemble majority vote across categories, commit/content tagging, or sentiment routing with a confidence score.
---

# jimmy-classify

N ensemble votes with **prompt template variation**, then majority-vote winner + confidence.

**REQUIRED:** [jimmy-cli](references/jimmy-cli.md). Templates + voting: [vote](references/vote.md).

## Not for

Multi-label classification, categories with no semantic distinction, deep reasoning.

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `text` | yes | — | Classified text (user message) |
| `categories` | yes | — | Label strings (multi-word OK) |
| `n` | yes | — | Votes (min 3; ~7 recommended) |
| `system` | no | — | Role line appended to classifier system |
| `max_concurrent` | no | 100 | Run all votes in parallel |

## Steps

1. **Validate** — empty text/categories, `n < 3` → usage error. Stop.
2. **Build N items** — cycle 7 templates from [vote](references/vote.md); **do not** use `--max-iterations` for diversity (that repeats the identical prompt). Default system: `You are a text classifier. Be concise.` Append caller `system` if set.
3. **Call once** with `--max-iterations 1` explicitly:

```bash
jimmy-skill --parallel --max-concurrent MAX --max-iterations 1 << 'JIMMY_INPUT'
[{"prompt":"TPL","system":"MERGED"}, ...]
JIMMY_INPUT
```

4. **Extract + vote** — [vote](references/vote.md).
5. **Return** — bare JSON object only (`classification`, `confidence`, `votes`, `total_votes`, `raw_responses`, optional `errors`).

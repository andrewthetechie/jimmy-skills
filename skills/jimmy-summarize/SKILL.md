---
name: jimmy-summarize
description: Use when you need a fast cheap summary of text, multiple summary variants via max_iterations, or a pre-pass summary before deeper Claude reasoning.
---

# jimmy-summarize

Summarize text with `jimmy-skill` (one parallel item; optional multiple variants via `--max-iterations`).

**REQUIRED:** Read [jimmy-cli](references/jimmy-cli.md).

## Not for

Analytical synthesis that needs Claude-level judgment, or multi-document research reports.

## Inputs

| Param | Required | Default | Notes |
|-------|----------|---------|-------|
| `text` | yes | — | Text to summarize |
| `system` | no | — | Appended after skill default |
| `max_concurrent` | no | 10 | Passed through |
| `max_iterations` | no | 1 | >1 yields multiple summaries in `results` |

## Steps

1. **Validate** — empty `text` → `{"error":"text is required","error_type":"usage"}`. Bad concurrency/iterations → usage error. Stop.
2. **Merge system** — default: `You are a summarizer. Produce a concise summary of the following text in 2-3 sentences.` If caller `system` set, append `\n` + caller text.
3. **Call once** — one-item array; put merged system on the JSON item; pass `--max-iterations` explicitly:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations MAX_ITERATIONS << 'JIMMY_INPUT'
[{"prompt":"TEXT","system":"MERGED_SYSTEM"}]
JIMMY_INPUT
```

4. **Return** — bare JSON array from the binary as-is (no reshape).

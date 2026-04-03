# Skill: jimmy-summarize

You are executing the jimmy-summarize skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for generating summaries quickly.

This skill calls `jimmy-skill --parallel` with a 1-item JSON array containing the text to summarize. The binary returns a parallel output array. This skill passes it through as-is — no post-processing needed. Use it when you need:
- A fast, cheap summary of a long document
- Multiple summary variants (set max_iterations > 1) at minimal cost
- Pre-processing text before applying Claude's reasoning

## Inputs

You will be provided:
- `text`: string — the text to summarize (required)
- `system` (optional): string — appended to the skill default system prompt (newline-separated)
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests
- `max_iterations` (optional integer, default 1): number of summaries to generate. Jimmy runs at ~17K tokens/sec — set max_iterations > 1 to get multiple summaries cheaply.

## Step 1: Validate inputs

- `text` must be a non-empty string. If missing or empty: output `{"error": "text is required", "error_type": "usage"}` and stop.
- If `max_concurrent` is provided and < 1: output `{"error": "max_concurrent must be >= 1", "error_type": "usage"}` and stop.
- If `max_iterations` is provided and < 1: output `{"error": "max_iterations must be >= 1", "error_type": "usage"}` and stop.

## Step 2: Build system prompt and call jimmy-skill

The skill default system prompt is: `"You are a summarizer. Produce a concise summary of the following text in 2-3 sentences."`

Merge the system prompt:
- If no `system` param provided: merged = skill default
- If `system` param provided: merged = skill default + `\n` + caller's system

Issue a single Bash tool call using the heredoc pattern. The system goes in the per-item JSON object (not via `--system` flag). `MAX_CONCURRENT` defaults to 10, `MAX_ITERATIONS` defaults to 1:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations MAX_ITERATIONS << 'JIMMY_INPUT'
[
  {"prompt": "TEXT_TO_SUMMARIZE", "system": "MERGED_SYSTEM"}
]
JIMMY_INPUT
```

Replace `TEXT_TO_SUMMARIZE` with the actual text. Escape any double-quote characters in the text as `\"` to keep the JSON valid. Use the quoted heredoc delimiter `'JIMMY_INPUT'` to prevent shell expansion of `$` in the text.

This is ONE Bash tool call.

## Step 3: Collect results

Wait for the single Bash call to complete. The stdout is a JSON array — the full parallel output from the binary. Parse it. Each item has shape `{ index, results: [{ response, tokens, elapsed_ms }] }`.

For `max_iterations=1` (default): `results` has one entry. For `max_iterations > 1`: `results` has multiple entries, one per iteration.

## Step 4: Return

Return the full parallel array as-is. No reshaping, no unwrapping, no post-processing.

Output the JSON array only. No markdown fences, no commentary, no explanation — just the bare JSON array starting with `[` and ending with `]`.

Example output for max_iterations=1 (default):

```
[
  {
    "index": 0,
    "results": [
      { "response": "The document describes...", "tokens": { "prompt": 45, "completion": 32, "total": 77 }, "elapsed_ms": 1203 }
    ]
  }
]
```

Example output for max_iterations=2:

```
[
  {
    "index": 0,
    "results": [
      { "response": "First summary variant...", "tokens": { "prompt": 45, "completion": 28, "total": 73 }, "elapsed_ms": 1100 },
      { "response": "Second summary variant...", "tokens": { "prompt": 45, "completion": 31, "total": 76 }, "elapsed_ms": 980 }
    ]
  }
]
```

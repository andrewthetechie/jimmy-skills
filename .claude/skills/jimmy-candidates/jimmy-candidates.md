# Skill: jimmy-candidates

You are executing the jimmy-candidates skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for generating multiple variations of something so you can pick the best one.

This skill calls `jimmy-skill --parallel` with a JSON array of N identical prompts and returns all results as a JSON array. Use it when you need:
- Multiple phrasing or wording candidates to compare and pick the best
- Cheap first-pass generation before applying your own reasoning
- Parallel exploration of a prompt from different angles
- Draft content that you will then refine or evaluate

## Inputs

You will be provided:
- `prompt`: string — the prompt to send to Jimmy (same prompt is sent for every candidate)
- `n`: integer — number of candidates to generate (must be >= 1; practical max ~20)
- `system` (optional): string — a system prompt to pass via --system to every call
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests passed to `jimmy-skill --max-concurrent`
- `max_iterations` (optional integer, default 1): how many times each prompt is sent to Jimmy per array item. Jimmy runs at ~17K tokens/sec — use `max_iterations > 1` to get multiple results per item at minimal cost. For jimmy-candidates, each item is the same prompt repeated, and `max_iterations` adds more repetitions within each item. In most cases, set `n` to the number of candidates you want and leave `max_iterations=1`.

## Step 1: Validate inputs

- `prompt` must be a non-empty string. If missing or empty, output `{"error": "prompt is required", "error_type": "usage"}` and stop.
- `n` must be a positive integer. If missing, zero, or negative, output `{"error": "n must be a positive integer", "error_type": "usage"}` and stop.
- If `max_concurrent` is provided and is less than 1, output `{"error": "max_concurrent must be >= 1", "error_type": "usage"}` and stop.
- If `max_iterations` is provided and is less than 1, output `{"error": "max_iterations must be >= 1", "error_type": "usage"}` and stop.

## Step 2: Construct the JSON array and call jimmy-skill --parallel

jimmy-candidates sends the SAME prompt N times. Construct a JSON array with N identical items (each item has the same `prompt` text; no per-item `system` is needed since all items are identical — use the shared `--system` flag instead).

In your next response, issue a single Bash tool call:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations MAX_ITERATIONS << 'JIMMY_INPUT'
[
  {"prompt": "PROMPT_TEXT"},
  {"prompt": "PROMPT_TEXT"},
  ...  (N items total, all identical)
]
JIMMY_INPUT
```

Replace `PROMPT_TEXT` with the actual prompt. Replace `MAX_CONCURRENT` and `MAX_ITERATIONS` with the provided values (or their defaults: 10 and 1). `N` is the number of candidates requested. Include `--system "SYSTEM_TEXT"` before `<<` if system is provided.

When constructing the JSON array, escape any double-quote characters in the prompt text as `\"` to keep the JSON valid.

Use the heredoc form `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` in the prompt text.

The binary handles all concurrency internally. This is ONE Bash tool call.

## Step 3: Collect results

Wait for the single Bash call to complete. The stdout is a JSON array. Parse it. Each item has shape `{ index, results: [{ response, tokens, elapsed_ms }] }`.

For candidates with `max_iterations=1` (the default), read `item.results[0].response` for the candidate text.

If `max_iterations > 1`, there are multiple results per item: `item.results[0].response`, `item.results[1].response`, etc. Flatten all responses across all items and results into the candidates array if needed, or keep the nested structure — caller decides.

## Step 4: Assemble output array

For each item at index I (0-based) in the parallel output:
- Extract `item.results[0]` (or all results if `max_iterations > 1`)
- If `item.results[0].response` is not null: it is a successful candidate
- If `item.results[0].response` is null: construct an error candidate item using `item.results[0].error` and `item.results[0].error_type`

Output array item shape (for `max_iterations=1`):
```json
{ "index": I, "response": "...", "tokens": { ... }, "elapsed_ms": N }
```

Error item shape:
```json
{ "index": I, "response": null, "tokens": { "prompt": 0, "completion": 0, "total": 0 }, "elapsed_ms": N, "error": "...", "error_type": "..." }
```

The array must contain exactly N items.

## Step 5: Return

Output the JSON array only. No markdown fences, no commentary, no explanation — just the bare JSON array starting with `[` and ending with `]`.

Example output for n=2:
```
[
  { "index": 0, "response": "Blazing fast AI at your fingertips.", "tokens": { "prompt": 12, "completion": 9, "total": 21 }, "elapsed_ms": 843 },
  { "index": 1, "response": "Think faster. Jimmy delivers.", "tokens": { "prompt": 12, "completion": 5, "total": 17 }, "elapsed_ms": 792 }
]
```

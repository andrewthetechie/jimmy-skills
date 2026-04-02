# Skill: jimmy-candidates

You are executing the jimmy-candidates skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends a single prompt to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns a JSON object. It is fast and cheap — ideal for generating multiple variations of something so you can pick the best one.

This skill calls `jimmy-skill` N times (once per candidate) and returns all results as a JSON array. Use it when you need:
- Multiple phrasing or wording candidates to compare and pick the best
- Cheap first-pass generation before applying your own reasoning
- Parallel exploration of a prompt from different angles
- Draft content that you will then refine or evaluate

## CRITICAL: How parallelism works here

`jimmy-skill` accepts ONE prompt and returns ONE result. It has NO `--parallel` flag. Do not attempt to use one.

Parallelism in this skill means: **use N separate Bash tool calls in a single response message**. Claude Code executes tool calls that appear in the same response at the same time. This is what makes the calls parallel — not anything in the shell command itself.

**DO NOT** write a shell loop (`for i in ...`), pipe multiple calls together, or use `&` background processes. Each candidate must be a separate Bash tool call.

Correct approach for n=3:
```
Response message containing 3 Bash tool calls at once:
  Bash call 1: jimmy-skill "your prompt"
  Bash call 2: jimmy-skill "your prompt"
  Bash call 3: jimmy-skill "your prompt"
```

These 3 calls run simultaneously. Wait for all 3 to complete, then assemble results.

## Inputs

You will be provided:
- `prompt`: string — the prompt to send to Jimmy (same prompt is sent to every call)
- `n`: integer — number of candidates to generate (must be >= 1; practical max ~10)
- `system` (optional): string — a system prompt to pass via --system to every call

## Step 1: Validate inputs

- `prompt` must be a non-empty string. If missing or empty, output `{"error": "prompt is required", "error_type": "usage"}` and stop.
- `n` must be a positive integer. If missing, zero, or negative, output `{"error": "n must be a positive integer", "error_type": "usage"}` and stop.

## Step 2: Issue N Bash tool calls in one response

In your next response, include exactly N Bash tool calls and nothing else. Do not write any text before or between them — just the tool calls.

Command when no system prompt:
```
jimmy-skill "PROMPT"
```

Command when system prompt is provided:
```
jimmy-skill "PROMPT" --system "SYSTEM"
```

Replace PROMPT with the actual prompt text. Replace SYSTEM with the system text if provided. The prompt is identical across all N calls — Jimmy's temperature gives natural variation.

**Reminder:** N calls in one response = parallel. One call with a loop = wrong.

## Step 3: Collect results

Wait for all N Bash calls to complete. Each stdout is always valid JSON (Jimmy outputs JSON even on errors — never empty, never plain text). Collect the stdout from each call in order (0 to N-1).

## Step 4: Assemble output array

For each result at position I (0-based):

1. Parse the stdout JSON
2. Add `"index": I` to the object
3. Append to the array

If a Bash call failed entirely or produced non-JSON stdout (shouldn't happen, but be safe), construct an error item:
```json
{ "index": I, "response": null, "tokens": { "prompt": 0, "completion": 0, "total": 0 }, "elapsed_ms": 0, "error": "bash call failed: STDERR", "error_type": "parse" }
```

The array must contain exactly N items regardless of failures.

## Step 5: Return

Output the JSON array only. No markdown fences, no commentary, no explanation — just the bare JSON array starting with `[` and ending with `]`.

Example output for n=2:
```
[
  { "index": 0, "response": "Blazing fast AI at your fingertips.", "tokens": { "prompt": 12, "completion": 9, "total": 21 }, "elapsed_ms": 843 },
  { "index": 1, "response": "Think faster. Jimmy delivers.", "tokens": { "prompt": 12, "completion": 5, "total": 17 }, "elapsed_ms": 792 }
]
```

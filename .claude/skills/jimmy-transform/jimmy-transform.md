# Skill: jimmy-transform

You are executing the jimmy-transform skill. Follow these instructions exactly.

## What this skill does

`jimmy-skill` is a CLI that sends prompts to ChatJimmy (a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec) and returns structured JSON. It is fast and cheap — ideal for rewriting or varying text across multiple instructions or multiple inputs in parallel.

This skill calls `jimmy-skill --parallel` with a JSON array of N transform items, assembles the results into a bare JSON array, and returns it. Use it for:
- Tone rewriting — make text formal, casual, friendly, or technical
- Format conversion — bullet points, single sentence, expanded paragraph
- Audience adaptation — executives, developers, beginners
- Batch style edits — apply one instruction across N pieces of content

Two modes are supported: one input with N instructions (one-to-many), or N inputs with one instruction (many-to-one). Mode is auto-detected from the parameter names.

This is NOT for creative generation from scratch (use jimmy-candidates) or anything requiring multi-step reasoning.

## Inputs

You will be provided exactly one of two valid parameter combinations:

- `input` (string): Single text to transform. Use with `instructions`.
- `instructions` (array of strings): Transformation instructions to apply to `input`. Use with `input`.
- `inputs` (array of strings): Texts to transform. Use with `instruction`.
- `instruction` (string): Single instruction applied to every text in `inputs`. Use with `inputs`.
- `system` (optional string): Caller's style/context instructions. Merged with skill default — skill default is prepended, caller's text is appended (newline-separated).
- `max_concurrent` (optional integer, default 10): maximum simultaneous HTTP requests
- `max_iterations` (optional integer, default 1): how many times each transform is sent to Jimmy. For jimmy-transform, only `results[0].response` is used as the transform result — `max_iterations > 1` is useful if you want to compare multiple transform attempts for the same item. Default 1 is recommended.

Exactly one of the two valid combos must be present: (`input` + `instructions`) or (`inputs` + `instruction`).

## Step 1: Validate inputs

Check in this order:

1. If `input` AND `inputs` are both present: output `{"error": "provide either input (one-to-many) or inputs (many-to-one), not both", "error_type": "usage"}` and stop.
2. If `instruction` AND `instructions` are both present: output `{"error": "provide either instruction (many-to-one) or instructions (one-to-many), not both", "error_type": "usage"}` and stop.
3. If `input` is present AND `instruction` is present AND neither `inputs` nor `instructions` is present: output `{"error": "one-to-many mode requires instructions (array); many-to-one mode requires inputs (array)", "error_type": "usage"}` and stop.
4. If `input` (string) is present AND `instructions` (array) is present — **one-to-many mode**:
   - If `input` is an empty string: output `{"error": "input is required and cannot be empty", "error_type": "usage"}` and stop.
   - If `instructions` is an empty array: output `{"error": "instructions must be a non-empty array", "error_type": "usage"}` and stop.
   - Set N = len(instructions). Mode confirmed: one-to-many.
5. If `inputs` (array) is present AND `instruction` (string) is present — **many-to-one mode**:
   - If `inputs` is an empty array: output `{"error": "inputs must be a non-empty array", "error_type": "usage"}` and stop.
   - If any item in `inputs` is an empty string: output `{"error": "input at index N is empty", "error_type": "usage"}` (use the actual index N) and stop.
   - If `instruction` is an empty string: output `{"error": "instruction is required and cannot be empty", "error_type": "usage"}` and stop.
   - Set N = len(inputs). Mode confirmed: many-to-one.
6. Neither valid combo present: output `{"error": "provide input+instructions (one-to-many) or inputs+instruction (many-to-one)", "error_type": "usage"}` and stop.

Output these error responses as bare JSON with no markdown fences and stop immediately — no Bash calls.

After validation: state clearly whether mode is one-to-many or many-to-one and what N is.

## Step 2: Construct the JSON array and call jimmy-skill --parallel

Assign per-call variables using mode detection from Step 1:

One-to-many (for call at index I, I = 0 to N-1):
```
input_for_call = input              (same string for all N calls)
instr_for_call = instructions[I]    (different instruction per call)
```

Many-to-one (for call at index I, I = 0 to N-1):
```
input_for_call = inputs[I]          (different input per call)
instr_for_call = instruction        (same string for all N calls)
```

**User message construction** (instruction first, then input, then priming suffix — UNCHANGED):

```
user_message = "{instr_for_call}\n\nText to transform:\n{input_for_call}\n\nTransformed text:"
```

**System prompt construction (`MERGED_SYSTEM`):**

No caller `system` param:
```
system_prompt = "You are a skilled writer. Follow the transformation instruction precisely."
```

With caller `system` param:
```
system_prompt = "You are a skilled writer. Follow the transformation instruction precisely.\n{caller_system}"
```

Place each user message as the `prompt` field and the merged system prompt as the `system` field:

```bash
jimmy-skill --parallel --max-concurrent MAX_CONCURRENT --max-iterations MAX_ITERATIONS << 'JIMMY_INPUT'
[
  {"prompt": "ITEM_0_USER_MSG", "system": "MERGED_SYSTEM"},
  {"prompt": "ITEM_1_USER_MSG", "system": "MERGED_SYSTEM"},
  ...
]
JIMMY_INPUT
```

Escape any `"` characters in prompt or system text as `\"` in the JSON. Use `<< 'JIMMY_INPUT'` (quoted delimiter) to prevent shell expansion of `$` characters.

Issue exactly ONE Bash tool call. The binary handles concurrency internally.

## Step 3: Collect results

Wait for the single Bash call to complete. The stdout is a JSON array. Parse it. Each item has shape `{ index, results: [{ response, tokens, elapsed_ms }] }`.

Extract the transform result for item at index I:
- `parallel_output[I].results[0].response` — this is the raw transform text (do NOT parse it, do NOT strip prefixes — accept as-is)
- If `parallel_output[I].results[0].response` is null: treat it as a failed transform using `parallel_output[I].results[0].error` and `parallel_output[I].results[0].error_type`

## Step 4: Assemble output array

For each index I (0-based):

**If the transform succeeded (`results[0].response` is not null):**
```json
{
  "index": I,
  "input": "value of input_for_call at index I",
  "instruction": "value of instr_for_call at index I",
  "result": "parallel_output[I].results[0].response",
  "tokens": "parallel_output[I].results[0].tokens",
  "elapsed_ms": "parallel_output[I].results[0].elapsed_ms"
}
```

**If the transform failed (`results[0].response` is null):**
```json
{
  "index": I,
  "input": "value of input_for_call at index I",
  "instruction": "value of instr_for_call at index I",
  "result": null,
  "error": "jimmy-skill error: {results[0].error}",
  "error_type": "{results[0].error_type}"
}
```

Important notes:
- `result` is the raw `response` string from `parallel_output[I].results[0].response` — do NOT parse it, do NOT strip prefixes. The priming suffix `\n\nTransformed text:` causes Jimmy to continue directly; accept the full response as-is.
- Output array must contain exactly N items regardless of failures.
- There is NO summary aggregation object. Do not add one.

## Step 5: Return

Output the JSON array only. No markdown fences, no commentary, no explanation — just the bare JSON array starting with `[` and ending with `]`.

Example output for N=2:
```json
[
  {
    "index": 0,
    "input": "The cat sat on the mat.",
    "instruction": "make it formal",
    "result": "The feline was positioned upon the mat.",
    "tokens": { "prompt": 32, "completion": 18, "total": 50 },
    "elapsed_ms": 743
  },
  {
    "index": 1,
    "input": "The cat sat on the mat.",
    "instruction": "translate to French",
    "result": null,
    "error": "jimmy-skill error: connection refused",
    "error_type": "network"
  }
]
```

# jimmy-transform

Rewrite or vary text using Jimmy in parallel: apply N transformation instructions to one input (one-to-many), or apply one instruction across N inputs (many-to-one).

## When to use this skill

Use jimmy-transform when you need to rewrite, reformat, or adapt text using multiple variations or across multiple pieces of content. Good for:

- **Tone rewriting** — "make this formal", "make this casual", "make this friendly for a non-technical audience"
- **Format conversion** — "convert to bullet points", "condense into a single sentence", "expand into a full paragraph"
- **Audience adaptation** — "adapt for executives", "adapt for a developer audience", "simplify for beginners"
- **Batch style edits** — apply the same house style instruction across N pieces of content simultaneously
- **Style variants** — generate 3 different tone variations of one text to pick from

Not suited for: creative generation from scratch (use jimmy-candidates), multi-step reasoning or analysis, anything requiring Claude-level intelligence.

## How parallelism works here

`jimmy-skill --parallel` handles all concurrency internally. The skill constructs one JSON item per transform call and pipes the array in a single Bash call. Mode detection (one-to-many vs many-to-one) is unchanged. See jimmy-transform.md for full details.

## Parameters

| Parameter      | Required     | Description |
|----------------|--------------|-------------|
| `input`        | one-to-many  | Single text to transform (provide with `instructions`) |
| `instructions` | one-to-many  | Array of transformation instructions to apply to `input` |
| `inputs`       | many-to-one  | Array of texts to transform (provide with `instruction`) |
| `instruction`  | many-to-one  | Single instruction applied to every text in `inputs` |
| `system`       | no           | Optional system prompt for shared context (style guide, constraints). Merged with skill default. Keep short — put reference material in the text content, not the system prompt. |
| `max_concurrent` | no         | Maximum simultaneous HTTP requests. Default: 10. |
| `max_iterations` | no         | How many times each transform is sent to Jimmy. Default: 1. Useful if you want multiple transform variants per item — only results[0] is used in the default output shape. |
| `validate`       | no           | Optional structural validation applied to each transform result before accepting it. Object with `type` field: `"pattern"` (regex match against full result text), `"length"` (character count bounds), or `"both"` (pattern AND length, both must pass). See validation object shapes below. |
| `max_retries`    | no           | Maximum retry attempts per item when `validate` is provided and the result fails validation. Default: 2. Each retry is a full new Jimmy request for only the failing item. Ignored if `validate` is absent. |

### Validation object shapes

```json
{ "type": "pattern", "pattern": "^[A-Z]" }
```
Checks that the result matches the regex pattern. Pattern is tested against the full result string.

```json
{ "type": "length", "min_length": 10, "max_length": 200 }
```
Checks that `len(result) >= min_length` and `len(result) <= max_length`. Omit `min_length` or `max_length` to leave that bound unchecked.

```json
{ "type": "both", "pattern": "^[A-Z]", "min_length": 10 }
```
Both pattern match AND length bound must pass. Either failure triggers a retry.

## Output

JSON array of N items. Each item echoes back the input, instruction, transformed result, token counts, and elapsed time:

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
    "error": "API timeout after 120s",
    "error_type": "timeout"
  }
]
```

Output is a bare JSON array — no wrapper object. Array always has exactly N items.

Per-item failures set `result: null`, `error`, and `error_type` without affecting other items.

`error_type` values: `"timeout"`, `"network"`, `"api"`, `"parse"`, `"usage"`, `"validation"`

When `validate` is provided: a result that fails validation after all retries produces `result: null`, `error_type: "validation"`, `error: "validation failed: ..."`. Shape is identical to other per-item failures.

**Note on the `system` param:** The `system` param is for style instructions only — keep it short. Put reference material (e.g., glossaries, brand voice guides) in the input text, not the system prompt, to avoid hitting Jimmy's 28K system prompt cap.

## Bulk-transform examples

Apply one instruction to many inputs (many-to-one mode):

```json
{
  "inputs": [
    "Our system encountered an unexpected condition.",
    "The operation could not be completed.",
    "Input validation did not succeed."
  ],
  "instruction": "Rewrite as a short, user-friendly error message in plain English. One sentence. No technical jargon.",
  "system": "You write error messages for a consumer mobile app."
}
```

Apply many instructions to one input (one-to-many mode):

```json
{
  "input": "The transformer model uses self-attention mechanisms to compute contextual embeddings across the input sequence.",
  "instructions": [
    "Rewrite for a non-technical executive audience in one sentence.",
    "Rewrite as a tweet (max 280 characters, casual tone).",
    "Rewrite as a definition for a technical glossary."
  ]
}
```

Apply one instruction to many inputs with output validation (many-to-one + validate):

```json
{
  "inputs": [
    "def add(a, b): return a + b",
    "fn multiply(x: i32, y: i32) -> i32 { x * y }",
    "const divide = (a, b) => a / b;"
  ],
  "instruction": "Write a one-sentence docstring for this function. Start with a capital letter. End with a period.",
  "validate": { "type": "both", "pattern": "^[A-Z]", "min_length": 10, "max_length": 120 },
  "max_retries": 2
}
```

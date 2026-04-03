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

`error_type` values: `"timeout"`, `"network"`, `"api"`, `"parse"`, `"usage"`

**Note on the `system` param:** The `system` param is for style instructions only — keep it short. Put reference material (e.g., glossaries, brand voice guides) in the input text, not the system prompt, to avoid hitting Jimmy's 28K system prompt cap.

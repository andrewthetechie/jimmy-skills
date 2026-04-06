# jimmy-candidates

Generate N candidate responses from a prompt using the jimmy-skill binary (ChatJimmy / Llama 3.1 8B at ~17K tokens/sec).

## When to use this skill

Use jimmy-candidates when you need fast, cheap generation of multiple variations — then apply your own reasoning to pick or refine the best one. Good for:

- **Wording candidates** — "Give me 5 ways to phrase this error message"
- **Draft content** — "Generate 3 README intros, I'll pick the best"
- **Parallel exploration** — "Try 4 different approaches to this function signature"
- **Cheap validation** — "Generate 10 yes/no answers to check consistency"
- **Creative variation** — "Write 5 taglines for this product"

Not suited for: long reasoning chains, tool use, precise instruction-following, or anything requiring Claude-level intelligence. Jimmy is fast and cheap — use it for volume.

## How parallelism works here

`jimmy-skill --parallel` handles all concurrency internally. The skill constructs a JSON array of N prompt objects (all identical — same prompt repeated N times) and pipes it to the binary in a single Bash call. The binary fans out HTTP requests up to `max_concurrent` at a time and returns an ordered JSON array.

See jimmy-candidates.md for full details.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `prompt`  | yes      | The prompt to send to Jimmy (same prompt sent to all N items) |
| `n`       | yes      | Number of candidates to generate (integer >= 1; practical max ~20) |
| `system`  | no       | Optional system prompt shared across all candidates |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 10. |
| `max_iterations` | no | How many times each prompt is sent to Jimmy per item. Default: 1. Jimmy runs at ~17K tokens/sec — increase this to get more candidates at minimal cost. |

## Output

JSON array of N items. Each item is assembled from the binary's parallel output (`results[0]` per item for the default `max_iterations=1`):

```json
[
  { "index": 0, "response": "Blazing fast AI.", "tokens": { "prompt": 12, "completion": 9, "total": 21 }, "elapsed_ms": 843 },
  { "index": 1, "response": null, "tokens": { "prompt": 0, "completion": 0, "total": 0 }, "elapsed_ms": 45, "error": "Request timed out", "error_type": "timeout" }
]
```

Per-item failures set `response: null`, `error`, and `error_type` without affecting other items. Array always has exactly N items.

`error_type` values: `"timeout"`, `"network"`, `"api"`, `"parse"`, `"usage"`.

## Scaffolding use cases

Use jimmy-candidates to generate code scaffolding in bulk. Pass a concrete signature, name, or spec as the prompt; use `system` for language and framework context.

**Struct / type definitions** (Rust structs, TypeScript interfaces, Python dataclasses):

```json
{
  "prompt": "Generate a Rust struct for a user record with fields: id (u64), email (String), display_name (String), created_at (i64 Unix timestamp), is_active (bool). Include serde derive macros for Serialize and Deserialize.",
  "n": 5,
  "system": "Generate Rust code using serde and derive macros. Output only the struct definition — no impl blocks, no use statements, no explanatory text."
}
```

**Test stubs** (unit test boilerplate, describe blocks, fixture setups):

```json
{
  "prompt": "Write a Jest describe block for a function called `validateEmail` that accepts a string and returns a boolean. Include test cases for: valid email, missing @ symbol, empty string, whitespace-only string, email with no domain.",
  "n": 4,
  "system": "Generate TypeScript Jest tests using describe/it/expect. Output only the describe block — no imports, no module.exports."
}
```

**Docstrings / JSDoc** (function-level documentation from a signature):

```json
{
  "prompt": "Write a JSDoc comment for this function signature: `async function fetchUserById(id: string, options?: { includeDeleted?: boolean }): Promise<User | null>`",
  "n": 6,
  "system": "Generate JSDoc comments only. Include @param tags for all parameters, @returns tag, and a one-sentence description. No implementation code."
}
```

**Impl / method stubs** (method body scaffolding, placeholder returns, TODO comments):

```json
{
  "prompt": "Generate a stub implementation for this Rust trait method: `fn validate(&self, input: &str) -> Result<(), ValidationError>`. Include a TODO comment and return a placeholder Err.",
  "n": 5,
  "system": "Generate Rust code only. Output only the method body inside curly braces — assume the trait and ValidationError type are already defined. Use `todo!()` macro or an explicit Err return."
}
```

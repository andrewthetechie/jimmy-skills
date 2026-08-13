# jimmy-skill CLI

Prerequisite: `jimmy-skill` on `$PATH` (build with `cargo build --release`, then install the binary).

ChatJimmy is a fast, cheap Llama 3.1 8B (~17K tok/s). You reason and judge; Jimmy generates volume. No auth in open beta.

## Parallel mode

**Always pass flags explicitly.** Binary defaults are `--max-concurrent 100` and `--max-iterations 25` — most skills override these.

```bash
jimmy-skill --parallel --max-concurrent N --max-iterations M << 'JIMMY_INPUT'
[
  {"prompt": "TEXT", "system": "OPTIONAL"}
]
JIMMY_INPUT
```

| Rule | Why |
|------|-----|
| Quoted heredoc `'JIMMY_INPUT'` | Prevents `$` shell expansion in prompts |
| Escape `"` as `\"` in JSON strings | Keep stdin JSON valid |
| One parallel invocation per batch | Binary fans out HTTP; do not open N serial shells |
| Prefer `--system` flag when all items share one system | Or put `"system"` per JSON item when they differ |

**Stdout shape** (ordered array, one entry per input item):

```json
[
  {
    "index": 0,
    "results": [
      { "response": "...", "tokens": { "prompt": 1, "completion": 1, "total": 2 }, "elapsed_ms": 400 }
    ]
  }
]
```

Failed items keep their index; `response` is `null` with `error` / `error_type` (`timeout` | `network` | `api` | `parse` | `usage`). Other items still succeed.

## Single-prompt mode

```bash
jimmy-skill "prompt text" --system "optional"
# or: echo "prompt" | jimmy-skill
```

Use parallel mode for multi-call skills.

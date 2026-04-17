# jimmy-router

An OpenAI-compatible HTTP proxy that fans a prompt out to [ChatJimmy](https://chatjimmy.ai) multiple times in parallel, then synthesizes the results into a single response using a configurable LLM backend.

## How it works

1. Receives a `POST /v1/chat/completions` request (OpenAI API format)
2. Sends the prompt to ChatJimmy `N` times concurrently (`iterations` in config)
3. Passes all N draft responses to your synthesizer backend with a merge prompt
4. Returns the synthesized result as a standard OpenAI chat completion response

## Requirements

- Rust toolchain (stable, via [rustup](https://rustup.rs))
- A running synthesizer backend (Ollama, OpenAI, OpenRouter, or any OpenAI-compatible API)

## Build

```bash
cargo build --bin jimmy-router
```

For a release build:

```bash
cargo build --release --bin jimmy-router
```

## Configure

Copy `router.toml` and edit it to point at your synthesizer:

```toml
port = 3000
iterations = 5   # number of ChatJimmy calls to fan out per request

[synthesizer]
base_url = "http://localhost:11434/v1"   # OpenAI-compatible endpoint
api_key  = "ollama"                      # use "ollama" for local Ollama
model    = "llama3.1"
```

The `[synthesizer]` section is **required**. The router will probe the endpoint on startup and refuse to start if it cannot reach it.

### Synthesizer backend options

**Ollama (local, no API key needed)**
```toml
[synthesizer]
base_url = "http://localhost:11434/v1"
api_key  = "ollama"
model    = "llama3.1"
```

**OpenAI**
```toml
[synthesizer]
base_url = "https://api.openai.com/v1"
api_key  = "sk-..."
model    = "gpt-4o-mini"
```

**OpenRouter**
```toml
[synthesizer]
base_url = "https://openrouter.ai/api/v1"
api_key  = "sk-or-..."
model    = "anthropic/claude-3.5-haiku"
```

**z.ai GLM**
```toml
[synthesizer]
base_url = "https://api.z.ai/api/paas/v4"
api_key  = "..."
model    = "glm-4.7"
```

### Optional synthesizer settings

```toml
[synthesizer]
# ... required fields above ...

max_tokens    = 2048
temperature   = 0.7
system_prompt = "Custom synthesis instructions here..."
```

### Optional router settings

```toml
max_concurrent = 10   # max parallel ChatJimmy calls (default: 10)
max_retries    = 2    # retries per ChatJimmy call on failure (default: 2)
```

## Run

```bash
./target/debug/jimmy-router --config router.toml
```

Or with a release build:

```bash
./target/release/jimmy-router --config router.toml
```

The server starts on the configured port and logs the address to stderr:

```
jimmy-router listening on 0.0.0.0:3000
```

## Usage

Send requests using the OpenAI chat completions format:

```bash
curl http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "jimmy-router",
    "messages": [{"role": "user", "content": "Explain backpressure in one paragraph."}]
  }'
```

Or point any OpenAI-compatible client at `http://localhost:3000` with any API key (authentication is not enforced).

## Limitations

- **No streaming** — `stream: true` returns `501 Not Implemented`
- **No tool calling** — `tools` field returns `501 Not Implemented`
- **Token counts** — `usage` fields in the response are always zero (stub)
- ChatJimmy is the fan-out worker; the synthesizer is the merge step — both must be reachable

<!-- GSD:project-start source:PROJECT.md -->
## Project

**jimmy-skill**

A Rust CLI (`jimmy-skill`) that talks directly to the ChatJimmy API — a hardware-accelerated Llama 3.1 8B running at ~17K tokens/sec. It is designed to be invoked by Claude Code skills, serving as the fast, cheap worker in a Claude + Jimmy split-brain pattern: Claude reasons and plans, Jimmy generates at scale.

**Core Value:** Claude skills can fan out work to Jimmy for fast parallel token generation and get back structured JSON results — enabling candidates, transforms, and cheap validation at a fraction of the cost of a Claude call.

### Constraints

- **Language**: Rust — binary distribution, speed, async concurrency with tokio
- **Output**: JSON only — skills need reliable parsing, not human-readable text
- **No auth**: ChatJimmy open beta has no API key requirement; don't add premature abstraction
- **Simplicity**: Text-in, text-out. No tool calling loop. Skills handle orchestration.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Runtime & Async
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **tokio** | `1` (latest: 1.50.0) | Async runtime | The only serious choice. Powers reqwest under the hood. LTS releases ensure stability. Use `features = ["full"]` for a CLI -- compile-time cost is negligible for a single binary and avoids chasing down missing feature flags. | HIGH |
| **futures** | `0.3` | `join_all` for fan-out | tokio's `join!` macro works for a fixed number of futures. For the dynamic fan-out pattern (variable-length vec of prompts), you need `futures::future::join_all`. This is the standard approach. | HIGH |
## HTTP Client
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **reqwest** | `0.13` (latest: 0.13.2) | HTTP POST to ChatJimmy API | De facto standard async HTTP client in Rust. Built on hyper + tokio. The `json` feature enables `.json()` request/response helpers via serde. Handles connection pooling, TLS, timeouts, and custom headers (needed for Origin/Referer spoofing). | HIGH |
- **Custom headers**: ChatJimmy requires `Origin`, `Referer`, and a browser-like `User-Agent`. reqwest's `ClientBuilder` supports all of these via `.default_headers()`.
- **Timeouts**: ChatJimmy buffers full responses before returning, so requests can be slow. Set a generous timeout (120s) via `.timeout()`.
- **Connection pooling**: Create ONE `Client` instance, reuse it across all parallel requests. reqwest pools connections automatically.
- **TLS**: Default rustls backend is fine. No need for native-tls.
- **No streaming**: ChatJimmy does not stream, so no need for reqwest's streaming response features.
## JSON
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **serde** | `1` (latest: 1.0.228) | Serialization framework | Universal standard. No alternative worth considering. Use `features = ["derive"]` for `#[derive(Serialize, Deserialize)]`. | HIGH |
| **serde_json** | `1` (latest: 1.0.149) | JSON parsing and output | The JSON implementation for serde. Used for: (1) building ChatJimmy request bodies, (2) parsing response text to extract stats, (3) outputting structured JSON to stdout. | HIGH |
- Define request/response structs with `#[derive(Serialize, Deserialize)]`
- Use `serde_json::to_string_pretty()` for stdout output (or `to_string()` if piping to other tools)
- Use `serde_json::from_str()` for parsing the stdin JSON array in batch mode
- The ChatJimmy stats block (`<|stats|>...<|/stats|>`) needs manual string extraction before JSON parsing -- serde_json parses the extracted substring
## CLI Argument Parsing
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **clap** | `4` (latest: 4.6.0) | Argument parsing | Dominant CLI parsing crate. The derive API maps cleanly to this project's needs: a positional `prompt` arg, `--system` flag, `--parallel` flag. Auto-generates `--help` and `--version`. | HIGH |
- **pico-args**: Zero dependencies and fast compile, but no auto-help generation, no derive macros. Not worth the DX regression for a project this small.
- **structopt**: Superseded by clap v4 derive. structopt's author merged into clap.
- **argh**: Google's minimalist parser. Less ecosystem support, fewer features. No benefit here.
#[derive(Parser)]
#[command(name = "jimmy-skill", about = "Fast parallel LLM calls via ChatJimmy")]
## Error Handling
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **anyhow** | `1` (latest: 1.0.102) | Application error handling | This is an application, not a library. anyhow gives ergonomic `?` propagation, `.context("what was happening")` for rich error messages, and automatic backtrace capture. The caller (Claude skills) does not need to match on error variants -- it just needs a clear error message. | HIGH |
- Errors go to stderr (not stdout) so they don't corrupt JSON output
- Use `anyhow::Context` to add "what was I doing" context: `client.post(url).send().await.context("ChatJimmy API request failed")?`
- Exit code 1 on error, 0 on success
## Testing
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **Built-in `#[test]`** | -- | Unit tests | Rust's built-in test framework is sufficient. No need for a test framework crate. | HIGH |
| **assert_json_diff** | `2` | JSON assertion helpers | Optional but useful for comparing expected vs actual JSON output without brittle string matching. | MEDIUM |
| **wiremock** | `0.6` | HTTP mocking | For integration tests that verify HTTP request formation without hitting the real ChatJimmy API. Integrates with tokio. | MEDIUM |
- **Unit tests**: JSON serialization/deserialization of request/response types, stats block extraction regex, system prompt truncation logic
- **Integration tests**: Mock the ChatJimmy endpoint with wiremock, verify full request/response cycle including headers
- **No need for**: proptest, criterion (benchmarking), or snapshot testing at this scale
## Build & Distribution
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **cargo** | (system) | Build tool | Standard. No alternatives needed. | HIGH |
## What NOT to Use
| Crate | Why Not |
|-------|---------|
| **hyper** (directly) | Too low-level. reqwest wraps hyper and adds the ergonomics you need (connection pooling, JSON helpers, header management). Using hyper directly means reimplementing what reqwest gives you. |
| **surf** | Alternative HTTP client. Less popular, less maintained, fewer features than reqwest. No reason to choose it. |
| **ureq** | Blocking-only HTTP client. Cannot do async concurrent requests, which is the core requirement for parallel mode. |
| **async-std** | Alternative async runtime. Incompatible with reqwest (which requires tokio). Mixing runtimes causes pain. |
| **structopt** | Deprecated in favor of clap v4 derive. |
| **eyre** / **color-eyre** | anyhow alternatives with colored output. Unnecessary complexity for a CLI whose output is consumed by machines (Claude skills), not humans. |
| **indicatif** | Progress bars. Output must be pure JSON. Progress indicators would corrupt stdout. |
| **colored** / **owo-colors** | Terminal colors. Same reason -- stdout is JSON for machine consumption. |
| **tracing** / **log** | Logging frameworks. This is a short-lived CLI, not a long-running service. Errors go to stderr via anyhow. Debug logging can be added later with `--verbose` if needed, but don't add the dependency upfront. |
| **tower** | Middleware stack (retry, rate limiting, etc.). Over-engineering for a CLI that makes 1-N direct HTTP calls. If retry logic is needed later, implement it as a simple loop. |
| **snafu** | Error handling alternative. More complex than anyhow, designed for large systems with many error contexts. Wrong tool for a small CLI. |
## Complete Cargo.toml Dependencies
## Confidence Summary
| Area | Confidence | Reason |
|------|------------|--------|
| Runtime (tokio) | HIGH | De facto standard, no viable alternative for reqwest-based async |
| HTTP client (reqwest) | HIGH | Dominant crate, v0.13 is current, verified on crates.io |
| JSON (serde + serde_json) | HIGH | Universal standard in Rust, no alternatives worth considering |
| CLI parsing (clap) | HIGH | Dominant crate, v4.6 is current, derive API fits perfectly |
| Error handling (anyhow) | HIGH | Standard recommendation for application code, author is dtolnay |
| Concurrency (futures) | HIGH | `join_all` is the textbook pattern for dynamic fan-out |
| Testing (wiremock) | MEDIUM | Solid crate but version needs verification; may have updated |
| Build config | HIGH | Standard Cargo release profile optimization |
## Sources
- [reqwest on crates.io](https://crates.io/crates/reqwest) -- v0.13.2, 2026-02-06
- [tokio on crates.io](https://crates.io/crates/tokio) -- v1.50.0, 2026-03-03
- [clap on crates.io](https://crates.io/crates/clap) -- v4.6.0
- [serde on crates.io](https://crates.io/crates/serde) -- v1.0.228
- [serde_json on crates.io](https://crates.io/crates/serde_json) -- v1.0.149
- [anyhow on crates.io](https://crates.io/crates/anyhow) -- v1.0.102
- [thiserror on crates.io](https://crates.io/crates/thiserror) -- v2.0.18
- [Reqwest Best Practices (Reintech)](https://reintech.io/blog/reqwest-tutorial-http-client-best-practices-rust)
- [Error Handling: anyhow vs thiserror vs snafu (DEV Community)](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003)
- [Rust Error Handling in CLI Apps (TechnoRely)](https://technorely.com/insights/effective-error-handling-in-rust-cli-apps-best-practices-examples-and-advanced-techniques)
- [Tokio join! docs](https://docs.rs/tokio/latest/tokio/macro.join.html)
- [Async Rust Book: Multiple Futures](https://rust-lang.github.io/async-book/06_multiple_futures/02_join.html)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->

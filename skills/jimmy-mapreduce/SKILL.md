---
name: jimmy-mapreduce
description: Chunk large documents or corpora, map one instruction across chunks with Jimmy, then synthesize the map results with the host agent or jimmy-router. Use for corpus summaries, themes, or distributed extraction.
---

# Jimmy MapReduce

Turn oversized or multi-document work into parallel map calls followed by a provenance-aware synthesis.

Read [the Jimmy CLI contract](references/jimmy-cli.md) before the first call. Follow [the MapReduce protocol](references/mapreduce.md) for chunking, map prompts, reduction, and output shaping.

## Inputs

| Parameter | Required | Default | Contract |
|---|---:|---:|---|
| `documents` | yes | — | Non-empty strings or `{ id, text }` objects; values and IDs are strings; IDs are unique |
| `instruction` | yes | — | Non-empty map objective |
| `chunk_chars` | no | 12000 | Positive integer Unicode-scalar target |
| `overlap_chars` | no | 500 | Integer from 0 through 25% of `chunk_chars` |
| `map_system` | no | — | Appended to the map-worker system instruction |
| `reducer` | no | `agent` | `agent` or `router` |
| `router_url` | conditional | — | Required base URL when reducer is `router` |
| `router_model` | no | `jimmy-router` | Router request model |
| `router_api_key` | no | `unused` | Bearer value for the router request |
| `router_timeout_seconds` | no | 120 | Positive HTTP timeout |
| `max_concurrent` | no | 50 | Positive integer |
| `max_chunks` | no | 200 | Positive cap on total mapped chunks |

## Process

1. **Validate.** Enforce every input contract and reducer requirements. Return a bare `usage` error object before any call on the first invalid input.
2. **Chunk.** Split every document deterministically with the MapReduce protocol. Chunking is complete when every source character is covered and IDs are unique.
3. **Map.** JSON-serialize one prompt per chunk and invoke Jimmy parallel mode once with `--max-concurrent MAX_CONCURRENT --max-iterations 1`.
4. **Reduce.** Synthesize successful map responses with the selected reducer, explicitly accounting for failed chunks and source IDs.
5. **Return.** Emit only `{ "maps", "synthesis", "reducer", "summary" }`. Completion requires one map record per chunk and summary counts that reconcile to the chunk set.

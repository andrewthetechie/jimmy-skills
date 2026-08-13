# Generic mapping protocol

Normalize each input to `{ id, input, system? }`; a string uses its zero-based index as ID. IDs remain unique and preserve input order.

Replace every literal `{input}`, `{id}`, and `{index}` in `prompt_template` with the corresponding data. Merge systems as `SHARED\nPER_INPUT` when both exist. Serialize one CLI item per normalized input and use CLI `--max-iterations` for repeated outputs.

## Shape iterations

- `raw`: preserve each CLI iteration result unchanged.
- `json`: parse the complete trimmed response as JSON. A success is `{ "value", "tokens", "elapsed_ms" }`. A parse failure is `{ "value": null, "error_type": "parse", "error": "invalid JSON", "raw", "tokens", "elapsed_ms" }`. A CLI failure uses `value: null`, `raw: null`, and the CLI `error`, `error_type`, tokens, and elapsed time.

Return one result object per input with `index`, `id`, and an `outputs` array of exactly `iterations` entries. Summary fields are `inputs`, `iterations_per_input`, `total_outputs`, `succeeded`, `api_errors`, and `parse_errors`. `api_errors` includes every non-null CLI error type.

Mapping is complete when `succeeded + api_errors + parse_errors = total_outputs`.

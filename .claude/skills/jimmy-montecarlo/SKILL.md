# jimmy-montecarlo

Send N identical prompts to Jimmy and measure response variance — agreement rate, unique count, length distribution, and stability verdict.

## When to use this skill

Use jimmy-montecarlo when you need to know whether a prompt is reliable before committing Claude tokens to act on its output. Good for:

- **Prompt quality gating** — measure a new prompt's stability before using it in production
- **Prompt comparison** — run two prompts at N=20 each; higher agreement_rate wins
- **Pre-commit regression check** — verify that a system prompt change does not reduce response stability
- **Estimating prompt difficulty** — unstable prompts signal ambiguous instructions or edge-case inputs

Not suited for: tasks where response variation is expected (creative writing, brainstorming) — use jimmy-candidates for those. Not suited for N < 10 (statistically meaningless variance estimates).

## How parallelism works here

jimmy-montecarlo sends ONE prompt item with `--max-iterations N`. The binary sends the identical prompt N times concurrently and returns all N results nested under `output[0].results[0..N-1]`. Claude aggregates all responses inline — no extra Bash calls.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `prompt` | yes | The prompt to test for stability. Sent identically N times via --max-iterations. |
| `n` | yes | Number of samples (minimum 10 for statistically meaningful variance; warn if < 10). |
| `system` | no | System prompt passed as per-item `system` field. Keep under 400 chars. |
| `threshold` | no | Agreement rate threshold for stability verdict. Default: 0.7. Values 0.0–1.0. A prompt where the top response accounts for >= threshold of all samples is "stable". |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 100. Set high — all N samples run in parallel. |

## Output

JSON object with prompt echo, sample count, aggregated metrics, stability verdict, and raw responses:

```json
{
  "prompt": "Classify this as bug/feature/chore: Fix null pointer in UserService",
  "samples": 20,
  "metrics": {
    "unique_responses": 3,
    "agreement_rate": 0.75,
    "top_response": { "value": "bug", "count": 15 },
    "length": { "mean": 4.2, "stddev": 1.8, "min": 3, "max": 9 },
    "response_distribution": { "bug": 15, "feature": 3, "chore": 2 }
  },
  "verdict": "stable",
  "raw_responses": ["bug", "Bug", "bug", "feature", "bug", "bug", "chore", "bug", "bug", "bug", "bug", "feature", "bug", "bug", "feature", "bug", "chore", "bug", "bug", "bug"]
}
```

**Notes:**
- `response_distribution` keys are normalized (lowercase, trimmed, trailing punctuation stripped). `raw_responses` contains the original unnormalized responses.
- `length` stats (mean, stddev, min, max) are computed on raw response character counts, not normalized.
- `agreement_rate` = count of top response / N (not / successful responses). If some samples error, they count as unique empty responses in the denominator.
- `verdict` is `"stable"` if `agreement_rate >= threshold` (default 0.7), `"unstable"` otherwise.
- If N < 10, a `"warning"` field is added to the output with the message `"n < 10 produces statistically unreliable variance estimates; recommend n >= 10"`.
- Output is bare JSON with no markdown fences.

**Denominator note (composed workflows):** `samples` = all N calls including any that errored (errors count as unique empty responses per the `agreement_rate` formula above). If you compose jimmy-montecarlo with jimmy-classify, note that `total_votes` in classify output counts only successfully extracted votes — it is NOT equivalent to `samples`. Do not treat them as the same denominator.

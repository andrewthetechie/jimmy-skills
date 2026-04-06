# jimmy-classify

Fan out N ensemble classification votes to Jimmy with built-in prompt variation and return a majority-vote winner with confidence score.

## When to use this skill

Use jimmy-classify when you need fast, cheap text classification across a fixed set of categories. Good for:

- **Commit message labeling** — classify as "bug fix", "feature", "chore", "refactor", "docs"
- **Content tagging** — assign one of N user-defined categories to text
- **Sentiment classification** — "positive", "negative", "neutral" across a corpus
- **Support ticket routing** — classify into support categories before escalation

Not suited for: multi-label classification (outputs exactly one winner), categories with no clear semantic distinction, tasks requiring reasoning beyond a sentence.

## How parallelism works here

jimmy-classify constructs N JSON items (one per vote), each with a distinct prompt template rephrasing the classification question. `jimmy-skill --parallel` runs all N votes concurrently. Claude aggregates vote counts inline — no extra bash calls.

## Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `text` | yes | The text to classify. Placed in the user message (not system prompt) for every template. |
| `categories` | yes | Array of category label strings. Supports multi-word ("bug fix"), hyphenated ("bug-fix"), and single-token formats. Matching is case-insensitive after normalization. |
| `n` | yes | Number of ensemble votes (minimum 3; recommended 7 for best accuracy). Each vote uses a different prompt template (cycles through 7 built-in templates). |
| `system` | no | Optional role instruction prepended to the classifier system prompt. Keep under 400 chars — category list goes in the user message, not here. |
| `max_concurrent` | no | Maximum simultaneous HTTP requests. Default: 100. (Set high — N votes should all run in parallel.) |

## Output

JSON object with majority-vote classification result:

```json
{
  "classification": "bug fix",
  "confidence": 0.71,
  "votes": { "bug fix": 5, "feature": 1, "chore": 1 },
  "total_votes": 7,
  "raw_responses": [
    "Bug Fix — this is clearly a defect correction.",
    "bug fix",
    "Feature",
    "bug fix — null pointer is a defect",
    "bug fix",
    "chore",
    "bug fix — should be in a patch release"
  ]
}
```

**On ties:** `classification` is an array of tied category strings (e.g., `["bug fix", "feature"]`) and confidence reflects the shared vote proportion.

**On parse failures:** Votes that Jimmy could not match to a known category are omitted from vote counting. If all N votes fail, `classification: null`, `confidence: 0`, `votes: {}`, `total_votes: 0`, plus the `raw_responses` array intact.

**Output format:** Bare JSON with no markdown fences.

**Note on the `system` param:** Keep `system` under ~400 chars and put the category list in the user message to avoid hitting Jimmy's 28K system prompt cap.

**Denominator note (composed workflows):** `total_votes` counts only successfully extracted votes — failed label extractions are excluded. If you compose jimmy-classify with jimmy-montecarlo, note that `samples` in montecarlo output counts all N calls (including errors), while `total_votes` here counts only clean votes. Do not compare these two values as equivalent denominators.

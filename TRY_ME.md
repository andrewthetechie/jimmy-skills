# Try the Jimmy Skills

Copy-paste these prompts into Claude Code to invoke the jimmy-skills and get usable results back. Each prompt is self-contained — just paste it and hit enter.

These skills work best for **volume generation** where Claude picks the winner afterward. Jimmy is fast and cheap but not deeply intelligent — that's the point.

---

## jimmy-candidates

Generate multiple variations of something, then pick the best.

### Write error messages for a CLI tool

> Use /jimmy-candidates with prompt: "Write a short, friendly error message for when a user provides an invalid file path to a CLI tool. One sentence only." and n: 8. Then pick the best 2 and explain why.

### Name a feature

> Use /jimmy-candidates with prompt: "Suggest a name for a developer tool feature that automatically fixes linting errors on save. The name should be 1-3 words, catchy, and developer-friendly." and n: 12. Then rank your top 3.

### Generate commit message options

> Use /jimmy-candidates with prompt: "Write a git commit message for: renamed the User class to Account across 14 files, updated all imports and tests. Follow conventional commits format (type: description). One line only." and n: 6, system: "You are a senior developer. Write concise, conventional commit messages. Output only the commit message, nothing else."

### Draft opening lines for a blog post

> Use /jimmy-candidates with prompt: "Write an opening sentence for a technical blog post about why small language models (7-8B parameters) are underrated for developer tooling. Hook the reader immediately." and n: 10, max_iterations: 3. Pick the top 3 across all results.

### Generate code scaffolding

> Use /jimmy-candidates with prompt: "Generate a Rust struct for a user record with fields: id (u64), email (String), display_name (String), created_at (i64 Unix timestamp), is_active (bool). Include serde derive macros for Serialize and Deserialize." and n: 5, system: "Generate Rust code using serde and derive macros. Output only the struct definition — no impl blocks, no use statements, no explanatory text." Then pick the cleanest implementation and explain what you'd change.

---

## jimmy-validate

Run yes/no checks in parallel. Great for checklists and pre-screening.

### Code review checklist

> Use /jimmy-validate with context: "```python\ndef process_payment(amount, currency, user_id):\n    if amount <= 0:\n        raise ValueError('Invalid amount')\n    rate = get_exchange_rate(currency)\n    final = amount * rate\n    charge_card(user_id, final)\n    send_receipt(user_id, final, currency)\n    return {'status': 'success', 'charged': final}\n```" and questions: ["Does this function validate that currency is not None?", "Does this function handle the case where get_exchange_rate fails?", "Is there a try/except around charge_card?", "Does this function return a consistent response shape on error?", "Is the amount validated as a number?", "Does this function log the transaction?"]

### README quality check

> Use /jimmy-validate with context: "# MyLib\nA fast JSON parser.\n\n## Install\nnpm install mylib\n\n## Usage\nconst parse = require('mylib')\nconst data = parse(jsonString)" and questions: ["Does the README have a project description?", "Is there an installation section?", "Is there a usage example with code?", "Does it mention supported Node.js versions?", "Is there a license section?", "Does it explain what makes this parser fast?", "Is there a contributing guide or link to one?", "Does it have a badge for build status?"]

### PR merge readiness

> Use /jimmy-validate with context: "PR #247: Add rate limiting to /api/search endpoint. Changes: added express-rate-limit middleware, configured to 100 requests per 15 minutes per IP, added 429 response handler, added rate limit headers to responses, no tests added yet, no documentation updates." and questions: ["Does this PR include tests?", "Does this PR update documentation?", "Is the rate limit configurable via environment variables?", "Does the 429 response include a Retry-After header?", "Is the rate limit per-IP appropriate for an API behind a load balancer?", "Are the rate limit headers following RFC 6585?"]

---

## jimmy-transform

Rewrite text in multiple styles or adapt content for different audiences.

### Adapt a technical description for different audiences (one-to-many)

> Use /jimmy-transform with input: "Our API now supports WebSocket connections for real-time data streaming. Clients can subscribe to specific channels and receive push updates with sub-100ms latency. Authentication uses the existing Bearer token flow." and instructions: ["rewrite for a non-technical executive summary in 2 sentences", "rewrite as a bullet-point changelog entry for developers", "rewrite as a customer-facing announcement for a product blog", "rewrite as an internal Slack message to the engineering team", "simplify for a junior developer who has never used WebSockets"]

### Clean up multiple pieces of rough copy (many-to-one)

> Use /jimmy-transform with inputs: ["We've made the dashboard way faster now, like 3x faster loading times and stuff", "Users can now do exports to CSV which was a big ask from enterprise customers for a while", "Fixed that annoying bug where the sidebar would randomly collapse when you resize the window", "Added dark mode support because literally everyone has been asking for it since day one"] and instruction: "Rewrite as a professional product changelog entry. One sentence, past tense, no slang. Start with a verb."

### Tone matrix — same text, five tones

> Use /jimmy-transform with input: "We're deprecating the v1 API on March 15. Please migrate to v2 before then." and instructions: ["rewrite in a warm, empathetic tone that acknowledges the inconvenience", "rewrite in a direct, no-nonsense enterprise tone", "rewrite in a casual, friendly startup tone", "rewrite in a formal legal notice style", "rewrite as an urgent warning with clear consequences of not migrating"]

### Enforce output format with validation

> Use /jimmy-transform with inputs: ["def add(a, b): return a + b", "fn multiply(x: i32, y: i32) -> i32 { x * y }", "const divide = (a, b) => a / b;"] and instruction: "Write a one-sentence docstring for this function. Start with a capital letter. End with a period." and validate: type "both", pattern "^[A-Z]", min_length: 10, max_length: 120, and max_retries: 2. Report any items that failed validation after all retries.

---

## jimmy-summarize

Fast summarization with optional multiple variants to compare.

### Summarize a long error log

> Use /jimmy-summarize with text: "2024-03-15 14:23:01 ERROR [api.auth] Failed to validate JWT token: TokenExpiredError at verify (/app/node_modules/jsonwebtoken/verify.js:152:21). Token issued at 2024-03-14T08:00:00Z, expired at 2024-03-14T20:00:00Z. User ID: usr_4821. Request: POST /api/v2/projects/create. Origin: 10.0.3.47. The token refresh middleware did not trigger because the /api/v2/ prefix was not included in the refresh path whitelist configured in auth.config.js. The whitelist currently contains: /api/v1/*, /dashboard/*, /webhooks/*. This is the 47th occurrence of this error in the last 24 hours, all from the v2 API prefix." and max_iterations: 5, system: "Focus on the root cause and the fix, not the symptoms."

### Summarize meeting notes

> Use /jimmy-summarize with text: "Sprint retro notes March 15: Good - shipped the new onboarding flow two days early, customer support tickets down 30% since last deploy, new hire Sarah ramped up faster than expected on the payments team. Bad - staging environment was down for 6 hours on Tuesday, nobody noticed until QA flagged it, CI pipeline takes 45 minutes now which is blocking rapid iteration, the shared component library has diverged between web and mobile teams. Actions - Jake to investigate CI caching to cut build time, Maria to set up staging health check alerts, schedule a sync between web and mobile teams on component library by end of next week, Sarah to document the payments onboarding she went through so we can improve it for the next hire." and max_iterations: 3

### Compare summary styles

> Use /jimmy-summarize with text: "Kubernetes 1.30 introduces several notable changes. The SidecarContainers feature has graduated to stable, allowing init containers with restartPolicy: Always to run alongside the main container for the pod's lifetime. This is significant for service mesh proxies and log collectors. Additionally, the CEL-based admission control has moved to GA, providing a simpler alternative to admission webhooks for many validation use cases. On the deprecation front, the legacy cloud provider integration code has been removed from the core repository, completing the multi-year effort to externalize cloud-specific code. Resource management improvements include better support for dynamic resource allocation (DRA), now in beta, which provides a more flexible alternative to device plugins for hardware resources like GPUs." and max_iterations: 5, system: "Write the summary as a single tweet-length message (under 280 characters)."

---

## jimmy-classify

Classify text into a fixed set of categories via majority-vote ensemble. More reliable than a single call — N votes cancel out random model noise.

### Label commit messages by type

> Use /jimmy-classify with text: "fix: prevent null pointer exception when user record has no email address" and categories: ["bug fix", "feature", "chore", "refactor", "docs"] and n: 7. Report the classification, confidence score, and vote breakdown.

### Route a support ticket

> Use /jimmy-classify with text: "I was charged twice for my subscription this month. I have two identical line items on my credit card statement and need a refund for the duplicate charge." and categories: ["billing", "technical support", "account access", "feature request", "other"] and n: 9. Use the result to recommend which support queue should handle this ticket.

### Sentiment check across a corpus

> Use /jimmy-classify three times, once per review — text: "Best purchase I've made this year, works exactly as described", categories: ["positive", "negative", "neutral"], n: 7. Then text: "Arrived damaged and customer service was unhelpful", same categories, n: 7. Then text: "Decent product, does what it says, nothing special", same categories, n: 7. Tally the three results and report the overall sentiment distribution.

---

## jimmy-montecarlo

Measure whether a prompt gives consistent results before you rely on it. Agreement rate tells you how reliable the 8B model is for a given task.

### Gate a new classification prompt

> Use /jimmy-montecarlo with prompt: "Classify this git commit as one of: bug fix, feature, chore, refactor, docs. Commit: fix: prevent null pointer exception when user record has no email address. Output only the category label." and n: 20 and threshold: 0.7. Report the verdict, agreement_rate, and top_response. If verdict is "unstable", suggest how to tighten the prompt.

### Compare two prompt phrasings

> Use /jimmy-montecarlo twice with n: 20 each. First run: prompt "Is this commit a bug fix? Answer yes or no only. Commit: fix null pointer in UserService when email is missing." Second run: prompt "Classify: bug fix or not a bug fix. Commit: fix null pointer in UserService when email is missing. Output the label only." Compare the agreement_rate from both runs and recommend which phrasing to use in production.

### Regression-check a system prompt change

> Run /jimmy-montecarlo with prompt: "Summarize this error in one sentence: TokenExpiredError at /api/v2/projects/create for user usr_4821, token expired 6 hours ago." and n: 15. Record the agreement_rate. Then run again with a modified system prompt. If the new agreement_rate drops more than 0.1 from the baseline, flag the system prompt change as a regression risk.

---

## jimmy-search

Generate candidate solutions and test each against a shell oracle. Finds working solutions without reasoning — just generate, test, and pick the winner.

### Find a regex that matches version strings

> Use /jimmy-search with problem: "Write a regex that matches semantic version strings like 1.0.0, 2.13.4, and 10.0.1 but rejects 1.0, v1.0.0, and 1.0.0.0. Output only the regex pattern, no explanation." and test_cases: [{ "command": "echo '2.13.4' | grep -qE $CANDIDATE_FILE", "expected_exit": 0 }, { "command": "echo 'v1.0.0' | grep -qE $CANDIDATE_FILE", "expected_exit": 1 }] and n: 10 and type: "regex". Report all candidates with a perfect pass rate.

### Find a shell one-liner to extract emails

> Use /jimmy-search with problem: "Write a grep command that extracts all email addresses from a file passed as the first argument. Output only the email addresses, one per line. No flags other than what is needed." and test_cases: [{ "command": "echo 'Contact us at hello@example.com or support@test.org for help' > /tmp/test_emails.txt && bash $CANDIDATE_FILE /tmp/test_emails.txt | grep -q 'hello@example.com'", "expected_exit": 0 }] and n: 8 and type: "shell". Pick the simplest passing candidate.

### Generate a passing SQL query

> Use /jimmy-search with problem: "Write a SQL SELECT query that returns users who registered in the last 30 days and have a verified email. Table: users(id, email, email_verified, created_at). email_verified is a boolean column." and test_cases: [{ "command": "grep -qi 'SELECT' $CANDIDATE_FILE && grep -qi 'WHERE' $CANDIDATE_FILE && grep -qi 'email_verified' $CANDIDATE_FILE", "expected_exit": 0 }] and n: 6 and type: "sql". Report the top candidate and explain any issues with lower-scoring ones.

---

## jimmy-testdata

Generate structured test fixtures from a schema. Great for seeding test databases and unit test setup without hand-writing JSON.

### Seed a user table for tests

> Use /jimmy-testdata with schema: { "id": "uuid", "username": "string", "email": "email", "age": "int", "active": "bool", "created_at": "date" } and n: 10. Report how many fixtures succeeded and insert the results into the test database seed file.

### Generate product catalog fixtures

> Use /jimmy-testdata with schema: { "sku": "string", "name": "string", "price": "float", "in_stock": "bool", "product_url": "url" } and n: 15. Use the fixtures as seed data for a product listing page integration test.

### Edge-case coverage for a signup form

> Use /jimmy-testdata with schema: { "username": "string", "email": "email", "age": "int", "active": "bool" } and n: 8 and edge_case: true. Review the generated fixtures and identify which input paths (empty strings, max integers, malformed emails) would expose validation gaps in the signup form handler.

---

## jimmy-fuzz

Generate adversarial payload variants for manual security testing. All output is inert data — review before use, never execute automatically.

### XSS payloads for a comment field

> Use /jimmy-fuzz with attack_surface: "a comment input field that renders HTML in a user profile page" and attack_types: ["xss"] and n: 10. List the payloads and note which ones look highest risk based on the severity field. Do not execute any payload automatically.

### Multi-category login form audit

> Use /jimmy-fuzz with attack_surface: "a login form with username and password POST parameters" and attack_types: ["sqli", "xss", "command_injection"] and n: 5. Organize the results by severity (critical first, then high) and summarize which attack categories pose the greatest risk to this endpoint.

### Path traversal coverage for a file download endpoint

> Use /jimmy-fuzz with attack_surface: "a /download?file= query parameter that reads files from a server directory" and attack_types: ["path_traversal", "ssrf"] and n: 8. Flag any payloads that could be immediately testable in a browser by pasting directly into the URL bar, and note which ones would require a proxy tool like Burp Suite.

---

## Tips for Getting Good Results

1. **Be specific in your prompts.** "Write a tagline" produces filler. "Write a one-sentence tagline under 10 words for a CLI tool" produces usable output.

2. **Use system prompts for constraints.** Adding `system: "Output only the result. No preamble, no explanation."` dramatically reduces chatty filler from the 8B model.

3. **Generate more, filter after.** The whole point is that Jimmy is free. Generate 10 candidates instead of 3, then let Claude pick the best.

4. **Jimmy is not Claude.** Don't ask for complex reasoning, multi-step analysis, or precise instruction-following. Use Jimmy for volume: candidates, rewrites, quick validation, summaries. Use Claude for judgment.

5. **Crank max_iterations for comparison.** When you want to compare multiple takes on the same prompt (especially summaries or transforms), set `max_iterations: 5` or higher rather than duplicating items.

6. **Validate results need Claude.** The validate skill parses YES/NO reliably (100% parse rate), but the 8B model only gets ~40-60% of nuanced technical questions right. Use it for cheap pre-screening, then have Claude review the flagged items.

7. **Match the skill to the pattern.** The v1.2 skills cover three patterns: *aggregation* (jimmy-classify, jimmy-montecarlo — run many and tally), *oracle search* (jimmy-search — generate and test), and *adversarial data* (jimmy-fuzz, jimmy-testdata — generate structured content). Pick the pattern first, then the skill.

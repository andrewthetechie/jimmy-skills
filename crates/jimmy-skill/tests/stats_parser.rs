use jimmy_skill::api::{build_request, parse_response, truncate_system_prompt};
use jimmy_skill::output::TokenCounts;

fn zero_tokens() -> TokenCounts {
    TokenCounts {
        prompt: 0,
        completion: 0,
        total: 0,
    }
}

#[test]
fn test_valid_stats_parsing() {
    let raw = r#"Hello<|stats|>{"prefill_tokens":10,"decode_tokens":20,"total_tokens":30}<|/stats|>"#;
    let (text, tokens) = parse_response(raw);
    assert_eq!(text, "Hello");
    assert_eq!(tokens.prompt, 10);
    assert_eq!(tokens.completion, 20);
    assert_eq!(tokens.total, 30);
}

#[test]
fn test_missing_stats_block() {
    let raw = "Hello world";
    let (text, tokens) = parse_response(raw);
    assert_eq!(text, "Hello world");
    assert_eq!(tokens, zero_tokens());
}

#[test]
fn test_malformed_stats_json() {
    let raw = "Hello<|stats|>not json<|/stats|>";
    let (text, tokens) = parse_response(raw);
    // returns full raw string with zero counts when JSON is malformed
    assert_eq!(text, raw);
    assert_eq!(tokens, zero_tokens());
}

#[test]
fn test_stats_rsplit_picks_last() {
    // If LLM output contains <|stats|>, rsplit_once should pick the LAST one (the server-appended one)
    let raw = r#"fake<|stats|>bad<|/stats|>real<|stats|>{"prefill_tokens":1,"decode_tokens":2,"total_tokens":3}<|/stats|>"#;
    let (text, tokens) = parse_response(raw);
    assert_eq!(text, "fake<|stats|>bad<|/stats|>real");
    assert_eq!(tokens.prompt, 1);
    assert_eq!(tokens.completion, 2);
    assert_eq!(tokens.total, 3);
}

#[test]
fn test_system_prompt_truncation() {
    let long_prompt = "a".repeat(30_000);
    let truncated = truncate_system_prompt(&long_prompt);
    assert_eq!(truncated.chars().count(), 28_000);
}

#[test]
fn test_system_prompt_no_truncation() {
    let short_prompt = "a".repeat(100);
    let result = truncate_system_prompt(&short_prompt);
    assert_eq!(result, short_prompt);
    assert_eq!(result.len(), 100);
}

#[test]
fn test_build_request_shape() {
    let req = build_request("hello", Some("be nice"));
    assert_eq!(req.messages.len(), 1);
    assert_eq!(req.messages[0].role, "user");
    assert_eq!(req.messages[0].content, "hello");
    assert_eq!(req.chat_options.selected_model, "llama3.1-8B");
    assert_eq!(req.chat_options.system_prompt, "be nice");
    assert_eq!(req.chat_options.top_k, 8);
    assert!(req.attachment.is_none());

    // Verify JSON serialization has correct field names
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains(r#""chatOptions""#), "chatOptions rename missing: {json}");
    assert!(json.contains(r#""selectedModel""#), "selectedModel rename missing: {json}");
    assert!(json.contains(r#""systemPrompt""#), "systemPrompt rename missing: {json}");
    assert!(json.contains(r#""topK""#), "topK rename missing: {json}");
}

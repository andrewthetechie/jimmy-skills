use std::time::Duration;

use jimmy_skill::api::{build_request, parse_response};
use jimmy_skill::client::{build_client, send_request_to, JimmyError};
use jimmy_skill::output::TokenCounts;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_successful_request_returns_json() {
    let mock_server = MockServer::start().await;
    let response_body = r#"Hello world<|stats|>{"prefill_tokens":5,"decode_tokens":10,"total_tokens":15}<|/stats|>"#;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let raw = send_request_to(&client, &url, &request).await.unwrap();
    let (text, tokens) = parse_response(&raw);

    assert_eq!(text, "Hello world");
    assert_eq!(tokens.prompt, 5);
    assert_eq!(tokens.completion, 10);
    assert_eq!(tokens.total, 15);
}

#[tokio::test]
async fn test_request_body_shape() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let _ = send_request_to(&client, &url, &request).await;

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["messages"][0]["role"], "user");
    assert_eq!(body["messages"][0]["content"], "hello");
    assert_eq!(body["chatOptions"]["selectedModel"], "llama3.1-8B");
    assert_eq!(body["chatOptions"]["topK"], 8);
}

#[tokio::test]
async fn test_required_headers() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(header("Origin", "https://chatjimmy.ai"))
        .and(header("Referer", "https://chatjimmy.ai/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let result = send_request_to(&client, &url, &request).await;
    // If headers are missing the mock won't match and wiremock returns 404
    assert!(result.is_ok(), "Request failed (headers likely missing): {:?}", result.err().map(|e| e.message().to_string()));
}

#[tokio::test]
async fn test_user_agent_contains_mozilla() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let _ = send_request_to(&client, &url, &request).await;

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let ua = requests[0]
        .headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ua.contains("Mozilla"),
        "User-Agent should contain 'Mozilla', got: {}",
        ua
    );
}

#[tokio::test]
async fn test_system_prompt_in_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", Some("be nice"));
    let url = format!("{}/api/chat", mock_server.uri());
    let _ = send_request_to(&client, &url, &request).await;

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["chatOptions"]["systemPrompt"], "be nice");
}

#[tokio::test]
async fn test_api_error_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let result = send_request_to(&client, &url, &request).await;

    assert!(matches!(result, Err(JimmyError::Api(_))), "Expected Api error, got: {:?}", result.ok().map(|_| "Ok"));
}

#[tokio::test]
async fn test_timeout_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("slow")
                .set_delay(Duration::from_secs(5)),
        )
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(100))
        .build()
        .unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let result = send_request_to(&client, &url, &request).await;

    assert!(matches!(result, Err(JimmyError::Timeout(_))), "Expected Timeout error");
}

#[tokio::test]
async fn test_missing_stats_in_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Hello"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let raw = send_request_to(&client, &url, &request).await.unwrap();
    let (text, tokens) = parse_response(&raw);

    assert_eq!(text, "Hello");
    assert_eq!(tokens, TokenCounts { prompt: 0, completion: 0, total: 0 });
}

#[tokio::test]
async fn test_non_200_status_codes() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Too Many Requests"))
        .mount(&mock_server)
        .await;

    let client = build_client().unwrap();
    let request = build_request("hello", None);
    let url = format!("{}/api/chat", mock_server.uri());
    let result = send_request_to(&client, &url, &request).await;

    assert!(matches!(result, Err(JimmyError::Api(_))), "Expected Api error for 429");
}

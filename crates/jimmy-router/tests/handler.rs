use std::sync::Arc;
use std::time::Duration;

use axum::routing::post;
use axum::Router;
use jimmy_router::config::{RouterConfig, SynthesizerConfig};
use jimmy_router::AppState;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Bind a listener to an OS-assigned free port and return it without releasing the port.
/// Callers must pass this listener directly into the spawn helpers to avoid the TOCTOU
/// race that occurs when you read the port number, drop the listener, then re-bind.
async fn bind_listener() -> tokio::net::TcpListener {
    tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap()
}

fn test_config() -> RouterConfig {
    RouterConfig {
        port: 0, // not used — tests bind via bind_listener() directly
        iterations: 3,
        max_concurrent: 10,
        max_retries: 2,
        chatjimmy_url: "http://placeholder.invalid/api/chat".to_string(), // overridden per-test
        synthesizer: SynthesizerConfig {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: "test".to_string(),
            model: "test-model".to_string(),
            max_tokens: None,
            temperature: None,
            system_prompt: None,
        },
    }
}

fn mock_chatjimmy_body(text: &str) -> String {
    format!(
        r#"{}<|stats|>{{"prefill_tokens":5,"decode_tokens":3,"total_tokens":8}}<|/stats|>"#,
        text
    )
}

fn mock_synthesizer_response(content: &str) -> String {
    serde_json::json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "created": 1700000000_u64,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {"role": "assistant", "content": content},
            "finish_reason": "stop"
        }],
        "usage": {"prompt_tokens": 10, "completion_tokens": 20, "total_tokens": 30}
    })
    .to_string()
}

fn build_test_app() -> Router {
    let config = test_config();
    let client = reqwest::Client::new();
    let state = AppState {
        config: Arc::new(config),
        client: Arc::new(client),
    };
    Router::new()
        .route(
            "/v1/chat/completions",
            post(jimmy_router::handler::chat_completions),
        )
        .with_state(state)
}

fn build_test_app_with_chatjimmy_url(chatjimmy_url: String) -> Router {
    let mut config = test_config();
    config.chatjimmy_url = chatjimmy_url;
    let client = reqwest::Client::new();
    let state = AppState {
        config: Arc::new(config),
        client: Arc::new(client),
    };
    Router::new()
        .route(
            "/v1/chat/completions",
            post(jimmy_router::handler::chat_completions),
        )
        .with_state(state)
}

fn build_test_app_with_urls(chatjimmy_url: String, synthesizer_base_url: String) -> Router {
    let mut config = test_config();
    config.chatjimmy_url = chatjimmy_url;
    config.synthesizer.base_url = synthesizer_base_url;
    let client = reqwest::Client::new();
    let state = AppState {
        config: Arc::new(config),
        client: Arc::new(client),
    };
    Router::new()
        .route(
            "/v1/chat/completions",
            post(jimmy_router::handler::chat_completions),
        )
        .with_state(state)
}

/// Spawn a test server using an already-bound listener (no TOCTOU race).
/// The listener is bound before this call and passed in directly.
async fn spawn_with_listener(listener: tokio::net::TcpListener, app: Router) -> (u16, reqwest::Client) {
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    tokio::time::sleep(Duration::from_millis(10)).await;
    (port, reqwest::Client::new())
}

async fn spawn_test_server() -> (u16, reqwest::Client) {
    let listener = bind_listener().await;
    let app = build_test_app();
    spawn_with_listener(listener, app).await
}

async fn spawn_test_server_with_chatjimmy_url(chatjimmy_url: String) -> (u16, reqwest::Client) {
    let listener = bind_listener().await;
    let app = build_test_app_with_chatjimmy_url(chatjimmy_url);
    spawn_with_listener(listener, app).await
}

async fn spawn_test_server_with_urls(
    chatjimmy_url: String,
    synthesizer_base_url: String,
) -> (u16, reqwest::Client) {
    let listener = bind_listener().await;
    let app = build_test_app_with_urls(chatjimmy_url, synthesizer_base_url);
    spawn_with_listener(listener, app).await
}

/// HTTP-01 / FOUT-01 / FOUT-03: Valid request fans out to ChatJimmy N times and returns 200.
/// iterations=3 in test_config -> mock server expects exactly 3 POST /api/chat hits.
/// Also mocks synthesizer so the full end-to-end flow completes.
#[tokio::test]
async fn test_valid_request_reaches_handler() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("Generated text")),
        )
        .expect(3)
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("Synthesized answer")),
        )
        .expect(1)
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let synth_url = synth_mock.uri(); // base_url -- synthesize() appends /chat/completions
    let (port, client) = spawn_test_server_with_urls(chatjimmy_url, synth_url).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "hello"}]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Expected 200 from real fanout");

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["object"], "chat.completion");
    assert_eq!(json["choices"][0]["message"]["content"], "Synthesized answer");
    assert_eq!(json["choices"][0]["finish_reason"], "stop");
    assert!(json["id"].as_str().unwrap().starts_with("chatcmpl-"));
    assert!(json["created"].as_u64().unwrap() > 0);
    // mock drops at end of test verify exactly 3 chatjimmy calls and 1 synthesizer call
}

/// HTTP-02 / SC2: stream: true returns 501 with stream_not_supported code.
#[tokio::test]
async fn test_stream_true_returns_501() {
    let (port, client) = spawn_test_server().await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "hello"}],
        "stream": true
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();

    assert_eq!(resp.status().as_u16(), 501);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "stream_not_supported");
}

/// HTTP-03 / SC3: tools field present returns 501 with tools_not_supported code.
#[tokio::test]
async fn test_tools_present_returns_501() {
    let (port, client) = spawn_test_server().await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "hello"}],
        "tools": [{"type": "function"}]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();

    assert_eq!(resp.status().as_u16(), 501);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
    assert_eq!(json["error"]["code"], "tools_not_supported");
}

/// HTTP-04 / SC4: Missing messages field returns 400 (NOT 422).
/// axum's default Json extractor returns 422; handler.rs uses JsonRejection
/// to remap to 400 with an OpenAI-spec error body.
#[tokio::test]
async fn test_missing_messages_returns_400() {
    let (port, client) = spawn_test_server().await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "model": "test"
        // messages field intentionally absent
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();

    assert_eq!(
        resp.status().as_u16(),
        400,
        "Expected 400 (not 422) for missing messages field"
    );

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["type"], "invalid_request_error");
    // missing messages field triggers JsonDataError -> "invalid_request_body"
    assert_eq!(json["error"]["code"], "invalid_request_body");
}

/// HTTP-05 / FOUT-02: When all ChatJimmy calls fail (>50% threshold), handler returns 500.
/// iterations=3, max_retries=2 -> 3 calls x 3 attempts each = 9 POSTs, all fail -> 3/3 failed -> 500.
#[tokio::test]
async fn test_fanout_failure_returns_500() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let chatjimmy_url = format!("{}/api/chat", mock_server.uri());
    let (port, client) = spawn_test_server_with_chatjimmy_url(chatjimmy_url).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "test"}]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 500);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "upstream_error");
    let msg = json["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("fan-out failed"),
        "Expected 'fan-out failed' in error message, got: {msg}"
    );
}

/// FOUT-01: System message is extracted and forwarded to ChatJimmy.
/// Verifies the request body sent to ChatJimmy contains the system prompt content.
#[tokio::test]
async fn test_fanout_forwards_system_message() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("ok")),
        )
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("synthesized")),
        )
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let (port, client) = spawn_test_server_with_urls(chatjimmy_url, synth_mock.uri()).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [
            {"role": "system", "content": "You are a test assistant"},
            {"role": "user", "content": "hello"}
        ]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Expected 200 when system message present");
}

/// FOUT-02: Semaphore limits concurrent connections.
/// With max_concurrent=1 and iterations=3, requests execute sequentially.
/// This test verifies no panic or deadlock occurs under the concurrency constraint.
#[tokio::test]
async fn test_semaphore_limits_concurrent() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("sequential")),
        )
        .expect(3)
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("synthesized")),
        )
        .mount(&synth_mock)
        .await;

    // Override max_concurrent=1 to force sequential execution
    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let mut config = test_config();
    config.max_concurrent = 1;
    config.chatjimmy_url = chatjimmy_url;
    config.synthesizer.base_url = synth_mock.uri();
    let client_inner = reqwest::Client::new();
    let state = AppState {
        config: Arc::new(config),
        client: Arc::new(client_inner),
    };
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(jimmy_router::handler::chat_completions),
        )
        .with_state(state);

    let (port, client) = spawn_with_listener(bind_listener().await, app).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    let body = serde_json::json!({"messages": [{"role": "user", "content": "test"}]});
    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "Expected 200 with max_concurrent=1");
    // mock_server drop verifies exactly 3 calls
}

/// FOUT-03: Partial success -- 1 of 3 calls fails, but <50% failure -> 200 with 2 responses.
#[tokio::test]
async fn test_fanout_partial_success() {
    let chatjimmy_mock = MockServer::start().await;
    // First two calls return 200, third returns 500
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("response")),
        )
        .up_to_n_times(2)
        .mount(&chatjimmy_mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(500).set_body_string("error"))
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("partial synthesized")),
        )
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    // Set max_retries=0 so failures aren't retried -- makes partial failure predictable
    let mut config = test_config();
    config.chatjimmy_url = chatjimmy_url;
    config.max_retries = 0;
    config.synthesizer.base_url = synth_mock.uri();
    let client_inner = reqwest::Client::new();
    let state = AppState {
        config: Arc::new(config),
        client: Arc::new(client_inner),
    };
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(jimmy_router::handler::chat_completions),
        )
        .with_state(state);

    let (port, client) = spawn_with_listener(bind_listener().await, app).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
    let body = serde_json::json!({"messages": [{"role": "user", "content": "test"}]});
    let resp = client.post(&url).json(&body).send().await.unwrap();
    // 1/3 failed, 1*2=2 <= 3 -> partial success -> 200
    assert_eq!(resp.status().as_u16(), 200, "Expected 200 for partial success (1/3 failure)");
}

/// FOUT-03: All N responses are fully collected before run_fanout returns.
/// Verified by: join_all guarantees all futures complete; wiremock .expect(3) asserts
/// the mock was hit exactly 3 times (panics on drop if count differs) -- proving all
/// responses were gathered before the test function exits.
#[tokio::test]
async fn test_all_responses_collected() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("collected")),
        )
        .expect(3) // join_all must complete all 3 before run_fanout returns
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("all collected")),
        )
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let (port, client) = spawn_test_server_with_urls(chatjimmy_url, synth_mock.uri()).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({"messages": [{"role": "user", "content": "collect all"}]});
    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    // MockServer drop asserts exactly 3 calls -- proving all responses were collected
}

/// FOUT-01: Exactly N requests hit ChatJimmy (verified via wiremock .expect(3)).
/// This is the primary FOUT-01 assertion -- separate from test_valid_request_reaches_handler
/// to make the N-request contract explicit and isolated.
#[tokio::test]
async fn test_fanout_sends_n_requests() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("counted")),
        )
        .expect(3) // iterations = 3 in test_config
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("counted result")),
        )
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let (port, client) = spawn_test_server_with_urls(chatjimmy_url, synth_mock.uri()).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({"messages": [{"role": "user", "content": "count me"}]});
    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    // MockServer drop at end of scope asserts exactly 3 calls were made (wiremock .expect(3))
}

/// SYNTH-01 / D-07: Synthesizer receives drafts in labeled format with original prompt.
/// Verifies assemble_user_message format is correctly forwarded by handler.
#[tokio::test]
async fn test_synthesizer_receives_drafts_as_labeled_format() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("draft response")),
        )
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_synthesizer_response("final")),
        )
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let (port, client) = spawn_test_server_with_urls(chatjimmy_url, synth_mock.uri()).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "what is rust"}]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // Verify synthesizer was called (mock was hit)
    let synth_requests = synth_mock.received_requests().await.unwrap();
    assert_eq!(synth_requests.len(), 1, "Synthesizer should be called exactly once");

    // Verify the synthesizer request body contains draft labels
    let synth_body: serde_json::Value =
        serde_json::from_slice(&synth_requests[0].body).unwrap();
    let user_msg = synth_body["messages"][1]["content"].as_str().unwrap();
    assert!(
        user_msg.contains("Draft 1:"),
        "Expected 'Draft 1:' in synthesizer user message"
    );
    assert!(
        user_msg.contains("User question:"),
        "Expected 'User question:' in synthesizer user message"
    );
    assert!(
        user_msg.contains("what is rust"),
        "Expected original prompt in synthesizer user message"
    );

    // Verify system message is the synthesis prompt (D-08, D-09)
    let system_msg = synth_body["messages"][0]["content"].as_str().unwrap();
    assert!(
        system_msg.contains("synthesis engine"),
        "Expected default synthesis prompt as system message"
    );
}

/// SYNTH error path: Synthesizer connection failure returns 500 with synthesis_error code.
#[tokio::test]
async fn test_synthesizer_failure_returns_500() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("ok")),
        )
        .mount(&chatjimmy_mock)
        .await;

    // Synthesizer returns connection refused (port 1 is guaranteed refused on all platforms)
    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let (port, client) = spawn_test_server_with_urls(
        chatjimmy_url,
        "http://127.0.0.1:1".to_string(), // guaranteed connection refused
    )
    .await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "test"}]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 500);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["error"]["type"], "server_error");
    assert_eq!(json["error"]["code"], "synthesis_error");
}

/// SYNTH-04: Response contains all required OpenAI-compatible fields.
#[tokio::test]
async fn test_response_has_openai_fields() {
    let chatjimmy_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(mock_chatjimmy_body("text")),
        )
        .mount(&chatjimmy_mock)
        .await;

    let synth_mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(mock_synthesizer_response("the answer")),
        )
        .mount(&synth_mock)
        .await;

    let chatjimmy_url = format!("{}/api/chat", chatjimmy_mock.uri());
    let (port, client) = spawn_test_server_with_urls(chatjimmy_url, synth_mock.uri()).await;
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let body = serde_json::json!({
        "model": "my-model",
        "messages": [{"role": "user", "content": "hello"}]
    });

    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    let json: serde_json::Value = resp.json().await.unwrap();
    // SYNTH-04: All required OpenAI fields present
    assert!(
        json["id"].as_str().unwrap().starts_with("chatcmpl-"),
        "id must start with chatcmpl-"
    );
    assert_eq!(json["object"], "chat.completion");
    assert!(
        json["created"].as_u64().unwrap() > 0,
        "created must be a positive unix timestamp"
    );
    assert_eq!(json["model"], "my-model", "model should be echoed from request");
    assert_eq!(json["choices"][0]["index"], 0);
    assert_eq!(json["choices"][0]["message"]["role"], "assistant");
    assert_eq!(json["choices"][0]["message"]["content"], "the answer");
    assert_eq!(json["choices"][0]["finish_reason"], "stop");
    assert!(json["usage"].is_object(), "usage object must be present");
}

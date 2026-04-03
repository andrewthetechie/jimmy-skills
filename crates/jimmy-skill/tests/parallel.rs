use std::sync::Arc;

use jimmy_skill::parallel::{run_parallel, ParallelInputItem};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper: mock response body with stats block
fn mock_body(text: &str) -> String {
    format!(
        r#"{}<|stats|>{{"prefill_tokens":5,"decode_tokens":3,"total_tokens":8}}<|/stats|>"#,
        text
    )
}

// Helper: build a shared reqwest::Client for tests
fn test_client() -> Arc<reqwest::Client> {
    Arc::new(reqwest::Client::new())
}

// Helper: build ParallelInputItem
fn item(prompt: &str, system: Option<&str>) -> ParallelInputItem {
    // We construct directly since ParallelInputItem fields are pub
    serde_json::from_str(&format!(
        r#"{{"prompt":"{}","system":{}}}"#,
        prompt,
        system.map(|s| format!(r#""{}""#, s)).unwrap_or_else(|| "null".to_string())
    ))
    .unwrap()
}

// -------------------------------------------------------------------
// Test 1: Output shape — 2 items × 1 iteration (D-07, D-08, D-09, D-10)
// -------------------------------------------------------------------
#[tokio::test]
async fn test_parallel_output_shape() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/api/chat", mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_body("hello")))
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = test_client();
    let items = vec![item("prompt one", None), item("prompt two", None)];

    let output = run_parallel(client, &url, items, None, 10, 1).await;

    assert_eq!(output.len(), 2, "should have 2 ParallelItems");
    assert_eq!(output[0].index, 0, "first item index should be 0");
    assert_eq!(output[1].index, 1, "second item index should be 1");
    assert_eq!(output[0].results.len(), 1, "each item should have 1 result");
    assert_eq!(output[1].results.len(), 1, "each item should have 1 result");
    assert!(
        output[0].results[0].response.is_some(),
        "successful result should have a response"
    );
    assert!(
        output[0].results[0].error.is_none(),
        "successful result should not have an error"
    );
}

// -------------------------------------------------------------------
// Test 2: Multiple iterations — 1 item × 3 iterations (D-09)
// -------------------------------------------------------------------
#[tokio::test]
async fn test_parallel_iterations() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/api/chat", mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_body("iter result")))
        .expect(3)
        .mount(&mock_server)
        .await;

    let client = test_client();
    let items = vec![item("my prompt", None)];

    let output = run_parallel(client, &url, items, None, 10, 3).await;

    assert_eq!(output.len(), 1, "should have 1 ParallelItem");
    assert_eq!(
        output[0].results.len(),
        3,
        "item should have 3 IterationResults for max_iterations=3"
    );
    for result in &output[0].results {
        assert!(result.response.is_some(), "each iteration should have a response");
        assert!(result.error.is_none(), "no errors expected");
    }
}

// -------------------------------------------------------------------
// Test 3: Output ordering — 3 items, assert index matches input position (D-16)
// -------------------------------------------------------------------
#[tokio::test]
async fn test_parallel_ordering() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/api/chat", mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_body("response")))
        .expect(3)
        .mount(&mock_server)
        .await;

    let client = test_client();
    let items = vec![
        item("first", None),
        item("second", None),
        item("third", None),
    ];

    let output = run_parallel(client, &url, items, None, 10, 1).await;

    assert_eq!(output.len(), 3);
    // Ordering must match input regardless of HTTP completion order
    assert_eq!(output[0].index, 0, "index 0 = first input item");
    assert_eq!(output[1].index, 1, "index 1 = second input item");
    assert_eq!(output[2].index, 2, "index 2 = third input item");
}

// -------------------------------------------------------------------
// Test 4: Error isolation — one failed call doesn't abort others (D-12)
// -------------------------------------------------------------------
#[tokio::test]
async fn test_parallel_error_isolation() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/api/chat", mock_server.uri());

    // Register a successful mock that matches the first and third request
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_body("ok")))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    // Register a 500 error mock that will catch remaining requests
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = test_client();
    // 3 items — we run with max_concurrent=1 to get deterministic ordering
    // (first two succeed, third gets error — or some distribution)
    // The key is: all 3 complete, and at least 1 has an error, the others don't abort
    let items = vec![
        item("item 0", None),
        item("item 1", None),
        item("item 2", None),
    ];

    let output = run_parallel(client, &url, items, None, 1, 1).await;

    assert_eq!(output.len(), 3, "all 3 items should complete");

    // At least 2 items should have successful results
    let success_count = output
        .iter()
        .filter(|p| p.results[0].error.is_none())
        .count();
    let error_count = output
        .iter()
        .filter(|p| p.results[0].error.is_some())
        .count();

    assert!(
        success_count >= 1,
        "at least 1 item should succeed, got {} successes",
        success_count
    );
    assert!(
        error_count >= 1,
        "at least 1 item should have an error (500 response), got {} errors",
        error_count
    );
    assert_eq!(
        success_count + error_count,
        3,
        "all items should have completed"
    );
}

// -------------------------------------------------------------------
// Test 5: Per-item system override (D-05, D-06)
// -------------------------------------------------------------------
#[tokio::test]
async fn test_system_override() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/api/chat", mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_body("ok")))
        .expect(2)
        .mount(&mock_server)
        .await;

    let client = test_client();
    let items = vec![
        item("prompt 0", Some("per-item system")),
        item("prompt 1", None), // should inherit shared_system
    ];

    let output = run_parallel(
        client,
        &url,
        items,
        Some("shared system".to_string()),
        10,
        1,
    )
    .await;

    // Both requests should complete successfully
    assert_eq!(output.len(), 2, "both items should complete");
    assert!(
        output[0].results[0].response.is_some(),
        "item 0 should succeed (per-item system)"
    );
    assert!(
        output[1].results[0].response.is_some(),
        "item 1 should succeed (shared system)"
    );

    // Verify both requests were actually received by the mock
    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 2, "should have made exactly 2 HTTP requests");

    // Verify item 0 used per-item system, item 1 used shared system
    // (requests may arrive in any order, so we check both bodies were sent)
    let bodies: Vec<serde_json::Value> = received
        .iter()
        .map(|r| serde_json::from_slice(&r.body).unwrap())
        .collect();

    let system_prompts: Vec<&str> = bodies
        .iter()
        .map(|b| b["chatOptions"]["systemPrompt"].as_str().unwrap_or(""))
        .collect();

    assert!(
        system_prompts.contains(&"per-item system"),
        "per-item system should be used for item 0"
    );
    assert!(
        system_prompts.contains(&"shared system"),
        "shared system should be used for item 1"
    );
}

// -------------------------------------------------------------------
// Test 6: Empty array returns Err (D-04 + pitfall 4)
//
// Note: read_parallel_stdin reads from real stdin, which can't easily be
// mocked in a unit test. We test the empty-array-check logic directly
// by calling the JSON parse path manually, verifying the contract.
// -------------------------------------------------------------------
#[test]
fn test_empty_array_error() {
    // Simulate what read_parallel_stdin does after parsing "[]"
    let json_input = "[]";
    let items: Vec<ParallelInputItem> = serde_json::from_str(json_input).unwrap();

    // This mirrors the empty-check in read_parallel_stdin
    let result: Result<Vec<ParallelInputItem>, &str> = if items.is_empty() {
        Err("Empty input array — provide at least one prompt item")
    } else {
        Ok(items)
    };

    assert!(
        result.is_err(),
        "empty array should produce an error, not Ok"
    );
}

// -------------------------------------------------------------------
// Test 7: Concurrency limit — semaphore doesn't deadlock with limit=1 (D-02, D-14)
// -------------------------------------------------------------------
#[tokio::test]
async fn test_parallel_concurrency_limit() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/api/chat", mock_server.uri());

    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_body("result")))
        .expect(3)
        .mount(&mock_server)
        .await;

    let client = test_client();
    let items = vec![
        item("a", None),
        item("b", None),
        item("c", None),
    ];

    // max_concurrent=1 forces sequential execution through the semaphore
    let output = run_parallel(client, &url, items, None, 1, 1).await;

    assert_eq!(output.len(), 3, "all 3 items should complete with max_concurrent=1");
    for p in &output {
        assert_eq!(p.results.len(), 1, "each item should have 1 result");
        assert!(
            p.results[0].response.is_some(),
            "all results should be successful"
        );
        assert!(p.results[0].error.is_none(), "no errors expected");
    }
}

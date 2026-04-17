use std::net::TcpStream;
use std::time::Duration;

use axum::Router;
use jimmy_router::config::{RouterConfig, SynthesizerConfig};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn test_config(port: u16) -> RouterConfig {
    RouterConfig {
        port,
        iterations: 1,
        max_concurrent: 10,
        max_retries: 2,
        chatjimmy_url: "http://placeholder.invalid/api/chat".to_string(),
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

#[tokio::test]
async fn test_server_binds_and_accepts_tcp_connections() {
    let port = free_port();
    let config = test_config(port);

    // Build a minimal app (same layers as main.rs, no routes per D-06)
    let app = Router::new()
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(5),
        ))
        .layer(TraceLayer::new_for_http());
    // Note: AppState not used here — no routes to inject state into in Phase 12

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Spawn server as a background task
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to accept the bind
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify TCP connection succeeds
    let connect_result = TcpStream::connect(format!("127.0.0.1:{port}"));
    assert!(
        connect_result.is_ok(),
        "Expected TCP connection to succeed on port {port}, got: {:?}",
        connect_result.err()
    );
}

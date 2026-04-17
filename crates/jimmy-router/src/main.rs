use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::routing::post;
use axum::Router;
use clap::Parser;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use jimmy_router::config::{load_config, validate_synthesizer};
use jimmy_router::AppState;

#[derive(Parser)]
#[command(name = "jimmy-router", about = "OpenAI-compatible proxy with ChatJimmy fan-out")]
struct Cli {
    /// Path to router.toml config file
    #[arg(long, required = true)]
    config: String,
}

fn build_jimmy_client() -> anyhow::Result<reqwest::Client> {
    use reqwest::header::{HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};

    let mut headers = HeaderMap::new();
    headers.insert(ORIGIN, HeaderValue::from_static("https://chatjimmy.ai"));
    headers.insert(REFERER, HeaderValue::from_static("https://chatjimmy.ai/"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
             AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        ),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(120))
        .build()
        .context("Failed to build HTTP client")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = load_config(&cli.config)
        .with_context(|| format!("Failed to load config from: {}", cli.config))?;

    let client = build_jimmy_client()?;

    validate_synthesizer(&config.synthesizer, &client)
        .await
        .context("Synthesizer unreachable -- check [synthesizer] config in router.toml")?;

    let port = config.port;
    let state = AppState {
        config: Arc::new(config),
        client: Arc::new(client),
    };

    // POST /v1/chat/completions — OpenAI-compatible endpoint
    let app = Router::new()
        .route("/v1/chat/completions", post(jimmy_router::handler::chat_completions))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(300),
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    eprintln!("jimmy-router listening on {addr}");

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}

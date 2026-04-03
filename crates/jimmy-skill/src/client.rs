use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};

use crate::api::ChatJimmyRequest;

pub const API_URL: &str = "https://chatjimmy.ai/api/chat";

pub fn build_client() -> anyhow::Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Origin",
        HeaderValue::from_static("https://chatjimmy.ai"),
    );
    headers.insert(
        "Referer",
        HeaderValue::from_static("https://chatjimmy.ai/"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(Into::into)
}

#[derive(Debug)]
pub enum JimmyError {
    Usage(String),
    Timeout(String),
    Network(String),
    Api(String),
    Parse(String),
}

impl JimmyError {
    pub fn exit_code(&self) -> i32 {
        match self {
            JimmyError::Usage(_) => 1,
            JimmyError::Timeout(_) | JimmyError::Network(_) | JimmyError::Api(_) => 2,
            JimmyError::Parse(_) => 3,
        }
    }

    pub fn error_type(&self) -> &str {
        match self {
            JimmyError::Usage(_) => "usage",
            JimmyError::Timeout(_) => "timeout",
            JimmyError::Network(_) => "network",
            JimmyError::Api(_) => "api",
            JimmyError::Parse(_) => "parse",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            JimmyError::Usage(m)
            | JimmyError::Timeout(m)
            | JimmyError::Network(m)
            | JimmyError::Api(m)
            | JimmyError::Parse(m) => m,
        }
    }

    pub fn classify_reqwest_error(err: &reqwest::Error) -> JimmyError {
        if err.is_timeout() {
            JimmyError::Timeout(format!("Request timed out: {err}"))
        } else if err.is_connect() {
            JimmyError::Network(format!("Connection failed: {err}"))
        } else if err.is_status() {
            JimmyError::Api(format!("API error: {err}"))
        } else {
            JimmyError::Network(format!("Request failed: {err}"))
        }
    }
}

pub fn classify_reqwest_error(err: &reqwest::Error) -> JimmyError {
    JimmyError::classify_reqwest_error(err)
}

pub async fn send_request_to(
    client: &reqwest::Client,
    url: &str,
    request: &ChatJimmyRequest,
) -> Result<String, JimmyError> {
    let response = client
        .post(url)
        .json(request)
        .send()
        .await
        .map_err(|e| classify_reqwest_error(&e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(JimmyError::Api(format!("HTTP {}: {}", status, body)));
    }

    response
        .text()
        .await
        .map_err(|e| JimmyError::Network(format!("Failed to read response body: {e}")))
}

pub async fn send_request(
    client: &reqwest::Client,
    request: &ChatJimmyRequest,
) -> Result<String, JimmyError> {
    send_request_to(client, API_URL, request).await
}

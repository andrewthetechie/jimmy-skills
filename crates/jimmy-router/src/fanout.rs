use std::sync::Arc;

use anyhow::Context;
use futures::future::join_all;
use tokio::sync::Semaphore;

use crate::config::RouterConfig;

#[derive(serde::Serialize)]
struct FanoutRequest {
    messages: Vec<FanoutMessage>,
    #[serde(rename = "chatOptions")]
    chat_options: FanoutChatOptions,
    attachment: Option<()>,
}

#[derive(serde::Serialize)]
struct FanoutMessage {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct FanoutChatOptions {
    #[serde(rename = "selectedModel")]
    selected_model: String,
    #[serde(rename = "systemPrompt")]
    system_prompt: String,
    #[serde(rename = "topK")]
    top_k: i32,
}

fn build_request(prompt: &str, system_prompt: Option<&str>) -> FanoutRequest {
    FanoutRequest {
        messages: vec![FanoutMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        chat_options: FanoutChatOptions {
            selected_model: "llama3.1-8B".to_string(),
            system_prompt: system_prompt.unwrap_or("").to_string(),
            top_k: 8,
        },
        attachment: None,
    }
}

fn strip_stats(raw: &str) -> String {
    match raw.rsplit_once("<|stats|>") {
        Some((text, _)) => text.to_string(),
        None => {
            eprintln!("Warning: response missing <|stats|> block");
            raw.to_string()
        }
    }
}

/// Fan out the user prompt to ChatJimmy N times in parallel.
///
/// Issues `config.iterations` identical HTTP POST requests to ChatJimmy using
/// tokio Semaphore + join_all with per-call retry logic and >50% failure threshold.
pub async fn run_fanout(
    client: &reqwest::Client,
    config: &RouterConfig,
    prompt: &str,
    system: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    run_fanout_to(client, &config.chatjimmy_url, config, prompt, system).await
}

async fn run_fanout_to(
    client: &reqwest::Client,
    url: &str,
    config: &RouterConfig,
    prompt: &str,
    system: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    // reqwest::Client is Arc-based internally; clone() is cheap
    let client = client.clone();
    let sem = Arc::new(Semaphore::new(config.max_concurrent));
    let url = url.to_string();
    let prompt = prompt.to_string();
    let system = system.map(str::to_owned);
    let max_retries = config.max_retries;
    let iterations = config.iterations;

    let futures_vec: Vec<_> = (0..iterations)
        .map(|_| {
            let sem = Arc::clone(&sem);
            let client = client.clone();
            let url = url.clone();
            let prompt = prompt.clone();
            let system = system.clone();

            async move {
                let mut last_err: anyhow::Error = anyhow::anyhow!("no attempts made");
                for attempt in 0..=max_retries {
                    let request = build_request(&prompt, system.as_deref());
                    let permit = sem.acquire().await.expect("semaphore closed");
                    let send_result = client.post(&url).json(&request).send().await;
                    // Release permit BEFORE text() + strip_stats (parallel.rs discipline)
                    drop(permit);
                    match send_result {
                        Ok(resp) if resp.status().is_success() => {
                            let raw = resp
                                .text()
                                .await
                                .context("Failed to read ChatJimmy response body")?;
                            return Ok(strip_stats(&raw));
                        }
                        Ok(resp) => {
                            let status = resp.status();
                            let body = resp.text().await.unwrap_or_default();
                            last_err = anyhow::anyhow!("HTTP {}: {}", status, body);
                        }
                        Err(e) => {
                            last_err =
                                anyhow::Error::from(e).context("ChatJimmy request failed");
                        }
                    }
                    if attempt < max_retries {
                        continue;
                    }
                }
                Err(last_err)
            }
        })
        .collect();

    let results: Vec<anyhow::Result<String>> = join_all(futures_vec).await;

    let failed_count = results.iter().filter(|r| r.is_err()).count();
    // Integer-only threshold: avoids float division (from CONTEXT.md specifics)
    if failed_count * 2 > iterations {
        anyhow::bail!(
            "fan-out failed: {}/{} calls failed after retries",
            failed_count,
            iterations
        );
    }

    let responses: Vec<String> = results.into_iter().filter_map(Result::ok).collect();
    Ok(responses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_stats_removes_block() {
        let raw = "Generated text<|stats|>{\"prefill_tokens\":5,\"decode_tokens\":3,\"total_tokens\":8}<|/stats|>";
        let result = strip_stats(raw);
        assert_eq!(result, "Generated text");
    }

    #[test]
    fn test_strip_stats_missing_block_returns_raw() {
        let raw = "Some text with no stats block";
        let result = strip_stats(raw);
        assert_eq!(result, raw);
    }

    #[test]
    fn test_build_request_with_system() {
        let req = build_request("hello", Some("You are helpful"));
        assert_eq!(req.messages[0].role, "user");
        assert_eq!(req.messages[0].content, "hello");
        assert_eq!(req.chat_options.system_prompt, "You are helpful");
        assert_eq!(req.chat_options.selected_model, "llama3.1-8B");
        assert_eq!(req.chat_options.top_k, 8);
    }

    #[test]
    fn test_build_request_without_system() {
        let req = build_request("hello", None);
        assert_eq!(req.chat_options.system_prompt, "");
    }
}

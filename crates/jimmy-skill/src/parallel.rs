use std::io::Read;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use futures::future::join_all;
use tokio::sync::Semaphore;

use crate::api::{build_request, parse_response};
use crate::client::send_request_to;
use crate::output::{IterationResult, ParallelItem};

#[derive(serde::Deserialize, Debug)]
pub struct ParallelInputItem {
    pub prompt: String,
    pub system: Option<String>,
}

pub fn read_parallel_stdin() -> anyhow::Result<Vec<ParallelInputItem>> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("Failed to read stdin")?;
    let items: Vec<ParallelInputItem> = serde_json::from_str(buf.trim())
        .context("stdin must be a JSON array of {prompt, system?} objects")?;
    if items.is_empty() {
        anyhow::bail!("Empty input array — provide at least one prompt item");
    }
    Ok(items)
}

pub async fn run_parallel(
    client: Arc<reqwest::Client>,
    url: &str,
    items: Vec<ParallelInputItem>,
    shared_system: Option<String>,
    max_concurrent: usize,
    max_iterations: usize,
) -> Vec<ParallelItem> {
    let sem = Arc::new(Semaphore::new(max_concurrent));
    let url = url.to_string();
    let item_count = items.len();

    let futures_vec: Vec<_> = items
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            let sem = Arc::clone(&sem);
            let client = Arc::clone(&client);
            // Per-item system field overrides shared_system (D-05, D-06)
            let item_system = item.system.or_else(|| shared_system.clone());
            let prompt = item.prompt;
            let url = url.clone();

            async move {
                let mut results = Vec::with_capacity(max_iterations);
                for _ in 0..max_iterations {
                    let start = Instant::now();
                    let request = build_request(&prompt, item_system.as_deref());
                    let permit = sem.acquire().await.expect("semaphore closed");
                    let raw_result = send_request_to(&client, &url, &request).await;
                    // Release permit BEFORE parse_response (D-14, pitfall 3)
                    drop(permit);
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    let iteration = match raw_result {
                        Ok(raw) => {
                            let (text, tokens) = parse_response(&raw);
                            IterationResult::success(text, tokens, elapsed_ms)
                        }
                        Err(e) => IterationResult::error(
                            e.message().to_string(),
                            e.error_type(),
                            elapsed_ms,
                        ),
                    };
                    results.push(iteration);
                }
                (idx, results)
            }
        })
        .collect();

    // Pre-allocate output vec by index to guarantee ordering (D-16)
    let mut output: Vec<ParallelItem> = (0..item_count)
        .map(|i| ParallelItem {
            index: i,
            results: Vec::new(),
        })
        .collect();

    let completed = join_all(futures_vec).await;
    for (idx, results) in completed {
        output[idx].results = results;
    }
    output
}


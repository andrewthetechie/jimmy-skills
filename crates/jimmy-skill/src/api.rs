use serde::{Deserialize, Serialize};

use crate::output::TokenCounts;

#[derive(Serialize)]
pub struct ChatJimmyRequest {
    pub messages: Vec<Message>,
    #[serde(rename = "chatOptions")]
    pub chat_options: ChatOptions,
    pub attachment: Option<()>,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatOptions {
    #[serde(rename = "selectedModel")]
    pub selected_model: String,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
    #[serde(rename = "topK")]
    pub top_k: i32,
}

#[derive(Deserialize)]
struct RawStats {
    prefill_tokens: u32,
    decode_tokens: u32,
    total_tokens: u32,
}

pub fn parse_response(raw: &str) -> (String, TokenCounts) {
    match raw.rsplit_once("<|stats|>") {
        Some((text, stats_part)) => {
            let stats_json = stats_part.trim_end_matches("<|/stats|>").trim();
            match serde_json::from_str::<RawStats>(stats_json) {
                Ok(stats) => (
                    text.to_string(),
                    TokenCounts {
                        prompt: stats.prefill_tokens,
                        completion: stats.decode_tokens,
                        total: stats.total_tokens,
                    },
                ),
                Err(_) => {
                    eprintln!("Warning: failed to parse stats block, using zero counts");
                    (raw.to_string(), TokenCounts::zero())
                }
            }
        }
        None => {
            eprintln!("Warning: response missing <|stats|> block, using zero counts");
            (raw.to_string(), TokenCounts::zero())
        }
    }
}

pub fn build_request(prompt: &str, system_prompt: Option<&str>) -> ChatJimmyRequest {
    ChatJimmyRequest {
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        chat_options: ChatOptions {
            selected_model: "llama3.1-8B".to_string(),
            system_prompt: system_prompt.unwrap_or("").to_string(),
            top_k: 8,
        },
        attachment: None,
    }
}

pub fn truncate_system_prompt(system: &str) -> String {
    let original_len = system.chars().count();
    if original_len > 28_000 {
        eprintln!(
            "Warning: system prompt truncated from {} to 28000 chars",
            original_len
        );
        system.chars().take(28_000).collect::<String>()
    } else {
        system.to_string()
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TokenCounts {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

impl TokenCounts {
    pub fn zero() -> Self {
        Self {
            prompt: 0,
            completion: 0,
            total: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JimmyOutput {
    pub response: Option<String>,
    pub tokens: TokenCounts,
    pub elapsed_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
}

impl JimmyOutput {
    pub fn success(response: String, tokens: TokenCounts, elapsed_ms: u64) -> Self {
        Self {
            response: Some(response),
            tokens,
            elapsed_ms,
            error: None,
            error_type: None,
        }
    }

    pub fn error(error: String, error_type: &str, elapsed_ms: u64) -> Self {
        Self {
            response: None,
            tokens: TokenCounts::zero(),
            elapsed_ms,
            error: Some(error),
            error_type: Some(error_type.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IterationResult {
    pub response: Option<String>,
    pub tokens: TokenCounts,
    pub elapsed_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
}

impl IterationResult {
    pub fn success(response: String, tokens: TokenCounts, elapsed_ms: u64) -> Self {
        Self {
            response: Some(response),
            tokens,
            elapsed_ms,
            error: None,
            error_type: None,
        }
    }

    pub fn error(error: String, error_type: &str, elapsed_ms: u64) -> Self {
        Self {
            response: None,
            tokens: TokenCounts::zero(),
            elapsed_ms,
            error: Some(error),
            error_type: Some(error_type.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParallelItem {
    pub index: usize,
    pub results: Vec<IterationResult>,
}

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RouterConfig {
    pub port: u16,
    pub iterations: usize,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_chatjimmy_url")]
    pub chatjimmy_url: String,
    pub synthesizer: SynthesizerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SynthesizerConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

fn default_max_concurrent() -> usize { 10 }
fn default_max_retries() -> usize { 2 }
fn default_chatjimmy_url() -> String { "https://chatjimmy.ai/api/chat".to_string() }

pub fn load_config(path: &str) -> anyhow::Result<RouterConfig> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {path}"))?;
    toml::from_str(&contents)
        .with_context(|| format!("Failed to parse TOML config: {path}"))
}

/// Probe the configured synthesizer at startup.
/// Any HTTP response (2xx, 4xx, 401, 422) = reachable = pass.
/// Only connection refused, DNS failure, or timeout = fail.
pub async fn validate_synthesizer(
    config: &SynthesizerConfig,
    client: &reqwest::Client,
) -> anyhow::Result<()> {
    let url = format!("{}/chat/completions", config.base_url);
    let body = serde_json::json!({
        "model": &config.model,
        "messages": [{"role": "user", "content": "ping"}]
    });

    let result = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .json(&body)
        .send()
        .await;

    match result {
        Ok(_) => Ok(()), // Any HTTP response = synthesizer is reachable
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_synthesizer_connection_refused() {
        let config = SynthesizerConfig {
            base_url: "http://127.0.0.1:1".to_string(), // port 1 = guaranteed connection refused
            api_key: "test".to_string(),
            model: "test".to_string(),
            max_tokens: None,
            temperature: None,
            system_prompt: None,
        };
        let client = reqwest::Client::new();
        let result = validate_synthesizer(&config, &client).await;
        assert!(result.is_err(), "Expected error for connection refused");
    }

    fn minimal_toml() -> &'static str {
        r#"
port = 3000
iterations = 5

[synthesizer]
base_url = "http://localhost:11434/v1"
api_key = "ollama"
model = "llama3.1"
"#
    }

    #[test]
    fn test_parse_valid_config() {
        let config: RouterConfig = toml::from_str(minimal_toml()).unwrap();
        assert_eq!(config.port, 3000);
        assert_eq!(config.iterations, 5);
        assert_eq!(config.synthesizer.base_url, "http://localhost:11434/v1");
        assert_eq!(config.synthesizer.api_key, "ollama");
        assert_eq!(config.synthesizer.model, "llama3.1");
        assert!(config.synthesizer.max_tokens.is_none());
        assert!(config.synthesizer.temperature.is_none());
        assert!(config.synthesizer.system_prompt.is_none());
    }

    #[test]
    fn test_missing_synthesizer_section_fails() {
        let toml_str = "port = 3000\niterations = 5";
        let result: Result<RouterConfig, _> = toml::from_str(toml_str);
        assert!(result.is_err(), "Expected parse error when [synthesizer] is absent");
    }

    #[test]
    fn test_optional_system_prompt_present() {
        let toml_str = r#"
port = 3000
iterations = 5

[synthesizer]
base_url = "http://localhost:11434/v1"
api_key = "ollama"
model = "llama3.1"
system_prompt = "Custom synthesis prompt"
"#;
        let config: RouterConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.synthesizer.system_prompt,
            Some("Custom synthesis prompt".to_string())
        );
    }

    #[test]
    fn test_optional_system_prompt_absent() {
        let config: RouterConfig = toml::from_str(minimal_toml()).unwrap();
        assert!(config.synthesizer.system_prompt.is_none());
    }

    #[test]
    fn test_load_config_missing_file_includes_path() {
        let result = load_config("/nonexistent/path/router.toml");
        let err_msg = format!("{:#}", result.unwrap_err());
        assert!(
            err_msg.contains("/nonexistent/path/router.toml"),
            "Error message should contain the file path, got: {err_msg}"
        );
    }

    #[test]
    fn test_defaults_max_concurrent_and_max_retries() {
        let config: RouterConfig = toml::from_str(minimal_toml()).unwrap();
        assert_eq!(config.max_concurrent, 10, "default max_concurrent should be 10");
        assert_eq!(config.max_retries, 2, "default max_retries should be 2");
    }

    #[test]
    fn test_defaults_chatjimmy_url() {
        let config: RouterConfig = toml::from_str(minimal_toml()).unwrap();
        assert_eq!(
            config.chatjimmy_url,
            "https://chatjimmy.ai/api/chat",
            "default chatjimmy_url should be the real ChatJimmy endpoint"
        );
    }

    #[test]
    fn test_explicit_max_concurrent_and_max_retries() {
        let toml_str = r#"
port = 3000
iterations = 5
max_concurrent = 5
max_retries = 0

[synthesizer]
base_url = "http://localhost:11434/v1"
api_key = "ollama"
model = "llama3.1"
"#;
        let config: RouterConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.max_retries, 0);
    }
}

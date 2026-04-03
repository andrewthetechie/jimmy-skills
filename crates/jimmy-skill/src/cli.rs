use std::io::{self, IsTerminal, Read};

use anyhow::Context;
use clap::Parser;

use crate::api::truncate_system_prompt;

#[derive(Parser, Debug)]
#[command(name = "jimmy-skill", about = "Fast LLM calls via ChatJimmy", version)]
pub struct Cli {
    /// Prompt text (reads from stdin if omitted)
    pub prompt: Option<String>,

    /// System prompt (use @filepath to read from file)
    #[arg(long)]
    pub system: Option<String>,

    /// Parallel mode: reads a JSON array of {prompt, system?} objects from stdin
    #[arg(long)]
    pub parallel: bool,

    /// Max simultaneous HTTP requests (parallel mode only)
    #[arg(long, default_value_t = 10)]
    pub max_concurrent: usize,

    /// Times each prompt is sent to Jimmy (parallel mode only; min 1)
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..))]
    pub max_iterations: u64,
}

pub fn get_prompt(cli_prompt: Option<String>) -> Result<String, String> {
    if let Some(prompt) = cli_prompt {
        return Ok(prompt);
    }

    // No positional arg -- try stdin
    if io::stdin().is_terminal() {
        return Err("No prompt provided. Usage: jimmy-skill \"<prompt>\"".into());
    }

    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("Failed to read stdin: {e}"))?;

    let prompt = buf.trim().to_string();
    if prompt.is_empty() {
        return Err("Empty prompt from stdin".into());
    }

    Ok(prompt)
}

pub fn resolve_system_prompt(raw: Option<&str>) -> anyhow::Result<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };

    let content = if let Some(path) = value.strip_prefix('@') {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read system prompt file: {path}"))?
    } else {
        value.to_string()
    };

    let content = truncate_system_prompt(&content);

    Ok(Some(content))
}

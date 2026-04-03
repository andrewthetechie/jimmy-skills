use std::io::Write;

use clap::Parser;
use jimmy_skill::api::{build_request, parse_response};
use jimmy_skill::cli::{Cli, get_prompt, resolve_system_prompt};
use jimmy_skill::client::{build_client, send_request, API_URL};
use jimmy_skill::output::JimmyOutput;

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.payload_as_str().unwrap_or("internal error");

        // JSON error to stdout -- must not use println! (could panic recursively)
        let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
        let json = format!(
            r#"{{"response":null,"tokens":{{"prompt":0,"completion":0,"total":0}},"elapsed_ms":0,"error":"panic: {}","error_type":"parse"}}"#,
            escaped
        );
        let _ = std::io::stdout().write_all(json.as_bytes());
        let _ = std::io::stdout().write_all(b"\n");
        let _ = std::io::stdout().flush();

        // Human-readable to stderr
        eprintln!("jimmy-skill panic: {}", message);
        if let Some(loc) = info.location() {
            eprintln!("  at {}:{}:{}", loc.file(), loc.line(), loc.column());
        }

        std::process::exit(3);
    }));
}

async fn run(cli: Cli) -> Result<String, (i32, String, String)> {
    // Resolve prompt (CLI-01)
    let prompt = get_prompt(cli.prompt).map_err(|msg| {
        let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), "usage", 0))
            .unwrap_or_else(|_| r#"{"response":null,"tokens":{"prompt":0,"completion":0,"total":0},"elapsed_ms":0,"error":"usage error","error_type":"usage"}"#.to_string());
        (1, json, msg)
    })?;

    // Resolve system prompt (CLI-02)
    let system = resolve_system_prompt(cli.system.as_deref()).map_err(|e| {
        let msg = format!("{e}");
        let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), "usage", 0))
            .unwrap_or_else(|_| r#"{"response":null,"tokens":{"prompt":0,"completion":0,"total":0},"elapsed_ms":0,"error":"system prompt error","error_type":"usage"}"#.to_string());
        (1, json, msg)
    })?;

    // Build HTTP client (API-02, API-05)
    let client = build_client().map_err(|e| {
        let msg = format!("Failed to build HTTP client: {e}");
        let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), "network", 0))
            .unwrap_or_else(|_| r#"{"response":null,"tokens":{"prompt":0,"completion":0,"total":0},"elapsed_ms":0,"error":"client build error","error_type":"network"}"#.to_string());
        (2, json, msg)
    })?;

    // Build API request (API-01)
    let request = build_request(&prompt, system.as_deref());

    // Send request and measure elapsed time (API-04)
    let start = std::time::Instant::now();
    let raw_response = send_request(&client, &request).await.map_err(|e| {
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let msg = e.message().to_string();
        let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), e.error_type(), elapsed_ms))
            .unwrap_or_else(|_| r#"{"response":null,"tokens":{"prompt":0,"completion":0,"total":0},"elapsed_ms":0,"error":"api error","error_type":"api"}"#.to_string());
        (e.exit_code(), json, msg)
    })?;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    // Parse response and extract stats (API-04)
    let (text, tokens) = parse_response(&raw_response);

    // Build success output (OUT-01)
    let output = JimmyOutput::success(text, tokens, elapsed_ms);
    serde_json::to_string(&output).map_err(|e| {
        let msg = format!("Failed to serialize output: {e}");
        let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), "parse", elapsed_ms))
            .unwrap_or_else(|_| r#"{"response":null,"tokens":{"prompt":0,"completion":0,"total":0},"elapsed_ms":0,"error":"serialize error","error_type":"parse"}"#.to_string());
        (3, json, msg)
    })
}

fn main() {
    install_panic_hook();

    let cli = Cli::parse();

    // --- PARALLEL MODE PATH ---
    if cli.parallel {
        // Blocking stdin read BEFORE entering tokio runtime (pitfall 2 avoidance)
        let items = match jimmy_skill::parallel::read_parallel_stdin() {
            Ok(items) => items,
            Err(e) => {
                let msg = format!("{e}");
                let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), "usage", 0))
                    .unwrap_or_else(|_| r#"{"response":null,"tokens":{"prompt":0,"completion":0,"total":0},"elapsed_ms":0,"error":"parse error","error_type":"usage"}"#.to_string());
                println!("{json}");
                eprintln!("jimmy-skill: {msg}");
                std::process::exit(1);
            }
        };

        let shared_system = cli.system.clone();
        let max_concurrent = cli.max_concurrent;
        let max_iterations = cli.max_iterations as usize;

        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        let exit_code = rt.block_on(async {
            let client = match build_client() {
                Ok(c) => std::sync::Arc::new(c),
                Err(e) => {
                    let msg = format!("Failed to build HTTP client: {e}");
                    let json = serde_json::to_string(&JimmyOutput::error(msg.clone(), "network", 0))
                        .unwrap_or_default();
                    println!("{json}");
                    eprintln!("jimmy-skill: {msg}");
                    return 2;
                }
            };

            let results = jimmy_skill::parallel::run_parallel(
                client,
                API_URL,
                items,
                shared_system,
                max_concurrent,
                max_iterations,
            )
            .await;

            match serde_json::to_string(&results) {
                Ok(json) => {
                    println!("{json}");
                    0
                }
                Err(e) => {
                    let msg = format!("Failed to serialize output: {e}");
                    eprintln!("jimmy-skill: {msg}");
                    3
                }
            }
        });

        std::process::exit(exit_code);
    }

    // --- SINGLE MODE PATH (unchanged behavior) ---
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let exit_code = rt.block_on(async {
        match run(cli).await {
            Ok(json) => {
                println!("{json}");
                0
            }
            Err((code, error_json, message)) => {
                println!("{error_json}");
                eprintln!("jimmy-skill: {message}");
                code
            }
        }
    });

    std::process::exit(exit_code);
}

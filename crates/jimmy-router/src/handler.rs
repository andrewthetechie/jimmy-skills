use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::types::{
    ChatCompletionError, ChatCompletionResponse, Choice, ErrorDetail, ResponseMessage, Usage,
};
use crate::{fanout, AppState};
use crate::types::ChatCompletionRequest;

pub async fn chat_completions(
    State(state): State<AppState>,
    payload: Result<Json<ChatCompletionRequest>, JsonRejection>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, Json<ChatCompletionError>)> {
    // Step 1: Handle JSON parse/deserialize failures as 400 (D-08)
    let Json(request) = payload.map_err(|rejection| {
        let code = match &rejection {
            JsonRejection::JsonDataError(_) => "invalid_request_body",
            JsonRejection::JsonSyntaxError(_) => "invalid_json",
            JsonRejection::MissingJsonContentType(_) => "missing_content_type",
            _ => "invalid_request_error",
        };
        (
            StatusCode::BAD_REQUEST,
            Json(ChatCompletionError {
                error: ErrorDetail {
                    message: rejection.body_text(),
                    r#type: "invalid_request_error".to_string(),
                    code: Some(code.to_string()),
                },
            }),
        )
    })?;

    // Step 2: Reject stream: true (D-05 -> 501)
    if request.stream == Some(true) {
        return Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(ChatCompletionError {
                error: ErrorDetail {
                    message: "Streaming is not supported".to_string(),
                    r#type: "invalid_request_error".to_string(),
                    code: Some("stream_not_supported".to_string()),
                },
            }),
        ));
    }

    // Step 3: Reject tools (D-05 -> 501)
    if request.tools.is_some() {
        return Err((
            StatusCode::NOT_IMPLEMENTED,
            Json(ChatCompletionError {
                error: ErrorDetail {
                    message: "Tool calling is not supported".to_string(),
                    r#type: "invalid_request_error".to_string(),
                    code: Some("tools_not_supported".to_string()),
                },
            }),
        ));
    }

    // Step 4a: Extract system message (first role:system — D-04; use .find not .rev().find)
    let system = request
        .messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone());

    // Step 4: Extract prompt (last user message)
    let prompt = request
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    // Step 5: Call fanout
    let responses = fanout::run_fanout(&state.client, &state.config, prompt, system.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatCompletionError {
                    error: ErrorDetail {
                        message: e.to_string(),
                        r#type: "server_error".to_string(),
                        code: Some("upstream_error".to_string()),
                    },
                }),
            )
        })?;

    // D-10: Empty draft set -- fail fast before calling synthesize
    if responses.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChatCompletionError {
                error: ErrorDetail {
                    message: "fan-out returned 0 responses".to_string(),
                    r#type: "server_error".to_string(),
                    code: Some("upstream_error".to_string()),
                },
            }),
        ));
    }

    // Step 6: Synthesize the N draft responses into a single response
    let synthesized = crate::synthesizer::synthesize(
        &state.client,
        &state.config.synthesizer,
        &responses,
        prompt,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ChatCompletionError {
                error: ErrorDetail {
                    message: e.to_string(),
                    r#type: "server_error".to_string(),
                    code: Some("synthesis_error".to_string()),
                },
            }),
        )
    })?;

    // Step 7: Build OpenAI-compatible response
    let model = request.model.unwrap_or_else(|| "jimmy-router".to_string());
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(Json(ChatCompletionResponse {
        id,
        object: "chat.completion".to_string(),
        created,
        model,
        choices: vec![Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant".to_string(),
                content: synthesized,
            },
            finish_reason: "stop".to_string(),
        }],
        // TODO: Aggregate token counts from fanout + synthesizer responses.
        // Returning zeros is a known stub — callers MUST NOT rely on this field.
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    }))
}

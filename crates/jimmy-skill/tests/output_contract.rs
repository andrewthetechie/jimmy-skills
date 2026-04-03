use jimmy_skill::output::{JimmyOutput, TokenCounts};

fn tokens(prompt: u32, completion: u32, total: u32) -> TokenCounts {
    TokenCounts {
        prompt,
        completion,
        total,
    }
}

#[test]
fn test_success_json_shape() {
    let out = JimmyOutput::success("hello".to_string(), tokens(1, 2, 3), 100);
    let json = serde_json::to_string(&out).unwrap();

    assert!(json.contains(r#""response":"hello""#), "response field missing: {json}");
    assert!(
        json.contains(r#""tokens":{"prompt":1,"completion":2,"total":3}"#),
        "tokens field missing or wrong: {json}"
    );
    assert!(json.contains(r#""elapsed_ms":100"#), "elapsed_ms missing: {json}");
}

#[test]
fn test_error_json_shape() {
    let out = JimmyOutput::error("fail".to_string(), "timeout", 50);
    let json = serde_json::to_string(&out).unwrap();

    assert!(json.contains(r#""response":null"#), "response should be null: {json}");
    assert!(json.contains(r#""error":"fail""#), "error field missing: {json}");
    assert!(json.contains(r#""error_type":"timeout""#), "error_type missing: {json}");
    assert!(
        json.contains(r#""tokens":{"prompt":0,"completion":0,"total":0}"#),
        "tokens should be zero: {json}"
    );
}

#[test]
fn test_error_json_has_null_response() {
    let out = JimmyOutput::error("oops".to_string(), "api", 10);
    let json = serde_json::to_string(&out).unwrap();

    // "response":null must be present (not absent)
    assert!(json.contains(r#""response":null"#), "response:null must be present: {json}");
}

#[test]
fn test_success_json_no_error_fields() {
    let out = JimmyOutput::success("ok".to_string(), tokens(5, 10, 15), 200);
    let json = serde_json::to_string(&out).unwrap();

    // success variant must NOT contain "error" key at all
    assert!(!json.contains(r#""error""#), "success JSON must not contain error field: {json}");
}

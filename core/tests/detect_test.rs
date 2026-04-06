#[test]
fn detects_rate_limit_signals() {
    assert!(relay::detect::is_rate_limited("Error: rate limit exceeded"));
    assert!(relay::detect::is_rate_limited("HTTP 429 Too Many Requests"));
    assert!(relay::detect::is_rate_limited("quota exceeded for this model"));
    assert!(relay::detect::is_rate_limited("server is overloaded, try again later"));
    assert!(relay::detect::is_rate_limited("usage limit reached"));
    assert!(relay::detect::is_rate_limited("context window full"));
}

#[test]
fn no_false_positives() {
    assert!(!relay::detect::is_rate_limited("file created successfully"));
    assert!(!relay::detect::is_rate_limited("cargo build finished"));
    assert!(!relay::detect::is_rate_limited("all tests passed"));
    assert!(!relay::detect::is_rate_limited("fn rate_calculator() {}"));
}

#[test]
fn hook_output_detects_rate_limit() {
    let json = r#"{"tool_name":"bash","tool_output":"Error: rate limit exceeded"}"#;
    let detection = relay::detect::check_hook_output(json);
    assert!(detection.is_some());
    let d = detection.unwrap();
    assert_eq!(d.tool_name, "bash");
    assert!(d.signal.contains("rate limit"));
}

#[test]
fn hook_output_clean_returns_none() {
    let json = r#"{"tool_name":"bash","tool_output":"hello world"}"#;
    let detection = relay::detect::check_hook_output(json);
    assert!(detection.is_none());
}

#[test]
fn hook_invalid_json_returns_none() {
    let detection = relay::detect::check_hook_output("not json");
    assert!(detection.is_none());
}

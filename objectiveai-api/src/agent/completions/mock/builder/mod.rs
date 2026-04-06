use super::client::MockToolCall;

/// Generate the builder tool call: Write mock.txt with content "mock".
pub fn write_tool_call(
    rng: &mut impl rand::Rng,
) -> MockToolCall {
    MockToolCall {
        tool_name: "Write".to_string(),
        call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
        arguments: serde_json::json!({
            "file_path": "/mock.txt",
            "content": "mock"
        }).to_string(),
        n_deltas: 1,
    }
}

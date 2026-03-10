mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_get_stack_frames() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("get_stack_frames", json!({})).await?;
    assert!(text.contains("main"), "Stack should contain main frame");

    s.close().await
}

#[tokio::test]
async fn test_get_local_variables() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    for _ in 0..3 {
        s.next_and_wait().await?;
    }

    let text = s.call_with_session("get_local_variables", json!({})).await?;
    assert!(text.contains("Local variables"), "Should return local variables");

    s.close().await
}

#[tokio::test]
async fn test_evaluate_expression() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;
    s.next_and_wait().await?;

    let text = s
        .call_with_session(
            "evaluate_expression",
            json!({
                "expression": "a"
            }),
        )
        .await?;
    assert!(text.contains("10"), "Variable a should be 10, got: {}", text);

    s.close().await
}

#[tokio::test]
async fn test_evaluate_expression_arithmetic() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    for _ in 0..2 {
        s.next_and_wait().await?;
    }

    let text = s
        .call_with_session(
            "evaluate_expression",
            json!({
                "expression": "a + b"
            }),
        )
        .await?;
    assert!(text.contains("30"), "a + b should be 30, got: {}", text);

    s.close().await
}

#[tokio::test]
async fn test_get_registers() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    // Request GP registers by number to avoid parser issues with complex SIMD values
    let text = s
        .call_with_session(
            "get_registers",
            json!({
                "reg_list": ["0", "1", "2", "3", "6", "7", "16"]
            }),
        )
        .await?;
    assert!(text.contains("Registers"), "Should return registers, got: {}", text);

    s.close().await
}

#[tokio::test]
async fn test_get_register_names() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("get_register_names", json!({})).await?;
    assert!(text.contains("Registers"), "Should return register names");

    s.close().await
}

#[tokio::test]
async fn test_read_memory() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;
    s.next_and_wait().await?;

    let text = s
        .call_with_session(
            "read_memory",
            json!({
                "address": "&a",
                "count": 4
            }),
        )
        .await?;
    assert!(text.contains("Memory"), "Should return memory contents, got: {}", text);

    s.close().await
}

#[tokio::test]
async fn test_var_create_and_delete() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;
    s.next_and_wait().await?;

    let text = s
        .call_with_session(
            "var_create",
            json!({
                "expression": "a"
            }),
        )
        .await?;
    assert!(text.contains("Variable object"), "Should create variable object, got: {}", text);

    // Parse var name from JSON response
    let json_str = text
        .strip_prefix("Variable object: ")
        .expect("Response should start with 'Variable object: '");
    let parsed: serde_json::Value = serde_json::from_str(json_str)?;
    let name = parsed["name"].as_str().expect("Response should contain 'name' field").to_string();

    let text = s.call_with_session("var_delete", json!({ "name": name })).await?;
    assert!(text.contains("deleted"), "Should confirm deletion");

    s.close().await
}

#[tokio::test]
async fn test_get_stack_depth() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("get_stack_depth", json!({})).await?;
    assert!(text.contains("Stack depth"), "Should return stack depth");

    s.close().await
}

#[tokio::test]
async fn test_get_frame_info() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s
        .call_with_session(
            "get_frame_info",
            json!({
                "frame_number": 0
            }),
        )
        .await?;
    assert!(
        text.contains("Frame") || text.contains("frame") || text.contains("main"),
        "Should return frame info, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_cli_command() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s
        .call_with_session(
            "cli_command",
            json!({
                "command": "info functions main"
            }),
        )
        .await?;
    assert!(text.contains("Output"), "Should return CLI output");

    s.close().await
}

#[tokio::test]
async fn test_get_changed_registers() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;
    s.next_and_wait().await?;

    let text = s.call_with_session("get_changed_registers", json!({})).await?;
    assert!(text.contains("Changed registers"), "Should return changed registers, got: {}", text);

    s.close().await
}

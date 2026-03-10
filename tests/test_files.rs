mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_list_source_files() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("list_source_files", json!({})).await?;
    assert!(text.contains("Source files"), "Should return source files");

    s.close().await
}

#[tokio::test]
async fn test_get_current_source_file() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("get_current_source_file", json!({})).await?;
    assert!(text.contains("Current source"), "Should return current source info");
    assert!(text.contains("test_program.c"), "Should be in test_program.c");

    s.close().await
}

#[tokio::test]
async fn test_load_file() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s
        .call_with_session(
            "load_file",
            json!({
                "file": common::test_program_path().to_str().unwrap()
            }),
        )
        .await?;
    assert!(text.contains("Loaded file"), "Should confirm file loaded");

    s.close().await
}

#[tokio::test]
async fn test_gdb_set_and_show() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s
        .call_with_session(
            "gdb_set",
            json!({
                "variable": "print elements",
                "value": "100"
            }),
        )
        .await?;
    assert!(text.contains("Variable set"), "Should confirm variable set");

    let text = s
        .call_with_session(
            "gdb_show",
            json!({
                "variable": "print elements"
            }),
        )
        .await?;
    assert!(text.contains("Value"), "Should return value");

    s.close().await
}

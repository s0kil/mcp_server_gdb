mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_step_and_next() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.step_and_wait().await?;
    assert!(!text.is_empty(), "Step should return result");

    let text = s.next_and_wait().await?;
    assert!(!text.is_empty(), "Next should return result");

    s.close().await
}

#[tokio::test]
async fn test_continue_execution() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    s.call_with_session(
        "set_breakpoint",
        json!({
            "file": "test_program.c",
            "line": 45
        }),
    )
    .await?;

    let text = s.call_with_session("continue_execution", json!({})).await?;
    assert!(!text.is_empty(), "Continue should return result");

    s.close().await
}

#[tokio::test]
async fn test_finish_execution() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    for _ in 0..2 {
        s.next_and_wait().await?;
    }
    s.step_and_wait().await?;

    let text = s.call_with_session("finish_execution", json!({})).await?;
    assert!(
        text.contains("Finished") || text.contains("finish"),
        "Should indicate function finished, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_until_execution() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s
        .call_with_session(
            "until_execution",
            json!({
                "location": "test_program.c:45"
            }),
        )
        .await?;
    assert!(text.contains("Until"), "Should indicate until completed");

    s.close().await
}

#[tokio::test]
async fn test_set_exec_arguments() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s
        .call_with_session(
            "set_exec_arguments",
            json!({
                "args": ["arg1", "arg2"]
            }),
        )
        .await?;
    assert!(text.contains("Arguments set"), "Should confirm args set");

    s.close().await
}

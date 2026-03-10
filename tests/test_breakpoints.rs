mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_set_and_get_breakpoints() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s
        .call_with_session(
            "set_breakpoint",
            json!({
                "file": "test_program.c",
                "line": 37
            }),
        )
        .await?;
    assert!(text.contains("Set breakpoint"), "Should confirm breakpoint set");

    let text = s.call_with_session("get_breakpoints", json!({})).await?;
    assert!(text.contains("Breakpoints"), "Should return breakpoints");

    s.close().await
}

#[tokio::test]
async fn test_delete_breakpoint() -> Result<()> {
    let s = common::create_test_session().await?;

    s.call_with_session(
        "set_breakpoint",
        json!({
            "file": "test_program.c",
            "line": 37
        }),
    )
    .await?;

    let text = s
        .call_with_session(
            "delete_breakpoint",
            json!({
                "breakpoints": ["1"]
            }),
        )
        .await?;
    assert!(
        text.contains("deleted") || text.contains("Deleted"),
        "Should confirm deletion, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_conditional_breakpoint() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s
        .call_with_session(
            "set_breakpoint_conditional",
            json!({
                "file": "test_program.c",
                "line": 30,
                "condition": "i == 5"
            }),
        )
        .await?;
    assert!(
        text.contains("breakpoint") || text.contains("Breakpoint") || text.contains("bkpt"),
        "Should set conditional breakpoint, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_temporary_breakpoint() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s
        .call_with_session(
            "set_breakpoint_temporary",
            json!({
                "file": "test_program.c",
                "line": 37
            }),
        )
        .await?;
    assert!(
        text.contains("breakpoint") || text.contains("Breakpoint"),
        "Should set temporary breakpoint, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_enable_disable_breakpoint() -> Result<()> {
    let s = common::create_test_session().await?;

    s.call_with_session(
        "set_breakpoint",
        json!({
            "file": "test_program.c",
            "line": 37
        }),
    )
    .await?;

    let text = s
        .call_with_session(
            "disable_breakpoint",
            json!({
                "breakpoints": ["1"]
            }),
        )
        .await?;
    assert!(
        text.contains("disabled") || text.contains("Disabled"),
        "Should confirm disabled, got: {}",
        text
    );

    let text = s
        .call_with_session(
            "enable_breakpoint",
            json!({
                "breakpoints": ["1"]
            }),
        )
        .await?;
    assert!(
        text.contains("enabled") || text.contains("Enabled"),
        "Should confirm enabled, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_watchpoint() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;
    s.next_and_wait().await?;

    let text = s
        .call_with_session(
            "set_watchpoint",
            json!({
                "expression": "a"
            }),
        )
        .await?;
    assert!(
        text.contains("Watchpoint") || text.contains("watchpoint") || text.contains("wpt"),
        "Should set watchpoint, got: {}",
        text
    );

    s.close().await
}

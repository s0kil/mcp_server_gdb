mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_get_thread_info() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("get_thread_info", json!({})).await?;
    assert!(text.contains("Threads"), "Should return thread info");

    s.close().await
}

#[tokio::test]
async fn test_select_frame() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s
        .call_with_session(
            "select_frame",
            json!({
                "frame_number": 0
            }),
        )
        .await?;
    assert!(text.contains("Selected frame"), "Should confirm frame selection");

    s.close().await
}

#[tokio::test]
async fn test_list_thread_groups() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("list_thread_groups", json!({})).await?;
    assert!(text.contains("Thread groups"), "Should return thread groups");

    s.close().await
}

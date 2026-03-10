mod common;

use anyhow::Result;
use serde_json::json;

#[tokio::test]
async fn test_create_and_close_session() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s.call_with_session("get_session", json!({})).await?;
    assert!(text.contains(&s.session_id), "Response should contain session ID");

    let text = s.call_with_session("close_session", json!({})).await?;
    assert!(text.contains("Closed"), "Should confirm session closed");

    Ok(())
}

#[tokio::test]
async fn test_get_all_sessions() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s.call("get_all_sessions", None).await?;
    assert!(text.contains("Sessions"), "Should return sessions list, got: {}", text);

    s.close().await
}

#[tokio::test]
async fn test_is_session_active_before_run() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s.call_with_session("is_session_active", json!({})).await?;
    assert!(text.contains("Session active:"), "Should return active status, got: {}", text);

    s.close().await
}

#[tokio::test]
async fn test_start_and_stop_debugging() -> Result<()> {
    let s = common::setup_stopped_at_main().await?;

    let text = s.call_with_session("stop_debugging", json!({})).await?;
    assert!(
        text.contains("Stopped") || text.contains("stopped"),
        "Should confirm stop, got: {}",
        text
    );

    s.close().await
}

#[tokio::test]
async fn test_get_working_directory() -> Result<()> {
    let s = common::create_test_session().await?;

    let text = s.call_with_session("get_working_directory", json!({})).await?;
    assert!(text.contains("Working directory:"), "Should return working directory");

    s.close().await
}

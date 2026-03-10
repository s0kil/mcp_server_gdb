#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Result, bail};
use mcp_core::client::{Client, ClientBuilder};
use mcp_core::transport::ClientStdioTransport;
use mcp_core::types::ToolResponseContent;
use serde_json::{Value, json};
use tempfile::TempDir;

const GDB_SETTLE_DELAY: Duration = Duration::from_millis(300);

struct TestFixtures {
    test_program: PathBuf,
    multi_thread: PathBuf,
    _temp_dir: TempDir,
}

static FIXTURES: OnceLock<TestFixtures> = OnceLock::new();

fn compile_fixture(out_dir: &Path, source: &Path, name: &str, extra_args: &[&str]) -> PathBuf {
    let output = out_dir.join(name);
    let mut cmd = Command::new("gcc");
    cmd.args(["-g", "-O0"]);
    cmd.args(extra_args);
    cmd.arg("-o").arg(&output).arg(source);
    let status = cmd.status().expect("gcc not found - required for integration tests");
    assert!(status.success(), "Failed to compile {}", source.display());
    output
}

fn get_fixtures() -> &'static TestFixtures {
    FIXTURES.get_or_init(|| {
        let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let tmp = tempfile::tempdir().expect("Failed to create temp dir");

        TestFixtures {
            test_program: compile_fixture(
                tmp.path(),
                &fixtures_dir.join("test_program.c"),
                "test_program",
                &[],
            ),
            multi_thread: compile_fixture(
                tmp.path(),
                &fixtures_dir.join("multi_thread.c"),
                "multi_thread",
                &["-pthread"],
            ),
            _temp_dir: tmp,
        }
    })
}

pub fn test_program_path() -> &'static Path {
    &get_fixtures().test_program
}

pub fn multi_thread_path() -> &'static Path {
    &get_fixtures().multi_thread
}

/// Wrapper around MCP client + session for cleaner test code.
pub struct GdbTestSession {
    pub client: Client<ClientStdioTransport>,
    pub session_id: String,
}

impl GdbTestSession {
    pub async fn call(&self, tool_name: &str, params: Option<Value>) -> Result<String> {
        call_tool_text(&self.client, tool_name, params).await
    }

    pub async fn call_with_session(&self, tool_name: &str, extra: Value) -> Result<String> {
        let mut params = extra.as_object().cloned().unwrap_or_default();
        params.insert("session_id".to_string(), json!(&self.session_id));
        self.call(tool_name, Some(Value::Object(params))).await
    }

    pub async fn next_and_wait(&self) -> Result<String> {
        let text = self.call_with_session("next_execution", json!({})).await?;
        tokio::time::sleep(GDB_SETTLE_DELAY).await;
        Ok(text)
    }

    pub async fn step_and_wait(&self) -> Result<String> {
        let text = self.call_with_session("step_execution", json!({})).await?;
        tokio::time::sleep(GDB_SETTLE_DELAY).await;
        Ok(text)
    }

    pub async fn close(self) -> Result<()> {
        self.call_with_session("close_session", json!({})).await?;
        Ok(())
    }
}

async fn create_client() -> Result<Client<ClientStdioTransport>> {
    let transport =
        ClientStdioTransport::new(env!("CARGO_BIN_EXE_mcp-server-gdb"), &["--log-level", "error"])?;
    let client = ClientBuilder::new(transport).build();
    client.open().await?;
    client.initialize().await?;
    Ok(client)
}

pub async fn call_tool_text(
    client: &Client<ClientStdioTransport>,
    tool_name: &str,
    params: Option<Value>,
) -> Result<String> {
    let response = client.call_tool(tool_name, params).await?;
    match response.content.first() {
        Some(ToolResponseContent::Text(text_content)) => Ok(text_content.text.clone()),
        _ => bail!("Expected text content in response"),
    }
}

/// Create a session with the test_program loaded.
pub async fn create_test_session() -> Result<GdbTestSession> {
    let client = create_client().await?;
    let text = call_tool_text(
        &client,
        "create_session",
        Some(json!({ "program": test_program_path().to_str().unwrap() })),
    )
    .await?;
    let session_id = text
        .strip_prefix("Created GDB session: ")
        .ok_or_else(|| anyhow::anyhow!("Unexpected create_session response: {}", text))?
        .to_string();
    Ok(GdbTestSession { client, session_id })
}

/// Create a session, set breakpoint at main, start debugging, return stopped at main.
pub async fn setup_stopped_at_main() -> Result<GdbTestSession> {
    let session = create_test_session().await?;
    session
        .call_with_session(
            "set_breakpoint",
            json!({
                "file": "test_program.c",
                "line": 37
            }),
        )
        .await?;
    session.call_with_session("start_debugging", json!({})).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(session)
}

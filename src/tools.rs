use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use mcp_core::tool_text_content;
use mcp_core::types::ToolResponseContent;
use mcp_core_macros::{tool, tool_param};

use crate::gdb::GDBManager;

pub static GDB_MANAGER: LazyLock<Arc<GDBManager>> =
    LazyLock::new(|| Arc::new(GDBManager::default()));

pub fn init_gdb_manager() {
    LazyLock::force(&GDB_MANAGER);
}

#[tool(
    name = "create_session",
    description = "Create a new GDB debugging session with optional parameters,\
                   returns a session ID (UUID) if successful"
)]
pub async fn create_session_tool(
    program: tool_param!(
        Option<PathBuf>,
        description = "if provided, path to the executable to debug"
    ),
    nh: tool_param!(Option<bool>, description = "if provided, do not read ~/.gdbinit file"),
    nx: tool_param!(
        Option<bool>,
        description = "if provided, do not read any .gdbinit files in any directory"
    ),
    quiet: tool_param!(
        Option<bool>,
        description = "if provided, do not print version number on startup"
    ),
    cd: tool_param!(Option<PathBuf>, description = "if provided, change current directory to DIR"),
    bps: tool_param!(
        Option<u32>,
        description = "if provided, set serial port baud rate used for remote debugging"
    ),
    symbol_file: tool_param!(
        Option<PathBuf>,
        description = "if provided, read symbols from SYMFILE"
    ),
    core_file: tool_param!(
        Option<PathBuf>,
        description = "if provided, analyze the core dump COREFILE"
    ),
    proc_id: tool_param!(Option<u32>, description = "if provided, attach to running process PID"),
    command: tool_param!(
        Option<PathBuf>,
        description = "if provided, execute GDB commands from FILE"
    ),
    source_dir: tool_param!(
        Option<PathBuf>,
        description = "if provided, search for source files in DIR"
    ),
    args: tool_param!(
        Option<Vec<OsString>>,
        description = "if provided, arguments to be passed to the inferior program"
    ),
    tty: tool_param!(
        Option<PathBuf>,
        description = "if provided, use TTY for input/output by the program being debugged"
    ),
    gdb_path: tool_param!(Option<PathBuf>, description = "if provided, path to the GDB executable"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let session = GDB_MANAGER
        .create_session(
            program,
            nh,
            nx,
            quiet,
            cd,
            bps,
            symbol_file,
            core_file,
            proc_id,
            command,
            source_dir,
            args,
            tty,
            gdb_path,
        )
        .await?;
    Ok(tool_text_content!(format!("Created GDB session: {}", session)))
}

#[tool(name = "get_session", description = "Get a GDB debugging session by ID")]
pub async fn get_session_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let session = GDB_MANAGER.get_session(&session_id).await?;
    Ok(tool_text_content!(format!("Session: {}", serde_json::to_string(&session)?)))
}

#[tool(name = "get_all_sessions", description = "Get all GDB debugging sessions")]
pub async fn get_all_sessions_tool() -> Result<ToolResponseContent, anyhow::Error> {
    let sessions = GDB_MANAGER.get_all_sessions().await?;
    Ok(tool_text_content!(format!("Sessions: {}", serde_json::to_string(&sessions)?)))
}

#[tool(name = "close_session", description = "Close a GDB debugging session")]
pub async fn close_session_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.close_session(&session_id).await?;
    Ok(tool_text_content!("Closed GDB session".to_string()))
}

#[tool(name = "start_debugging", description = "Start debugging in a session")]
pub async fn start_debugging_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.start_debugging(&session_id).await?;
    Ok(tool_text_content!(format!("Started debugging: {}", ret)))
}

#[tool(name = "stop_debugging", description = "Stop debugging in a session")]
pub async fn stop_debugging_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.stop_debugging(&session_id).await?;
    Ok(tool_text_content!(format!("Stopped debugging: {}", ret)))
}

#[tool(name = "get_breakpoints", description = "Get all breakpoints in the current GDB session")]
pub async fn get_breakpoints_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let breakpoints = GDB_MANAGER.get_breakpoints(&session_id).await?;
    Ok(tool_text_content!(format!("Breakpoints: {}", serde_json::to_string(&breakpoints)?)))
}

#[tool(name = "set_breakpoint", description = "Set a breakpoint in the code")]
pub async fn set_breakpoint_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    file: tool_param!(String, description = "Source file path"),
    line: tool_param!(usize, description = "Line number"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let breakpoint = GDB_MANAGER.set_breakpoint(&session_id, &PathBuf::from(file), line).await?;
    Ok(tool_text_content!(format!("Set breakpoint: {}", serde_json::to_string(&breakpoint)?)))
}

#[tool(name = "delete_breakpoint", description = "Delete one or more breakpoints in the code")]
pub async fn delete_breakpoint_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    breakpoints: tool_param!(
        Vec<String>,
        description = "The array of the breakpoint numbers to delete"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.delete_breakpoint(&session_id, breakpoints).await?;
    Ok(tool_text_content!("Breakpoints deleted".to_string()))
}

#[tool(name = "get_stack_frames", description = "Get stack frames in the current GDB session")]
pub async fn get_stack_frames_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let frames = GDB_MANAGER.get_stack_frames(&session_id).await?;
    Ok(tool_text_content!(format!("Stack frames: {}", serde_json::to_string(&frames)?)))
}

#[tool(
    name = "get_local_variables",
    description = "Get local variables in the current stack frame"
)]
pub async fn get_local_variables_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    frame_id: tool_param!(
        Option<usize>,
        description = "The ID of the stack frame, defaults to 0, the topest frame"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let variables = GDB_MANAGER.get_local_variables(&session_id, frame_id).await?;
    Ok(tool_text_content!(format!("Local variables: {}", serde_json::to_string(&variables)?)))
}

#[tool(name = "get_registers", description = "Get registers in the current GDB session")]
pub async fn get_registers_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    reg_list: tool_param!(Option<Vec<String>>, description = "The array of the registers to get"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let registers = GDB_MANAGER.get_registers(&session_id, reg_list).await?;
    Ok(tool_text_content!(format!("Registers: {}", serde_json::to_string(&registers)?)))
}

#[tool(name = "get_register_names", description = "Get register names in the current GDB session")]
pub async fn get_register_names_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    reg_list: tool_param!(Option<Vec<String>>, description = "The array of the registers to get"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let registers = GDB_MANAGER.get_register_names(&session_id, reg_list).await?;
    Ok(tool_text_content!(format!("Registers: {}", serde_json::to_string(&registers)?)))
}

#[tool(
    name = "read_memory",
    description = "Read the memory in the current GDB session. \
        This command attempts to read all accessible memory regions in the specified range. \
        First, all regions marked as unreadable in the memory map (if one is defined) will be skipped. \
        See Memory Region Attributes. Second, GDB will attempt to read the remaining regions. \
        For each one, if reading full region results in an errors, GDB will try to read a subset of the region. \
        In general, every single memory unit in the region may be readable or not, \
        and the only way to read every readable unit is to try a read at every address, \
        which is not practical. Therefore, GDB will attempt to read all accessible memory units at either beginning \
        or the end of the region, using a binary division scheme. This heuristic works well for reading across \
        a memory map boundary. Note that if a region has a readable range that is neither \
        at the beginning or the end, GDB will not read it.\
        The command will return a JSON object with the following fields: \
            begin: The start address of the memory block, as hexadecimal literal. \
            end: The end address of the memory block, as hexadecimal literal. \
            offset: The offset of the memory block, as hexadecimal literal, relative to the start address passed to -data-read-memory-bytes.\
            contents: The contents of the memory block, in hex bytes."
)]
pub async fn read_memory_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    address: tool_param!(
        String,
        description = "An expression specifying the address of the first addressable memory unit to be read. \
        Complex expressions containing embedded white space should be quoted using the C convention."
    ),
    count: tool_param!(
        usize,
        description =
            "The number of addressable memory units to read. This should be an integer literal."
    ),
    offset: tool_param!(
        Option<isize>,
        description = "The offset relative to address at which to start reading. This should be an integer literal. \
        This option is provided so that a frontend is not required to first evaluate address and \
        then perform address arithmetic itself."
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let memory = GDB_MANAGER.read_memory(&session_id, offset, address, count).await?;
    Ok(tool_text_content!(format!("Memory: {}", serde_json::to_string(&memory)?)))
}

#[tool(name = "continue_execution", description = "Continue program execution")]
pub async fn continue_execution_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.continue_execution(&session_id).await?;
    Ok(tool_text_content!(format!("Continued execution: {}", ret)))
}

#[tool(name = "step_execution", description = "Step into next line")]
pub async fn step_execution_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.step_execution(&session_id).await?;
    Ok(tool_text_content!(format!("Stepped into next line: {}", ret)))
}

#[tool(name = "next_execution", description = "Step over next line")]
pub async fn next_execution_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.next_execution(&session_id).await?;
    Ok(tool_text_content!(format!("Stepped over next line: {}", ret)))
}

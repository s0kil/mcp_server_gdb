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

// --- Chunk 1: Expression Evaluation, Variable Objects, CLI Passthrough ---

#[tool(
    name = "evaluate_expression",
    description = "Evaluate an expression in the current frame. Returns the result as a string. \
                   Use for inspecting variables, calling functions, or computing values."
)]
pub async fn evaluate_expression_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    expression: tool_param!(
        String,
        description = "The expression to evaluate (e.g. 'x+1', 'strlen(s)', 'array[3]')"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.evaluate_expression(&session_id, expression).await?;
    Ok(tool_text_content!(format!("Result: {}", result)))
}

#[tool(
    name = "var_create",
    description = "Create a variable object for watching an expression across steps. \
                   Returns the variable object details including type and value."
)]
pub async fn var_create_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    expression: tool_param!(String, description = "The expression to watch"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.var_create(&session_id, expression).await?;
    Ok(tool_text_content!(format!("Variable object: {}", serde_json::to_string(&result)?)))
}

#[tool(name = "var_delete", description = "Delete a previously created variable object by name.")]
pub async fn var_delete_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    name: tool_param!(String, description = "The name of the variable object to delete"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.var_delete(&session_id, name).await?;
    Ok(tool_text_content!("Variable object deleted".to_string()))
}

#[tool(
    name = "var_list_children",
    description = "List children of a variable object (expand struct fields, array elements). \
                   Use after var_create to inspect complex data structures."
)]
pub async fn var_list_children_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    name: tool_param!(String, description = "The name of the variable object"),
    print_values: tool_param!(
        Option<bool>,
        description = "If true, include values of children (default: true)"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result =
        GDB_MANAGER.var_list_children(&session_id, name, print_values.unwrap_or(true)).await?;
    Ok(tool_text_content!(format!("Children: {}", serde_json::to_string(&result)?)))
}

#[tool(
    name = "cli_command",
    description = "Execute an arbitrary GDB console command. Use as an escape hatch for commands \
                   not covered by other tools (e.g. 'info proc mappings', 'set scheduler-locking on')."
)]
pub async fn cli_command_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    command: tool_param!(String, description = "The GDB console command to execute"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.cli_exec(&session_id, command).await?;
    Ok(tool_text_content!(format!("Output: {}", result)))
}

// --- Chunk 2: Execution Control Extensions ---

#[tool(
    name = "finish_execution",
    description = "Run until the current function returns. Shows the return value."
)]
pub async fn finish_execution_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.finish_execution(&session_id).await?;
    Ok(tool_text_content!(format!("Finished: {}", ret)))
}

#[tool(
    name = "until_execution",
    description = "Run until a specified source location is reached. Useful for running past loops."
)]
pub async fn until_execution_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    location: tool_param!(
        String,
        description = "Location to run until (e.g. 'file.c:42' or 'function_name')"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.until_execution(&session_id, location).await?;
    Ok(tool_text_content!(format!("Until: {}", ret)))
}

#[tool(
    name = "return_execution",
    description = "Force return from the current function, optionally with a specified return value."
)]
pub async fn return_execution_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    expression: tool_param!(Option<String>, description = "Optional return value expression"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.return_execution(&session_id, expression).await?;
    Ok(tool_text_content!(format!("Returned: {}", ret)))
}

#[tool(
    name = "reverse_continue",
    description = "Continue execution in reverse (requires record target or rr). \
                   Runs backward until hitting a breakpoint or the start of recording."
)]
pub async fn reverse_continue_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.reverse_continue(&session_id).await?;
    Ok(tool_text_content!(format!("Reverse continue: {}", ret)))
}

#[tool(
    name = "reverse_step",
    description = "Step one source line backward, entering function calls (requires record target or rr)."
)]
pub async fn reverse_step_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.reverse_step(&session_id).await?;
    Ok(tool_text_content!(format!("Reverse step: {}", ret)))
}

#[tool(
    name = "reverse_next",
    description = "Step one source line backward, stepping over function calls (requires record target or rr)."
)]
pub async fn reverse_next_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.reverse_next(&session_id).await?;
    Ok(tool_text_content!(format!("Reverse next: {}", ret)))
}

#[tool(
    name = "reverse_finish",
    description = "Run backward until entering the current function (requires record target or rr)."
)]
pub async fn reverse_finish_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let ret = GDB_MANAGER.reverse_finish(&session_id).await?;
    Ok(tool_text_content!(format!("Reverse finish: {}", ret)))
}

// --- Chunk 3: Breakpoint Enhancements ---

#[tool(
    name = "set_breakpoint_conditional",
    description = "Set a breakpoint that only triggers when a condition is true. \
                   Example condition: 'i == 42' or 'ptr != NULL'."
)]
pub async fn set_breakpoint_conditional_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    file: tool_param!(String, description = "Source file path"),
    line: tool_param!(usize, description = "Line number"),
    condition: tool_param!(String, description = "Condition expression (e.g. 'i == 42')"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let breakpoint = GDB_MANAGER
        .set_breakpoint_conditional(&session_id, &PathBuf::from(file), line, condition)
        .await?;
    Ok(tool_text_content!(format!(
        "Conditional breakpoint: {}",
        serde_json::to_string(&breakpoint)?
    )))
}

#[tool(
    name = "set_breakpoint_temporary",
    description = "Set a temporary breakpoint that is automatically deleted after the first hit."
)]
pub async fn set_breakpoint_temporary_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    file: tool_param!(String, description = "Source file path"),
    line: tool_param!(usize, description = "Line number"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let breakpoint =
        GDB_MANAGER.set_breakpoint_temporary(&session_id, &PathBuf::from(file), line).await?;
    Ok(tool_text_content!(format!("Temporary breakpoint: {}", serde_json::to_string(&breakpoint)?)))
}

#[tool(
    name = "enable_breakpoint",
    description = "Enable one or more previously disabled breakpoints."
)]
pub async fn enable_breakpoint_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    breakpoints: tool_param!(Vec<String>, description = "Array of breakpoint numbers to enable"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.enable_breakpoint(&session_id, breakpoints).await?;
    Ok(tool_text_content!("Breakpoints enabled".to_string()))
}

#[tool(
    name = "disable_breakpoint",
    description = "Disable one or more breakpoints without deleting them."
)]
pub async fn disable_breakpoint_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    breakpoints: tool_param!(Vec<String>, description = "Array of breakpoint numbers to disable"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.disable_breakpoint(&session_id, breakpoints).await?;
    Ok(tool_text_content!("Breakpoints disabled".to_string()))
}

#[tool(
    name = "set_watchpoint",
    description = "Set a watchpoint on an expression. Triggers when the value is written, read, or accessed \
                   depending on mode. Useful for finding where a variable gets modified."
)]
pub async fn set_watchpoint_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    expression: tool_param!(
        String,
        description = "Expression to watch (e.g. 'my_var', '*0x12345')"
    ),
    mode: tool_param!(
        Option<String>,
        description = "Watch mode: 'write' (default), 'read', or 'access'"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER
        .set_watchpoint(&session_id, expression, mode.unwrap_or_else(|| "write".to_string()))
        .await?;
    Ok(tool_text_content!(format!("Watchpoint: {}", serde_json::to_string(&result)?)))
}

// --- Chunk 4: Disassembly and Memory ---

#[tool(
    name = "disassemble_file",
    description = "Disassemble around a source file location. Returns mixed source and assembly."
)]
pub async fn disassemble_file_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    file: tool_param!(String, description = "Source file path"),
    line: tool_param!(usize, description = "Line number to disassemble around"),
    lines: tool_param!(Option<usize>, description = "Number of source lines to cover"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result =
        GDB_MANAGER.disassemble_file(&session_id, &PathBuf::from(file), line, lines).await?;
    Ok(tool_text_content!(format!("Disassembly: {}", serde_json::to_string(&result)?)))
}

#[tool(
    name = "disassemble_address",
    description = "Disassemble an address range. Returns raw assembly instructions."
)]
pub async fn disassemble_address_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    start_addr: tool_param!(usize, description = "Start address (decimal)"),
    end_addr: tool_param!(usize, description = "End address (decimal)"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.disassemble_address(&session_id, start_addr, end_addr).await?;
    Ok(tool_text_content!(format!("Disassembly: {}", serde_json::to_string(&result)?)))
}

#[tool(
    name = "write_memory",
    description = "Write hex bytes to a memory address. Contents is a hex string (e.g. 'deadbeef')."
)]
pub async fn write_memory_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    address: tool_param!(
        String,
        description = "Memory address expression (e.g. '0x7fffe000' or '&my_var')"
    ),
    contents: tool_param!(String, description = "Hex string of bytes to write (e.g. 'deadbeef')"),
    count: tool_param!(
        Option<usize>,
        description = "Optional number of bytes (defaults to contents length / 2)"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.write_memory(&session_id, address, contents, count).await?;
    Ok(tool_text_content!("Memory written".to_string()))
}

#[tool(
    name = "get_changed_registers",
    description = "List register numbers that changed since the last stop. \
                   Useful for seeing what an instruction modified."
)]
pub async fn get_changed_registers_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.get_changed_registers(&session_id).await?;
    Ok(tool_text_content!(format!("Changed registers: {}", serde_json::to_string(&result)?)))
}

// --- Chunk 5: Thread and Frame Management ---

#[tool(
    name = "get_thread_info",
    description = "Get information about threads. If thread_id is provided, returns info for that thread only; \
                   otherwise returns all threads."
)]
pub async fn get_thread_info_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    thread_id: tool_param!(Option<u64>, description = "Optional thread ID to query"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.get_thread_info(&session_id, thread_id).await?;
    Ok(tool_text_content!(format!("Threads: {}", serde_json::to_string(&result)?)))
}

#[tool(
    name = "select_frame",
    description = "Select a stack frame by number. Subsequent commands like get_local_variables \
                   will operate on this frame."
)]
pub async fn select_frame_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    frame_number: tool_param!(u64, description = "Frame number (0 = innermost/current)"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.select_frame(&session_id, frame_number).await?;
    Ok(tool_text_content!(format!("Selected frame {}", frame_number)))
}

#[tool(
    name = "get_frame_info",
    description = "Get detailed info about a stack frame (function name, file, line, address)."
)]
pub async fn get_frame_info_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    frame_number: tool_param!(Option<u64>, description = "Frame number (default: current frame)"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.get_frame_info(&session_id, frame_number).await?;
    Ok(tool_text_content!(format!("Frame: {}", serde_json::to_string(&result)?)))
}

#[tool(name = "get_stack_depth", description = "Get the total depth of the call stack.")]
pub async fn get_stack_depth_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.get_stack_depth(&session_id).await?;
    Ok(tool_text_content!(format!("Stack depth: {}", result)))
}

#[tool(
    name = "list_thread_groups",
    description = "List thread groups (inferiors/processes). Set list_all to true to see available groups."
)]
pub async fn list_thread_groups_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    list_all: tool_param!(
        Option<bool>,
        description = "If true, list all available groups (default: false)"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.list_thread_groups(&session_id, list_all.unwrap_or(false)).await?;
    Ok(tool_text_content!(format!("Thread groups: {}", serde_json::to_string(&result)?)))
}

// --- Chunk 6: Source and File Management ---

#[tool(
    name = "load_file",
    description = "Load an executable file and its symbols into GDB. \
                   Use this to change the program being debugged."
)]
pub async fn load_file_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    file: tool_param!(String, description = "Path to the executable file"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.load_file(&session_id, &PathBuf::from(&file)).await?;
    Ok(tool_text_content!(format!("Loaded file: {}", file)))
}

#[tool(
    name = "load_symbol_file",
    description = "Load symbols from a separate file, or unload symbols if no file is provided."
)]
pub async fn load_symbol_file_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    file: tool_param!(Option<String>, description = "Path to the symbol file (omit to unload)"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let path = file.as_deref().map(std::path::Path::new);
    GDB_MANAGER.load_symbol_file(&session_id, path).await?;
    Ok(tool_text_content!("Symbol file updated".to_string()))
}

#[tool(
    name = "list_source_files",
    description = "List all source files known to GDB for the loaded program."
)]
pub async fn list_source_files_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.list_source_files(&session_id).await?;
    Ok(tool_text_content!(format!("Source files: {}", serde_json::to_string(&result)?)))
}

#[tool(
    name = "get_current_source_file",
    description = "Get information about the current source file (file name, line, full path)."
)]
pub async fn get_current_source_file_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.get_current_source_file(&session_id).await?;
    Ok(tool_text_content!(format!("Current source: {}", serde_json::to_string(&result)?)))
}

// --- Chunk 7: Remote Debugging and Process Control ---

#[tool(
    name = "target_select",
    description = "Connect to a remote debugging target. \
                   Example: type='remote', parameters='localhost:1234'."
)]
pub async fn target_select_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    target_type: tool_param!(
        String,
        description = "Target type (e.g. 'remote', 'extended-remote')"
    ),
    parameters: tool_param!(
        String,
        description = "Connection parameters (e.g. 'localhost:1234', '/dev/ttyUSB0')"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.target_select(&session_id, target_type, parameters).await?;
    Ok(tool_text_content!(format!("Target: {}", result)))
}

#[tool(
    name = "target_detach",
    description = "Detach from the current target, optionally specifying a process ID."
)]
pub async fn target_detach_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    pid: tool_param!(Option<u32>, description = "Optional process ID to detach from"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.target_detach(&session_id, pid).await?;
    Ok(tool_text_content!("Detached from target".to_string()))
}

#[tool(
    name = "send_signal",
    description = "Send a signal to the debugged program (e.g. 'SIGINT', 'SIGUSR1', '9')."
)]
pub async fn send_signal_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    signal: tool_param!(
        String,
        description = "Signal name or number (e.g. 'SIGINT', 'SIGUSR1', '9')"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.send_signal(&session_id, signal).await?;
    Ok(tool_text_content!(format!("Signal sent: {}", result)))
}

#[tool(
    name = "gdb_set",
    description = "Set a GDB internal variable. Example: variable='print elements', value='100'."
)]
pub async fn gdb_set_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    variable: tool_param!(
        String,
        description = "GDB variable name (e.g. 'print elements', 'pagination')"
    ),
    value: tool_param!(String, description = "Value to set"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.gdb_set(&session_id, variable, value).await?;
    Ok(tool_text_content!("Variable set".to_string()))
}

#[tool(name = "gdb_show", description = "Show the value of a GDB internal variable.")]
pub async fn gdb_show_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    variable: tool_param!(
        String,
        description = "GDB variable name (e.g. 'print elements', 'pagination')"
    ),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.gdb_show(&session_id, variable).await?;
    Ok(tool_text_content!(format!("Value: {}", result)))
}

// --- Chunk 8: Remaining Tools ---

#[tool(
    name = "set_exec_arguments",
    description = "Set the command-line arguments for the next program run."
)]
pub async fn set_exec_arguments_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
    args: tool_param!(Vec<String>, description = "Array of arguments to pass to the program"),
) -> Result<ToolResponseContent, anyhow::Error> {
    GDB_MANAGER.set_exec_arguments(&session_id, args).await?;
    Ok(tool_text_content!("Arguments set".to_string()))
}

#[tool(name = "get_working_directory", description = "Get GDB's current working directory.")]
pub async fn get_working_directory_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let result = GDB_MANAGER.get_working_directory(&session_id).await?;
    Ok(tool_text_content!(format!("Working directory: {}", result)))
}

#[tool(
    name = "is_session_active",
    description = "Check whether the debugged program is still running (has active threads). \
                   Returns true if the program is alive, false if it has exited."
)]
pub async fn is_session_active_tool(
    session_id: tool_param!(String, description = "The ID of the GDB session"),
) -> Result<ToolResponseContent, anyhow::Error> {
    let active = GDB_MANAGER.is_session_active(&session_id).await?;
    Ok(tool_text_content!(format!("Session active: {}", active)))
}

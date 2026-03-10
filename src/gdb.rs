use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::TRANSPORT;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::mi::commands::{
    BreakPointLocation, BreakPointNumber, DisassembleMode, MiCommand, RegisterFormat, WatchMode,
};
use crate::mi::output::{AsyncClass, OutOfBandRecord, ResultClass, ResultRecord, ThreadEvent};
use crate::mi::{GDB, GDBBuilder};
use crate::models::{
    BreakPoint, GDBSession, GDBSessionStatus, Memory, Register, StackFrame, Variable,
};

/// GDB Session Manager
#[derive(Default)]
pub struct GDBManager {
    /// Configuration
    config: Config,
    /// Session mapping table
    sessions: Mutex<HashMap<String, GDBSessionHandle>>,
}

/// GDB Session Handle
struct GDBSessionHandle {
    /// Session information
    info: GDBSession,
    /// GDB instance (per-session lock to avoid blocking other sessions)
    gdb: Arc<Mutex<GDB>>,
    /// OOB handle
    oob_handle: JoinHandle<()>,
    /// Whether the inferior has exited
    program_exited: Arc<AtomicBool>,
}

impl GDBManager {
    /// Check if the program in the session is still alive (not exited)
    fn check_program_alive(handle: &GDBSessionHandle, context: &str) -> AppResult<()> {
        if handle.program_exited.load(Ordering::SeqCst) {
            return Err(AppError::ProgramExited(format!(
                "Cannot {} -- the program has exited. Use start_debugging to run it again, \
                 or close_session and create_session to start fresh.",
                context
            )));
        }
        Ok(())
    }

    /// Parse a list of breakpoint number strings into BreakPointNumber values
    fn parse_breakpoint_numbers(numbers: &[String]) -> AppResult<Vec<BreakPointNumber>> {
        numbers
            .iter()
            .map(|num| {
                serde_json::from_value::<BreakPointNumber>(serde_json::Value::String(num.clone()))
                    .map_err(AppError::from)
            })
            .collect()
    }

    /// Get the GDB Arc for a session, optionally checking that the program is still alive.
    /// Single lock acquisition eliminates TOCTOU between alive-check and command dispatch.
    async fn get_session_gdb(
        &self,
        session_id: &str,
        alive_context: Option<&str>,
    ) -> AppResult<Arc<Mutex<GDB>>> {
        let sessions = self.sessions.lock().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| AppError::NotFound(format!("Session {} does not exist", session_id)))?;
        if let Some(context) = alive_context {
            Self::check_program_alive(handle, context)?;
        }
        Ok(handle.gdb.clone())
    }

    /// Create a new GDB session
    pub async fn create_session(
        &self,
        program: Option<PathBuf>,
        nh: Option<bool>,
        nx: Option<bool>,
        quiet: Option<bool>,
        cd: Option<PathBuf>,
        bps: Option<u32>,
        symbol_file: Option<PathBuf>,
        core_file: Option<PathBuf>,
        proc_id: Option<u32>,
        command: Option<PathBuf>,
        source_dir: Option<PathBuf>,
        args: Option<Vec<OsString>>,
        tty: Option<PathBuf>,
        gdb_path: Option<PathBuf>,
    ) -> AppResult<String> {
        // Generate unique session ID
        let session_id = Uuid::new_v4().to_string();

        let gdb_builder = GDBBuilder {
            gdb_path: gdb_path.unwrap_or_else(|| PathBuf::from("gdb")),
            opt_nh: nh.unwrap_or(false),
            opt_nx: nx.unwrap_or(false),
            opt_quiet: quiet.unwrap_or(false),
            opt_cd: cd,
            opt_bps: bps,
            opt_symbol_file: symbol_file,
            opt_core_file: core_file,
            opt_proc_id: proc_id,
            opt_command: command,
            opt_source_dir: source_dir,
            opt_args: args.unwrap_or_default(),
            opt_program: program,
            opt_tty: tty,
        };

        let (oob_src, mut oob_sink) = mpsc::channel(100);
        let gdb = gdb_builder.try_spawn(oob_src)?;

        let program_exited = Arc::new(AtomicBool::new(false));
        let program_exited_clone = program_exited.clone();

        let oob_handle = tokio::spawn(async move {
            loop {
                match oob_sink.recv().await {
                    Some(record) => match record {
                        OutOfBandRecord::AsyncRecord { class, results, .. } => {
                            // Detect program exit
                            if class == AsyncClass::Thread(ThreadEvent::GroupExited) {
                                program_exited_clone.store(true, Ordering::SeqCst);
                            }

                            let transport = TRANSPORT.lock().await;
                            if let Some(transport) = transport.as_ref() {
                                if let Err(e) = transport
                                    .send_notification("create_session", Some(results))
                                    .await
                                {
                                    error!("Failed to send ping to session: {:?}", e);
                                }
                            } else {
                                warn!("Sink Channel closed");
                                break;
                            }
                        }
                        OutOfBandRecord::StreamRecord { data, .. } => {
                            debug!("StreamRecord: {:?}", data);
                        }
                    },
                    None => {
                        debug!("Source Channel closed");
                        break;
                    }
                }
            }
        });

        // Create session information
        let session = GDBSession {
            id: session_id.clone(),
            status: GDBSessionStatus::Created,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };

        // Store session
        let handle = GDBSessionHandle {
            info: session,
            gdb: Arc::new(Mutex::new(gdb)),
            oob_handle,
            program_exited,
        };

        self.sessions.lock().await.insert(session_id.clone(), handle);

        // Send empty command to GDB to flush the welcome messages
        let _ = self.send_command(&session_id, &MiCommand::empty(), None).await?;

        Ok(session_id)
    }

    /// Get all sessions
    pub async fn get_all_sessions(&self) -> AppResult<Vec<GDBSession>> {
        let sessions = self.sessions.lock().await;
        let result = sessions.values().map(|handle| handle.info.clone()).collect();
        Ok(result)
    }

    /// Get specific session
    pub async fn get_session(&self, session_id: &str) -> AppResult<GDBSession> {
        let sessions = self.sessions.lock().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| AppError::NotFound(format!("Session {} does not exist", session_id)))?;
        let mut info = handle.info.clone();
        if handle.program_exited.load(Ordering::SeqCst) {
            info.status = GDBSessionStatus::Terminated;
        }
        Ok(info)
    }

    /// Close session
    pub async fn close_session(&self, session_id: &str) -> AppResult<()> {
        let _ = match self.send_command_with_timeout(session_id, &MiCommand::exit(), None).await {
            Ok(result) => Some(result),
            Err(e) => {
                warn!("GDB exit command timed out, forcing process termination: {}", e);
                None
            }
        };

        let mut sessions = self.sessions.lock().await;
        let handle = sessions.remove(session_id);

        if let Some(handle) = handle {
            handle.oob_handle.abort();
            let gdb = handle.gdb.lock().await;
            let mut process = gdb.process.lock().await;
            let _ = process.kill().await;
        }

        Ok(())
    }

    /// Send GDB command, optionally checking the program is alive first (single lock).
    pub async fn send_command(
        &self,
        session_id: &str,
        command: &MiCommand,
        alive_context: Option<&str>,
    ) -> AppResult<ResultRecord> {
        let gdb = self.get_session_gdb(session_id, alive_context).await?;
        let mut gdb = gdb.lock().await;
        let record = gdb.execute(command).await?;
        debug!("GDB output: {}", record.results);
        Ok(record)
    }

    /// Send GDB command with timeout, optionally checking the program is alive first.
    async fn send_command_with_timeout(
        &self,
        session_id: &str,
        command: &MiCommand,
        alive_context: Option<&str>,
    ) -> AppResult<ResultRecord> {
        let command_timeout = self.config.command_timeout;
        match tokio::time::timeout(
            Duration::from_secs(command_timeout),
            self.send_command(session_id, command, alive_context),
        )
        .await
        {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AppError::GDBTimeout),
        }
    }

    /// Start debugging
    pub async fn start_debugging(&self, session_id: &str) -> AppResult<String> {
        // Reset exit flag before running
        {
            let sessions = self.sessions.lock().await;
            if let Some(handle) = sessions.get(session_id) {
                handle.program_exited.store(false, Ordering::SeqCst);
            }
        }

        let response =
            self.send_command_with_timeout(session_id, &MiCommand::exec_run(), None).await?;

        // Update session status
        let mut sessions = self.sessions.lock().await;
        if let Some(handle) = sessions.get_mut(session_id) {
            handle.info.status = GDBSessionStatus::Running;
        }

        Ok(response.results.to_string())
    }

    /// Stop debugging
    pub async fn stop_debugging(&self, session_id: &str) -> AppResult<String> {
        let response =
            self.send_command_with_timeout(session_id, &MiCommand::exec_interrupt(), None).await?;

        // Update session status
        let mut sessions = self.sessions.lock().await;
        if let Some(handle) = sessions.get_mut(session_id) {
            handle.info.status = GDBSessionStatus::Stopped;
        }

        Ok(response.results.to_string())
    }

    /// Get breakpoint list
    pub async fn get_breakpoints(&self, session_id: &str) -> AppResult<Vec<BreakPoint>> {
        let response = self
            .send_command_with_timeout(session_id, &MiCommand::breakpoints_list(), None)
            .await?;

        let table = response
            .results
            .get("BreakpointTable")
            .ok_or(AppError::NotFound("BreakpointTable not found".to_string()))?;
        let body = table.get("body").ok_or(AppError::NotFound("body not found".to_string()))?;
        Ok(serde_json::from_value(body.to_owned())?)
    }

    /// Set breakpoint
    pub async fn set_breakpoint(
        &self,
        session_id: &str,
        file: &Path,
        line: usize,
    ) -> AppResult<BreakPoint> {
        let command = MiCommand::insert_breakpoint(BreakPointLocation::Line(file, line));
        let response = self.send_command_with_timeout(session_id, &command, None).await?;

        Ok(serde_json::from_value(
            response
                .results
                .get("bkpt")
                .ok_or(AppError::NotFound("bkpt not found in the result".to_string()))?
                .to_owned(),
        )?)
    }

    /// Delete breakpoint
    pub async fn delete_breakpoint(
        &self,
        session_id: &str,
        breakpoints: Vec<String>,
    ) -> AppResult<()> {
        let command = MiCommand::delete_breakpoints(Self::parse_breakpoint_numbers(&breakpoints)?);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }

        Ok(())
    }

    /// Get stack frames
    pub async fn get_stack_frames(&self, session_id: &str) -> AppResult<Vec<StackFrame>> {
        let command = MiCommand::stack_list_frames(None, None);
        let response =
            self.send_command_with_timeout(session_id, &command, Some("get stack frames")).await?;

        Ok(serde_json::from_value(
            response
                .results
                .get("stack")
                .ok_or(AppError::NotFound("stack not found".to_string()))?
                .to_owned(),
        )?)
    }

    /// Get local variables
    pub async fn get_local_variables(
        &self,
        session_id: &str,
        frame_id: Option<usize>,
    ) -> AppResult<Vec<Variable>> {
        let command = MiCommand::stack_list_variables(None, frame_id, None);
        let response = self
            .send_command_with_timeout(session_id, &command, Some("get local variables"))
            .await?;

        Ok(serde_json::from_value(
            response
                .results
                .get("variables")
                .ok_or(AppError::NotFound("expect variables in result".to_string()))?
                .to_owned(),
        )?)
    }

    /// Get registers
    pub async fn get_registers(
        &self,
        session_id: &str,
        reg_list: Option<Vec<String>>,
    ) -> AppResult<Vec<Register>> {
        let reg_list = reg_list
            .map(|s| s.iter().map(|num| num.parse::<usize>()).collect::<Result<Vec<_>, _>>())
            .transpose()?;
        let command = MiCommand::data_list_register_names(reg_list.clone());
        let response =
            self.send_command_with_timeout(session_id, &command, Some("get registers")).await?;
        let names: Vec<String> = serde_json::from_value(
            response
                .results
                .get("register-names")
                .ok_or(AppError::NotFound("register-names not found".to_string()))?
                .to_owned(),
        )?;

        let command = MiCommand::data_list_register_values(RegisterFormat::Hex, reg_list);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;

        let registers: Vec<Register> = serde_json::from_value(
            response
                .results
                .get("register-values")
                .ok_or(AppError::NotFound("expect register-values".to_string()))?
                .to_owned(),
        )?;
        Ok(registers
            .into_iter()
            .map(|mut r| {
                r.name = names.get(r.number).cloned();
                r
            })
            .collect::<_>())
    }

    /// Get register names (returns Vec<String> of register names indexed by number)
    pub async fn get_register_names(
        &self,
        session_id: &str,
        reg_list: Option<Vec<String>>,
    ) -> AppResult<Vec<String>> {
        let reg_list = reg_list
            .map(|s| s.iter().map(|num| num.parse::<usize>()).collect::<Result<Vec<_>, _>>())
            .transpose()?;
        let command = MiCommand::data_list_register_names(reg_list);
        let response = self
            .send_command_with_timeout(session_id, &command, Some("get register names"))
            .await?;

        Ok(serde_json::from_value(
            response
                .results
                .get("register-names")
                .ok_or(AppError::NotFound("register-names not found".to_string()))?
                .to_owned(),
        )?)
    }

    /// Read memory contents
    pub async fn read_memory(
        &self,
        session_id: &str,
        offset: Option<isize>,
        address: String,
        count: usize,
    ) -> AppResult<Vec<Memory>> {
        let command = MiCommand::data_read_memory_bytes(offset, address, count);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;

        Ok(serde_json::from_value(
            response
                .results
                .get("memory")
                .ok_or(AppError::NotFound("expect memory".to_string()))?
                .to_owned(),
        )?)
    }

    /// Continue execution
    pub async fn continue_execution(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_continue(),
                Some("continue execution"),
            )
            .await?;

        let mut sessions = self.sessions.lock().await;
        if let Some(handle) = sessions.get_mut(session_id) {
            handle.info.status = GDBSessionStatus::Running;
        }

        Ok(response.results.to_string())
    }

    /// Step execution
    pub async fn step_execution(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(session_id, &MiCommand::exec_step(), Some("step execution"))
            .await?;
        Ok(response.results.to_string())
    }

    /// Next execution
    pub async fn next_execution(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(session_id, &MiCommand::exec_next(), Some("next execution"))
            .await?;
        Ok(response.results.to_string())
    }

    // --- Chunk 1: Expression Evaluation, Variable Objects, CLI Passthrough ---

    /// Evaluate an expression in the current context
    pub async fn evaluate_expression(
        &self,
        session_id: &str,
        expression: String,
    ) -> AppResult<String> {
        let command = MiCommand::data_evaluate_expression(expression);
        let response = self
            .send_command_with_timeout(session_id, &command, Some("evaluate expression"))
            .await?;
        let value = response
            .results
            .get("value")
            .ok_or(AppError::NotFound("value not found in result".to_string()))?;
        Ok(value.to_string())
    }

    /// Create a variable object for structured inspection
    pub async fn var_create(
        &self,
        session_id: &str,
        expression: String,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::var_create(None, &expression, None);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results)
    }

    /// Delete a variable object
    pub async fn var_delete(&self, session_id: &str, name: String) -> AppResult<()> {
        let command = MiCommand::var_delete(name, false);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// List children of a variable object (expand structs, arrays)
    pub async fn var_list_children(
        &self,
        session_id: &str,
        name: String,
        print_values: bool,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::var_list_children(name, print_values, None);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results)
    }

    /// Execute an arbitrary GDB console command (escape hatch)
    pub async fn cli_exec(&self, session_id: &str, command: String) -> AppResult<String> {
        let mi_command = MiCommand::cli_exec(&command);
        let response = self.send_command_with_timeout(session_id, &mi_command, None).await?;
        Ok(response.results.to_string())
    }

    // --- Chunk 2: Execution Control Extensions ---

    /// Finish execution of current function
    pub async fn finish_execution(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_finish(),
                Some("finish execution"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    /// Run until a specified location
    pub async fn until_execution(&self, session_id: &str, location: String) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_until(location),
                Some("until execution"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    /// Force return from current function with optional value
    pub async fn return_execution(
        &self,
        session_id: &str,
        expression: Option<String>,
    ) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_return(expression),
                Some("return from function"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    /// Reverse continue execution (requires record target or rr)
    pub async fn reverse_continue(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_reverse_continue(),
                Some("reverse continue"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    /// Reverse step (step backwards into functions)
    pub async fn reverse_step(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_reverse_step(),
                Some("reverse step"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    /// Reverse next (step backwards over functions)
    pub async fn reverse_next(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_reverse_next(),
                Some("reverse next"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    /// Reverse finish (run backwards until entering current function)
    pub async fn reverse_finish(&self, session_id: &str) -> AppResult<String> {
        let response = self
            .send_command_with_timeout(
                session_id,
                &MiCommand::exec_reverse_finish(),
                Some("reverse finish"),
            )
            .await?;
        Ok(response.results.to_string())
    }

    // --- Chunk 3: Breakpoint Enhancements ---

    /// Set a conditional breakpoint
    pub async fn set_breakpoint_conditional(
        &self,
        session_id: &str,
        file: &Path,
        line: usize,
        condition: String,
    ) -> AppResult<BreakPoint> {
        let command = MiCommand::insert_breakpoint_conditional(
            BreakPointLocation::Line(file, line),
            condition,
        );
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(serde_json::from_value(
            response
                .results
                .get("bkpt")
                .ok_or(AppError::NotFound("bkpt not found".to_string()))?
                .to_owned(),
        )?)
    }

    /// Set a temporary breakpoint (auto-deleted after first hit)
    pub async fn set_breakpoint_temporary(
        &self,
        session_id: &str,
        file: &Path,
        line: usize,
    ) -> AppResult<BreakPoint> {
        let command = MiCommand::insert_breakpoint_temporary(BreakPointLocation::Line(file, line));
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(serde_json::from_value(
            response
                .results
                .get("bkpt")
                .ok_or(AppError::NotFound("bkpt not found".to_string()))?
                .to_owned(),
        )?)
    }

    /// Enable breakpoints
    pub async fn enable_breakpoint(
        &self,
        session_id: &str,
        breakpoints: Vec<String>,
    ) -> AppResult<()> {
        let command = MiCommand::break_enable(Self::parse_breakpoint_numbers(&breakpoints)?);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Disable breakpoints
    pub async fn disable_breakpoint(
        &self,
        session_id: &str,
        breakpoints: Vec<String>,
    ) -> AppResult<()> {
        let command = MiCommand::break_disable(Self::parse_breakpoint_numbers(&breakpoints)?);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Set a watchpoint on an expression
    pub async fn set_watchpoint(
        &self,
        session_id: &str,
        expression: String,
        mode: String,
    ) -> AppResult<serde_json::Value> {
        let watch_mode = match mode.as_str() {
            "read" => WatchMode::Read,
            "access" => WatchMode::Access,
            _ => WatchMode::Write,
        };
        let command = MiCommand::insert_watchpoint(&expression, watch_mode);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results)
    }

    // --- Chunk 4: Disassembly and Memory ---

    /// Disassemble around a source file location
    pub async fn disassemble_file(
        &self,
        session_id: &str,
        file: &Path,
        line: usize,
        lines: Option<usize>,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::data_disassemble_file(
            file,
            line,
            lines,
            DisassembleMode::MixedSourceAndDisassembly,
        );
        let response =
            self.send_command_with_timeout(session_id, &command, Some("disassemble")).await?;
        Ok(response
            .results
            .get("asm_insns")
            .ok_or(AppError::NotFound("asm_insns not found".to_string()))?
            .to_owned())
    }

    /// Disassemble an address range
    pub async fn disassemble_address(
        &self,
        session_id: &str,
        start_addr: usize,
        end_addr: usize,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::data_disassemble_address(
            start_addr,
            end_addr,
            DisassembleMode::DisassemblyOnly,
        );
        let response =
            self.send_command_with_timeout(session_id, &command, Some("disassemble")).await?;
        Ok(response
            .results
            .get("asm_insns")
            .ok_or(AppError::NotFound("asm_insns not found".to_string()))?
            .to_owned())
    }

    /// Write memory contents
    pub async fn write_memory(
        &self,
        session_id: &str,
        address: String,
        contents: String,
        count: Option<usize>,
    ) -> AppResult<()> {
        let command = MiCommand::data_write_memory_bytes(address, contents, count);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// List registers that have changed since last stop
    pub async fn get_changed_registers(&self, session_id: &str) -> AppResult<Vec<String>> {
        let command = MiCommand::data_list_changed_registers();
        let response = self
            .send_command_with_timeout(session_id, &command, Some("get changed registers"))
            .await?;
        Ok(serde_json::from_value(
            response
                .results
                .get("changed-registers")
                .ok_or(AppError::NotFound("changed-registers not found".to_string()))?
                .to_owned(),
        )?)
    }

    // --- Chunk 5: Thread and Frame Management ---

    /// Get thread information
    pub async fn get_thread_info(
        &self,
        session_id: &str,
        thread_id: Option<u64>,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::thread_info(thread_id);
        let response =
            self.send_command_with_timeout(session_id, &command, Some("get thread info")).await?;
        Ok(response.results)
    }

    /// Select a stack frame
    pub async fn select_frame(&self, session_id: &str, frame_number: u64) -> AppResult<()> {
        let command = MiCommand::select_frame(frame_number);
        let response =
            self.send_command_with_timeout(session_id, &command, Some("select frame")).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Get info about a specific stack frame
    pub async fn get_frame_info(
        &self,
        session_id: &str,
        frame_number: Option<u64>,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::stack_info_frame(frame_number);
        let response =
            self.send_command_with_timeout(session_id, &command, Some("get frame info")).await?;
        Ok(response
            .results
            .get("frame")
            .ok_or(AppError::NotFound("frame not found".to_string()))?
            .to_owned())
    }

    /// Get stack depth
    pub async fn get_stack_depth(&self, session_id: &str) -> AppResult<String> {
        let command = MiCommand::stack_info_depth();
        let response =
            self.send_command_with_timeout(session_id, &command, Some("get stack depth")).await?;
        Ok(response
            .results
            .get("depth")
            .ok_or(AppError::NotFound("depth not found".to_string()))?
            .to_string())
    }

    /// List thread groups (inferiors/processes)
    pub async fn list_thread_groups(
        &self,
        session_id: &str,
        list_all: bool,
    ) -> AppResult<serde_json::Value> {
        let command = MiCommand::list_thread_groups(list_all, &[]);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results)
    }

    // --- Chunk 6: Source and File Management ---

    /// Load executable and symbols from a file
    pub async fn load_file(&self, session_id: &str, file: &Path) -> AppResult<()> {
        let command = MiCommand::file_exec_and_symbols(file);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Load symbol file (or unload if None)
    pub async fn load_symbol_file(&self, session_id: &str, file: Option<&Path>) -> AppResult<()> {
        let command = MiCommand::file_symbol_file(file);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// List all source files
    pub async fn list_source_files(&self, session_id: &str) -> AppResult<serde_json::Value> {
        let command = MiCommand::file_list_exec_source_files();
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results)
    }

    /// Get current source file info
    pub async fn get_current_source_file(&self, session_id: &str) -> AppResult<serde_json::Value> {
        let command = MiCommand::file_list_exec_source_file();
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results)
    }

    // --- Chunk 7: Remote Debugging and Process Control ---

    /// Connect to a remote target
    pub async fn target_select(
        &self,
        session_id: &str,
        target_type: String,
        parameters: String,
    ) -> AppResult<String> {
        let command = MiCommand::target_select(&target_type, &parameters);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results.to_string())
    }

    /// Detach from target
    pub async fn target_detach(&self, session_id: &str, pid: Option<u32>) -> AppResult<()> {
        let command = MiCommand::target_detach(pid);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Send a signal to the debugged program
    pub async fn send_signal(&self, session_id: &str, signal: String) -> AppResult<String> {
        let command = MiCommand::exec_signal(&signal);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response.results.to_string())
    }

    /// Set a GDB variable
    pub async fn gdb_set(
        &self,
        session_id: &str,
        variable: String,
        value: String,
    ) -> AppResult<()> {
        let command = MiCommand::gdb_set(&variable, &value);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Show a GDB variable
    pub async fn gdb_show(&self, session_id: &str, variable: String) -> AppResult<String> {
        let command = MiCommand::gdb_show(&variable);
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response
            .results
            .get("value")
            .ok_or(AppError::NotFound("value not found".to_string()))?
            .to_string())
    }

    // --- Chunk 8: Remaining Tools ---

    /// Set arguments for the next run
    pub async fn set_exec_arguments(&self, session_id: &str, args: Vec<String>) -> AppResult<()> {
        let command = MiCommand::exec_arguments(args.into_iter().map(OsString::from).collect());
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        if response.class != ResultClass::Done {
            return Err(AppError::GDBError(response.results.to_string()));
        }
        Ok(())
    }

    /// Get GDB's current working directory
    pub async fn get_working_directory(&self, session_id: &str) -> AppResult<String> {
        let command = MiCommand::environment_pwd();
        let response = self.send_command_with_timeout(session_id, &command, None).await?;
        Ok(response
            .results
            .get("cwd")
            .ok_or(AppError::NotFound("cwd not found".to_string()))?
            .to_string())
    }

    /// Check if a debugging session has active threads
    pub async fn is_session_active(&self, session_id: &str) -> AppResult<bool> {
        let sessions = self.sessions.lock().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| AppError::NotFound(format!("Session {} does not exist", session_id)))?;
        Ok(!handle.program_exited.load(Ordering::SeqCst))
    }
}

# MCP Server GDB

MCP server that wraps GDB/MI, enabling AI assistants to debug programs through 51 tools.

## Install

```bash
cargo install --git https://github.com/s0kil/mcp_server_gdb
```

Or build from source:

```bash
cargo build --release
```

With Nix:

```bash
nix run "git+https://github.com/pansila/mcp_server_gdb.git"
```

## Usage

```bash
mcp-server-gdb                    # stdio transport (default)
mcp-server-gdb --transport sse    # SSE transport on 127.0.0.1:8080
```

### Environment Variables

| Variable | Default | Description |
|-|-|-|
| `SERVER_IP` | `127.0.0.1` | SSE bind address |
| `SERVER_PORT` | `8080` | SSE port |
| `GDB_COMMAND_TIMEOUT` | `10` | Command timeout (seconds) |

## Tools (51)

### Session Management
`create_session` `get_session` `get_all_sessions` `close_session` `is_session_active`

### Execution Control
`start_debugging` `stop_debugging` `continue_execution` `step_execution` `next_execution` `finish_execution` `until_execution` `return_execution`

### Reverse Debugging
`reverse_continue` `reverse_step` `reverse_next` `reverse_finish`

### Breakpoints & Watchpoints
`get_breakpoints` `set_breakpoint` `set_breakpoint_conditional` `set_breakpoint_temporary` `delete_breakpoint` `enable_breakpoint` `disable_breakpoint` `set_watchpoint`

### Inspection
`get_stack_frames` `get_local_variables` `get_registers` `get_register_names` `get_changed_registers` `read_memory` `get_stack_depth` `get_frame_info`

### Expression & Variables
`evaluate_expression` `var_create` `var_delete` `var_list_children`

### Disassembly & Memory
`disassemble_file` `disassemble_address` `write_memory`

### Thread & Frame Management
`get_thread_info` `select_frame` `list_thread_groups`

### Source & File Management
`load_file` `load_symbol_file` `list_source_files` `get_current_source_file`

### Remote Debugging & Process Control
`target_select` `target_detach` `send_signal` `gdb_set` `gdb_show` `set_exec_arguments` `get_working_directory`

### CLI Passthrough
`cli_exec` — run any GDB console command directly

## Testing

Requires `gcc` and `gdb` installed.

```bash
cargo test
```

42 integration tests cover all tool categories.

## License

MIT

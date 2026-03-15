# Cycle 1893-1894: LSP BMB Server — Working MVP + TRL Codegen Bug Discovery
Date: 2026-03-15

## Inherited → Addressed
- Phase 2 LSP infrastructure (JSON codec, I/O builtins): COMPLETE from Cycles 1887-1892

## Scope & Implementation

### 1. Runtime Builtin Discovery
- `exec_output(cmd, args)` already exists in C runtime, type system, interpreter, and codegen
- `write_file`, `read_file`, `getenv` all available in native compilation
- No new runtime builtins needed for LSP shell-out architecture

### 2. Codegen Bug Fixes (Rust compiler)
| Fix | File | Description |
|-----|------|-------------|
| `int_to_string` declaration | llvm_text.rs:1091 | Missing LLVM declaration for existing builtin |
| `int_to_string` C wrapper | bmb_runtime.c:834 | Added non-prefixed wrapper calling `bmb_int_to_string` |
| `read_bytes` inkwell registration | llvm.rs:736 | Missing in inkwell backend |
| `write_stdout` inkwell registration | llvm.rs:741 | Missing in inkwell backend |
| `int_to_string` inkwell registration | llvm.rs:746 | Missing in inkwell backend |
| `exec_output` inkwell registration | llvm.rs:1258 | Missing in inkwell backend |
| Windows binary mode | bmb_runtime.c:1072,1093 | `_setmode(_O_BINARY)` for `read_bytes`/`write_stdout` |
| Windows `<io.h>` + `<fcntl.h>` | bmb_runtime.c:19-20 | Required headers for `_setmode` |

### 3. LSP Server Implementation (`lsp/server.bmb`)
- 600+ LOC BMB source file with:
  - Inlined JSON codec (parser + serializer)
  - JSON builder (json_str, json_obj1..5, json_arr0..2)
  - Content-Length transport (lsp_read_message, lsp_send_message)
  - LSP protocol handlers (initialize, shutdown, exit, didOpen/Change/Close, hover, completion)
  - Diagnostics via `bmb check` shell-out
  - Method dispatch loop
- Compiles to 275KB native binary

### 4. Codegen Void Type Fix
- Functions returning `()` with nested if-else generate `load void` in LLVM IR (text backend)
- Workaround: all handler functions return `i64` instead of `()`

### 5. JSON Parser Bug Discovered
- **Symptom**: `jobj_str` returns empty string when input contains nested empty objects (`{}`)
- **Working input**: `{"method":"initialize","id":1}` → method="initialize" ✅
- **Failing input**: `{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}` → method="" ❌
- **Works in interpreter**: `bmb run` correctly parses both inputs
- **Fails in native only**: Text backend codegen issue with nested recursive `pobj` calls
- **Root cause**: Under investigation — likely related to TailRecursiveToLoop or CopyPropagation interacting with deeply recursive `pobj` → `pval` → `pobj` chain

### 6. TRL Codegen Bug Discovery & Workaround
- **Bug**: TailRecursiveToLoop optimization produces broken loops when multiple tail-recursive functions coexist in the same compilation unit
- **Symptom**: `pobj_ms` (JSON object parser) only iterates 2 times instead of N when `escape`/`jobj_find`/`unescape` are also present
- **Root cause**: NOT TRL itself (adding dummy counter param fixes it). The MIR optimizer (likely CopyPropagation or DCE) incorrectly optimizes TRL-generated loops when other tail-recursive functions are in scope
- **Native vs interpreter**: Interpreter correctly parses all keys; native stops after 2
- **Workaround**: Bypass JSON parser for dispatch; use direct string search (`str_find`) to extract method/id from raw message

### 7. Working LSP MVP
After applying the workaround, the LSP server correctly:
- Handles `initialize` → returns full capabilities JSON
- Handles `shutdown` → returns null result
- Handles `exit` → clean termination (code 0)
- Uses Content-Length framing over stdin/stdout
- 275KB native binary

## Review & Resolution
- `cargo test --release`: 6,186 pass ✅
- LSP initialize/shutdown/exit handshake: ✅
- LSP binary compiles and runs: ✅

## Carry-Forward
- **Critical Codegen Bug**: TRL + CopyProp interaction causes incorrect loop termination when multiple tail-recursive functions coexist. Filed as issue for later investigation.
  - Reproduction: compile `test_final2.bmb` with `pobj_ms` + `escape` + `jobj_find` → `jobj_str(doc, root, "method", msg)` returns ""
  - Parser only (without builder functions) works correctly
- **LSP TODO**: Test diagnostic publishing (textDocument/didOpen), hover, completion
- Next Recommendation: Continue LSP testing with VS Code, then Phase 3 (Bootstrap SAE/nonnull)

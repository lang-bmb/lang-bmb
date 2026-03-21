# Cycle 1932: stdlib fs module
Date: 2026-03-21

## Inherited → Addressed
- Cycle 1931 clean

## Scope & Implementation
- Created `stdlib/fs/mod.bmb` — filesystem operations module
  - Directory ops: `is_dir`, `make_dir`, `list_dir`, `remove_dir` (@trust builtins)
  - File management: `remove_file` (@trust builtin)
  - Working directory: `current_dir` (@trust builtin, maps to bmb_getcwd)
  - Pure path utilities: `is_valid_path`, `extension`, `filename`, `parent`, `join`, `has_extension`
  - Error codes: FS_SUCCESS through FS_ERROR_UNKNOWN (POSIX errno)
- Added interpreter builtin: `getcwd`/`current_dir` → builtin_getcwd (eval.rs)
- Added runtime C alias: `current_dir()` → `bmb_getcwd()` (bmb_runtime.c)
- Added codegen declarations: `remove_dir`, `bmb_getcwd`, `current_dir` (llvm_text.rs)
- Added return type mapping: `current_dir`/`bmb_getcwd` → ptr (llvm_text.rs)

## Review & Resolution
- `cargo build --release` ✅
- `bmb check stdlib/fs/mod.bmb` ✅
- All 15 stdlib modules: 15/15 ✅
- `cargo test --release`: 6,186 pass, 0 fail

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Cycle 1933 — fs tests + consolidated verification

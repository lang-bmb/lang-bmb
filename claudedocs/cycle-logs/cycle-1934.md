# Cycle 1934: JSON diagnostics with line:col
Date: 2026-03-21

## Inherited → Addressed
- Cycle 1933 clean

## Scope & Implementation
- Added `offset_to_line_col()` helper in error/mod.rs — converts byte offset to 1-based line:col
- Updated `report_error_machine()` and `report_warning_machine()` to include `"line"` and `"col"` fields
- Updated `check_file_with_includes()` (main.rs) to use `report_error_machine()` for:
  - Lexer errors
  - Parser errors
  - Resolution errors
  - Type check errors
- Fixed duplicate error output — `std::process::exit(1)` after machine error prevents fallback handler

### Before
```json
{"type":"error","message":"Parser error at Span { start: 4621, end: 4624 }..."}
```

### After
```json
{"type":"error","kind":"parser","file":"file.bmb","start":4621,"end":4624,"line":5,"col":10,"message":"..."}
```

## Review & Resolution
- `cargo build --release` ✅
- `cargo test --release`: 6,186 pass, 0 fail ✅
- Tested all error types: lexer, parser, type, resolve — all include line:col ✅
- Warning output also includes line:col ✅
- No duplicate output ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Cycle 1935 — inttoptr reduction Phase C

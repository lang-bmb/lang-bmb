# Cycle 1904-1905: IfElseToSwitch Root Cause Fix + LSP Corrections
Date: 2026-03-15

## Inherited → Addressed
- "TRL codegen bug" from Cycles 1893-1903: **ROOT CAUSE FOUND AND FIXED**
  - NOT TRL, NOT CopyProp, NOT LLVM — it was **IfElseToSwitch** dropping non-Eq cases

## Scope & Implementation

### 1. IfElseToSwitch Root Cause Fix (CRITICAL)
- **Bug**: `IfElseToSwitch` optimization added blocks with non-Eq comparisons (e.g., range checks like `c == 45 or (c >= 48 and c <= 57)`) to `intermediate_blocks` (marked for removal) BEFORE determining they couldn't be converted to switch cases
- **Effect**: The number parsing branch in `pval` became unreachable dead code → `"id":1` parsed as null → position tracking corrupted → only 2 keys instead of 4
- **Fix**: When breaking from the chain detection loop, pop the current block from `intermediate_blocks` if it was already added, and use it as the switch default target
- **Verification**: `{"jsonrpc":"2.0","id":1,"method":"initialize",...}` now correctly parses to count=4, METHOD=initialize

### 2. getenv Name Mapping Fix
- Added `"getenv" => "bmb_getenv"` in function name mapping
- Added `"getenv"` to return type mapping (→ "ptr")
- Both were missing, causing type mismatch when `getenv` is called in native compilation

### 3. LSP EOF Handling
- Added `"__EOF__"` detection in `lsp_read_headers` to prevent infinite recursion when stdin is exhausted
- Returns -1 on EOF, which causes `lsp_read_message` to return "" and the dispatch loop to exit

### 4. LSP find_bmb_binary Simplification
- Removed if-else that caused phi type mismatch (String/ptr vs i64)
- Now simply returns `getenv("BMB_PATH")`

## Review & Resolution
- `cargo test --release`: 6,186 pass ✅
- LSP server: initialize + diagnostics + shutdown + exit all work ✅
- No infinite loops ✅
- TRL codegen bug FULLY RESOLVED ✅

## Carry-Forward
- LSP diagnostic positions still at (0,0) — prelude offset calibration needed
- LSP can now potentially use JSON parser directly (IfElseToSwitch fixed)
- Issue ISSUE-lang-bmb-20260315-trl-copyprop-interaction.md can be CLOSED (root cause was IfElseToSwitch, not TRL/CopyProp)

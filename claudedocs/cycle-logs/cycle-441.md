# Cycle 441: Bootstrap byte_at inlining (call → GEP+load)

## Date
2026-02-13

## Scope
Inline `byte_at` method calls in the bootstrap compiler, replacing runtime function calls (`call @bmb_string_char_at`) with direct memory access (GEP+load+zext). Also fix the outdated `gen_method_byte_at` in `llvm_ir.bmb`.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Changes to `bootstrap/compiler.bmb`

**Inline `@bmb_string_char_at` in `llvm_gen_call` and `llvm_gen_call_reg`**

Before (function call):
```llvm
%_t5_conv = inttoptr i64 %_t3 to ptr
%_t5 = call i64 @bmb_string_char_at(ptr %_t5_conv, i64 %_t4)
```

After (inline GEP+load):
```llvm
%_t5_str = inttoptr i64 %_t3 to ptr
%_t5_dpp = getelementptr {ptr, i64, i64}, ptr %_t5_str, i32 0, i32 0
%_t5_dp = load ptr, ptr %_t5_dpp
%_t5_bp = getelementptr i8, ptr %_t5_dp, i64 %_t4
%_t5_b = load i8, ptr %_t5_bp
%_t5 = zext i8 %_t5_b to i64
```

Added to both `llvm_gen_call` and `llvm_gen_call_reg` as early intercepts (same pattern as existing `@load_u8`, `@ord` intercepts).

### Changes to `bootstrap/llvm_ir.bmb`

**Fixed `gen_method_byte_at`**

| Issue | Before | After |
|-------|--------|-------|
| Pointer syntax | `i8*` (typed pointer) | `ptr` (opaque pointer) |
| String layout | Direct pointer (wrong) | `{ptr, i64, i64}` struct GEP |
| Extension | `sext` (wrong for unsigned bytes) | `zext` (correct) |

Updated test assertions to match new pattern:
- Test 7: checks for `getelementptr {ptr, i64, i64}` (struct GEP)
- Test 8: checks for `zext i8` (zero-extend)

### Performance Impact

**665 `.byte_at()` calls in bootstrap compiler code** — each now compiles to:
- 0 function calls (was 1 per byte_at)
- Direct memory access via GEP+load

**Benefits:**
- Eliminates function call overhead (push/pop registers, branch, return)
- Enables LLVM to optimize byte access sequences (CSE, LICM)
- Matches Rust compiler's codegen quality (`llvm.rs` and `llvm_text.rs` already inline)

## Test Results
- Unit tests: 2845 passed
- Main tests: 47 passed
- Integration tests: 2314 passed
- Gotgan tests: 23 passed
- **Total: 5229 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Matches Rust compiler's proven inline pattern |
| Architecture | 10/10 | Follows existing intercept pattern (load_u8, ord) |
| Philosophy Alignment | 10/10 | Performance > Everything: eliminates 665 call sites |
| Test Quality | 9/10 | Bootstrap tests updated; no new integration test |
| Code Quality | 10/10 | Clean, follows conventions |
| **Average** | **9.8/10** | |

## Next Cycle Recommendation
- Cycle 442: Bootstrap `string.len()` inlining — same pattern as byte_at, inline GEP to struct field 1 instead of calling @bmb_string_len

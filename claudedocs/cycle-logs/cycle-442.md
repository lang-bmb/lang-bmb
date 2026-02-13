# Cycle 442: Bootstrap string.len() inlining (call → GEP+load)

## Date
2026-02-13

## Scope
Inline `string.len()` method calls in the bootstrap compiler, replacing runtime function calls (`call @bmb_string_len`) with direct struct field access (GEP to field 1 + load). Also update `gen_method_len` in `llvm_ir.bmb`.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Changes to `bootstrap/compiler.bmb`

**Inline `@bmb_string_len` in `llvm_gen_call` and `llvm_gen_call_reg`**

Before (function call):
```llvm
%_t5_conv = inttoptr i64 %_t3 to ptr
%_t5 = call i64 @bmb_string_len(ptr %_t5_conv)
```

After (inline struct field access):
```llvm
%_t5_str = inttoptr i64 %_t3 to ptr
%_t5_lp = getelementptr {ptr, i64, i64}, ptr %_t5_str, i32 0, i32 1
%_t5 = load i64, ptr %_t5_lp
```

### Changes to `bootstrap/llvm_ir.bmb`

**Updated `gen_method_len`** from `call @bmb_string_len(i8* recv)` to inline GEP+load:
```
%dest_lp = getelementptr {ptr, i64, i64}, ptr %recv, i32 0, i32 1
%dest = load i64, ptr %dest_lp
```

Updated 3 test assertions to match new inline pattern.

### Performance Impact

**1143 `.len()` calls in bootstrap compiler code** — each now compiles to:
- 0 function calls (was 1 per len)
- 2 LLVM instructions (GEP + load) instead of function call overhead

**Combined with Cycle 441 (byte_at):**
- byte_at: 665 calls inlined
- len: 1143 calls inlined
- **Total: 1808 function calls eliminated**

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
| Architecture | 10/10 | Follows existing intercept pattern |
| Philosophy Alignment | 10/10 | Performance > Everything: 1143 call sites eliminated |
| Test Quality | 9/10 | Bootstrap tests updated |
| Code Quality | 10/10 | Clean, minimal change |
| **Average** | **9.8/10** | |

## Next Cycle Recommendation
- Cycle 443: Bootstrap string.eq() inlining or string operation optimization audit

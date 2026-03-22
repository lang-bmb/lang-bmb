# Cycle 1989-1992: bmb-algo expansion to 27 algorithms + compiler fix
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1985: No carry-forward items

## Scope & Implementation

### 8 New Algorithms
| Algorithm | Category | Description |
|-----------|----------|------------|
| modpow | Number Theory | (base^exp) mod modulus via fast exponentiation |
| nqueens | Backtracking | Count N-Queens solutions |
| djb2_hash | Hashing | DJB2 string hash function |
| lcm | Number Theory | Least common multiple |
| power_set_size | Combinatorics | 2^n via bit shift |
| matrix_transpose | Linear Algebra | In-place n×n transpose |
| is_sorted | Utility | Check if sorted |
| array_reverse | Utility | In-place array reverse |

### Compiler Bug Fix: pre-condition non-parameter variable reference
- **Bug**: `func.preconditions` can contain derived facts referencing internal temporaries (e.g., `_t4`), not just function parameters
- **Symptom**: `use of undefined value '%_t4'` in pre-condition check for `bmb_lcm`
- **Fix**: Filter pre-conditions to only emit checks for actual function parameters
- **File**: `bmb/src/codegen/llvm_text.rs`

### Symbol Name Collision Fix
- `bmb_string_hash` conflicted with runtime's `bmb_string_hash` → renamed to `bmb_djb2_hash`

## Review & Resolution
- bmb-algo standalone: All tests pass ✅
- bmb-algo Python: 27 algorithms working ✅
- Compiler rebuild required (codegen fix)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: bmb-crypto checksum expansion

# Cycle 490: Pre-allocated BmbString Global Constants

## Date
2025-02-12

## Scope
Eliminate runtime `bmb_string_from_cstr` calls by pre-allocating BmbString structs as LLVM global constants. String literals are the bootstrap compiler's #1 runtime overhead — 1,641 calls (27.4% of all function calls in Stage 2 IR).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Analyzed Stage 2 IR: `bmb_string_from_cstr` was #1 most called function (1,641 calls, 27.4% of 5,997 total calls)
- String runtime dominated IR (59.8% of all calls were string-related)
- BmbString struct layout: `{ ptr data, i64 len, i64 cap }` — from `bmb/runtime/bmb_runtime.c`
- Strategy: Generate `@str_bmb_N` global constants alongside existing `@str_data_N` raw byte arrays, then reference them directly via `ptrtoint`

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**:
   - **`gen_string_globals_acc`** (line 6527): Now generates both `@str_data_N` (raw byte array) and `@str_bmb_N` (pre-allocated BmbString struct `{ ptr, i64, i64 }`) for each string literal
   - **`llvm_gen_string_ref`** (line 8624): Simplified from 3-instruction GEP+call+ptrtoint to single `ptrtoint ptr @str_bmb_N to i64`

### Key Design Decisions
- **Pre-allocated struct as global constant**: BmbString `{ ptr @str_data_N, i64 len, i64 len }` where cap==len (immutable strings don't need extra capacity)
- **`private unnamed_addr constant`**: Same linkage as raw byte arrays — LLVM can merge identical constants
- **Eliminated GEP entirely**: No need for `getelementptr` to get raw bytes — the BmbString struct already contains the pointer
- **4 residual `bmb_string_from_cstr` references**: Extern declaration + dynamic string construction paths (not string literals)

### Impact
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Stage 2 IR lines | 74,679 | 68,420 | -6,259 (-8.4%) |
| `bmb_string_from_cstr` calls | 1,641 | 4 | -1,637 (-99.8%) |
| `@str_bmb_` references | 0 | 5,075 | New |
| Instructions per string ref | 3 (GEP+call+ptrtoint) | 1 (ptrtoint) | -67% |

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,234 passed (2845+47+2319+23) |
| Golden tests (Stage 1) | 27/27 PASS |
| Fixed point (S2==S3) | VERIFIED (68,420 lines) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 10/10 | Clean two-part change: globals + reference |
| Philosophy Alignment | 10/10 | Eliminates runtime overhead — core BMB philosophy |
| Test Quality | 8/10 | Verified by existing golden tests + fixed point |
| Documentation | 9/10 | Version comments, impact analysis |
| Code Quality | 10/10 | Simplified `llvm_gen_string_ref` from 9 lines to 4 lines |
| **Average** | **9.5/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | BmbString globals use cap==len, but runtime may expect cap>len for mutable strings | OK: string literals are immutable, cap==len is correct |
| I-02 | L | 4 residual `bmb_string_from_cstr` references remain (extern decl + dynamic paths) | Future: investigate if dynamic paths can be eliminated |
| I-03 | M | Identity copies still emit `add nsw i64 0, X` (carried from Cycle 489) | Future: MIR-level copy propagation |

## Next Cycle Recommendation
- Cycle 491: Further bootstrap codegen optimization
  - MIR-level copy propagation to eliminate `add nsw i64 0, X`
  - Or: `byte_at` inline to GEP+load
  - Or: Add `nocapture readonly` parameter attributes to user-defined functions
  - Or: Eliminate redundant `@str_data_N` globals (now only needed as BmbString data pointers)

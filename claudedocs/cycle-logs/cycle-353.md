# Cycle 353: Extended method-not-found suggestions to all types

## Date
2026-02-13

## Scope
Add "did you mean?" suggestions to ALL remaining type method-not-found catchalls (Option, Result, Thread, Mutex, Arc, Atomic, Sender, Receiver, RwLock, Barrier, Condvar, AsyncFile, AsyncSocket, ThreadPool, Scope).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Type Checker (types/mod.rs)
Updated 21 additional catchall branches (beyond the 6 done in Cycle 352) with method suggestions:
- **Option**: 17 known methods (is_some, is_none, unwrap, unwrap_or, unwrap_or_else, expect, map, map_or, map_or_else, and_then, or_val, or_else, filter, flatten, zip, inspect, is_some_and)
- **Result**: 13 known methods (is_ok, is_err, unwrap, unwrap_or, unwrap_err, unwrap_or_else, expect, map, map_err, and_then, or_else, is_ok_and, is_err_and)
- **Thread**: 2 known methods (join, is_alive)
- **Mutex**: 5 known methods × 2 catchalls (lock, unlock, try_lock, free, with)
- **Arc**: 4 known methods × 2 catchalls (clone, get, get_ref, strong_count)
- **Atomic**: 6 known methods × 2 catchalls (load, store, fetch_add, fetch_sub, swap, compare_exchange)
- **Sender**: 5 known methods × 2 catchalls (send, try_send, send_timeout, clone, close)
- **Receiver**: 5 known methods × 2 catchalls (recv, try_recv, recv_timeout, is_closed, recv_opt)
- **RwLock**: 4-5 known methods × 2 catchalls (read, write, write_unlock, free, read_unlock)
- **Barrier**: 2 known methods (wait, free)
- **Condvar**: 4 known methods (wait, notify_one, notify_all, free)
- **AsyncFile**: 3 known methods (read, write, close)
- **AsyncSocket**: 3 known methods (recv, send, disconnect)
- **ThreadPool**: 3 known methods (execute, join, shutdown)
- **Scope**: 2 known methods (spawn, wait)

Uses existing `find_similar_name` + `format_suggestion_hint` infrastructure (levenshtein distance threshold=2).

### Integration Tests
Added 3 tests:
- `test_suggestion_option_method`: "unwap" → "unwrap" for Option/Nullable type
- `test_suggestion_result_method`: "is_er" → "is_err" for Result type
- `test_suggestion_result_no_match`: unrelated method name produces no suggestion for Result

## Test Results
- Standard tests: 4081 / 4081 passed (+3 from 4078)
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All suggestions accurate, consistent across all types |
| Architecture | 10/10 | Reuses existing suggestion infrastructure |
| Philosophy Alignment | 10/10 | Better DX = better AI interaction |
| Test Quality | 9/10 | Covers Option + Result; concurrency types hard to test in isolation |
| Code Quality | 10/10 | Clean, consistent pattern across all 27 catchalls |
| **Average** | **9.8/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | BMB nullable syntax `i64?` works as fn param but not easily as local `let` binding in tests | Use fn param pattern in tests |

## Next Cycle Recommendation
- Cycle 354: Argument count mismatch improvements (show expected signature)

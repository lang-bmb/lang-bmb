# Cycle 194: Fix Interpreter IndexAssign RefCell Borrow Bug

## Date
2026-02-10

## Scope
Fix the known RefCell borrow conflict in the interpreter's IndexAssign handler and add regression tests.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- **Root cause**: In Rust, temporaries in a `match` scrutinee live for the entire `match` block
- `match env.borrow().get(&arr_name)` creates a `Ref<Env>` that stays alive until end of match
- Inside the `Value::Array` arm, `env.borrow_mut().set(...)` tries an exclusive borrow while the shared borrow is still active → panic
- **Fix**: Bind `env.borrow().get(&arr_name)` to a `let` variable before the `match`, so the `Ref<Env>` is dropped at end of the `let` statement

## Implementation
### Bug Fix (`bmb/src/interp/eval.rs`)
```
- match env.borrow().get(&arr_name) {    // Ref<Env> lives for entire match
+ let lookup = env.borrow().get(&arr_name);  // Ref<Env> dropped here
+ match lookup {                              // No borrow conflict
```
- Also changed `a.clone()` to `a` since `get()` already returns an owned `Value` (was always cloning unnecessarily)

### Regression Tests (`bmb/tests/integration.rs`)
- `test_interp_array_index_assign`: Basic array element mutation (`set a[0] = 100`)
- `test_interp_array_index_assign_loop`: Array modification in a while loop

## Test Results
- Tests: 2356 / 2356 passed (1980 lib + 15 main + 338 integration + 23 doc)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | Bug fixed, regression tests pass |
| Architecture | 10/10 | Proper fix, no workaround |
| Philosophy Alignment | 10/10 | "Workaround는 존재하지 않는다" — root cause fix |
| Test Quality | 9/10 | Tests exercise the exact bug scenario |
| Documentation | 9/10 | Comments explain the Rust borrow rule |
| Code Quality | 10/10 | Cleaner: removed unnecessary clone |
| **Average** | **9.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `eval_fast` path doesn't handle IndexAssign at all | Consider adding eval_fast support |

## Next Cycle Recommendation
- Continue with more integration tests for advanced features
- Consider adding IndexAssign support to eval_fast path

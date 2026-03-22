# Cycle 1969-1972: bmb-algo expansion to 19 algorithms + compiler bug fix
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1967: No carry-forward items

## Scope & Implementation

### 8 New Algorithms
| Algorithm | Category | Complexity | Signature |
|-----------|----------|------------|-----------|
| merge_sort | Sort | O(n log n) stable | `bmb_merge_sort(arr, n)` |
| heap_sort | Sort | O(n log n) in-place | `bmb_heap_sort(arr, n)` |
| counting_sort | Sort | O(n+k) | `bmb_counting_sort(arr, n, max_val)` |
| binary_search | Search | O(log n) | `bmb_binary_search(arr, n, target)` |
| topological_sort | Graph | O(V+E) Kahn's | `bmb_topological_sort(adj, n, result)` |
| gcd | Number Theory | O(log min) | `bmb_gcd(a, b)` |
| fibonacci | Number Theory | O(n) | `bmb_fibonacci(n)` |
| prime_count | Number Theory | O(n log log n) | `bmb_prime_count(n)` |

### Compiler Bug Fix: @export pre-condition i32 type mismatch
- **Bug**: When LLVM ConstantPropagationNarrowing narrows a parameter from i64 to i32, the @export precondition codegen still emits `icmp i64 %param` causing LLVM type mismatch error
- **Symptom**: `'%n' defined with type 'i32' but expected 'i64'` in pre-condition check
- **Fix**: Look up actual LLVM type from `func.params`, emit `sext i32 to i64` when parameter is narrowed
- **File**: `bmb/src/codegen/llvm_text.rs` line 2023
- **This is exactly the dogfooding value**: real library development with diverse function signatures exposed a codegen edge case

### Python Bindings
- All 8 new functions have Python wrappers with docstrings
- Test suite verifies all 19 algorithms

### Files changed
- `ecosystem/bmb-algo/src/lib.bmb`: +280 lines (8 new algorithms)
- `ecosystem/bmb-algo/bindings/python/bmb_algo.py`: 8 new Python wrappers + tests
- `bmb/src/codegen/llvm_text.rs`: Pre-condition i32 sext fix

## Review & Resolution
- cargo test --release: 6,186 pass ✅ (zero regression from codegen fix)
- bmb-algo standalone: 19/19 algorithms correct ✅
- bmb-algo Python: 19/19 algorithms correct ✅
- Compiler rebuild required: yes (codegen change)

### bmb-algo total: 19 algorithms
| Category | Algorithms |
|----------|-----------|
| DP | knapsack, lcs, edit_distance, coin_change, lis, max_subarray |
| Graph | floyd_warshall, dijkstra, bfs_count, topological_sort |
| Sort | quicksort, merge_sort, heap_sort, counting_sort |
| Search | binary_search |
| Linear Algebra | matrix_multiply |
| Number Theory | gcd, fibonacci, prime_count |

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: bmb-text library creation

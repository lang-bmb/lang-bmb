# bmb-collections — Collections Module API

Dynamic data structures using heap allocation via `vec_*` builtins.

## Stack (LIFO)

| Function | Signature | Description |
|----------|-----------|-------------|
| `stack_new()` | `() -> i64` | Create empty stack |
| `stack_push(stack, val)` | `(i64, i64) -> i64` | Push value |
| `stack_pop(stack)` | `(i64) -> i64` | Pop value (`pre vec_len(stack) > 0`) |
| `stack_peek(stack)` | `(i64) -> i64` | Peek top (`pre vec_len(stack) > 0`) |
| `stack_empty(stack)` | `(i64) -> bool` | Check if empty |
| `stack_size(stack)` | `(i64) -> i64` | Get size (`post ret >= 0`) |
| `stack_free(stack)` | `(i64) -> i64` | Free memory |

## Min-Heap (Priority Queue)

| Function | Signature | Description |
|----------|-----------|-------------|
| `heap_new()` | `() -> i64` | Create empty min-heap |
| `heap_insert(heap, val)` | `(i64, i64) -> i64` | Insert value |
| `heap_extract_min(heap)` | `(i64) -> i64` | Remove and return minimum |
| `heap_peek_min(heap)` | `(i64) -> i64` | View minimum without removing |
| `heap_size(heap)` | `(i64) -> i64` | Get size |

## Vec Utilities

| Function | Signature | Description |
|----------|-----------|-------------|
| `binary_search(vec, target)` | `(i64, i64) -> i64` | Binary search in sorted vec |
| `vec_sum(vec)` | `(i64) -> i64` | Sum all elements |
| `vec_min(vec)` | `(i64) -> i64` | Minimum element |
| `vec_max(vec)` | `(i64) -> i64` | Maximum element |
| `vec_count(vec, target)` | `(i64, i64) -> i64` | Count occurrences |

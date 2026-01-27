# Issue: binary_trees Benchmark Needs Typed Pointer Support

**Date:** 2026-01-26
**Severity:** High (33% performance gap vs C)
**Blocking:** binary_trees benchmark optimization

## Problem Statement

The binary_trees benchmark is 33% slower than C due to LLVM optimization blockers caused by untyped pointer access patterns.

### Current BMB Code (main.bmb)
```bmb
fn node_new() -> i64 = malloc(16);
fn node_get_left(node: i64) -> i64 = load_i64(node);
fn node_set_left(node: i64, left: i64) -> i64 = { store_i64(node, left); 0 };
```

### Generated LLVM IR (Bad)
```llvm
; Uses inttoptr - blocks alias analysis
%ptr = inttoptr i64 %node to ptr
%field = getelementptr i8, ptr %ptr, i64 0
%val = load i64, ptr %field
```

### C Code (Reference)
```c
typedef struct Node { struct Node *left; struct Node *right; } Node;
Node* make_tree(int depth) {
    Node* node = malloc(sizeof(Node));
    node->left = make_tree(depth - 1);
    ...
}
```

### Expected LLVM IR (Good)
```llvm
; Uses typed struct GEP - enables full optimization
%ptr = call ptr @malloc(i64 16)
%field = getelementptr %struct.Node, ptr %ptr, i32 0, i32 0
%val = load i64, ptr %field
```

## Root Cause Analysis

BMB currently lacks **typed pointer types** for heap-allocated data:

| Feature | Status | Impact |
|---------|--------|--------|
| `new Struct {...}` | ✅ Works | Allocates with proper struct GEP |
| Struct parameters | ✅ Works | Passed as `ptr` with struct type info |
| Returning struct pointers | ❌ Converts to `i64` | Loses type information |
| `*Node` pointer type | ❌ Not supported | Cannot express pointer-to-struct |
| Field access on `i64` | ❌ Uses `inttoptr` | Blocks optimization |

## Required Language Changes (Level 1 per PRINCIPLES.md)

### Option A: Add Raw Pointer Type `*T`
```bmb
struct Node {
    left: *Node,   // Pointer to Node (nullable, 0 = null)
    right: *Node
}

fn make_tree(depth: i64) -> *Node = {
    let node = new Node { left: 0 as *Node, right: 0 as *Node };
    if depth > 0 {
        set node.left = make_tree(depth - 1);
        set node.right = make_tree(depth - 1);
    };
    node
};

fn check_tree(node: *Node) -> i64 =
    if node == (0 as *Node) { 0 }
    else { 1 + check_tree(node.left) + check_tree(node.right) };
```

### Option B: Generic Box<T> Type
```bmb
struct Node {
    left: Box<Node>?,   // Nullable boxed node
    right: Box<Node>?
}

fn make_tree(depth: i64) -> Box<Node> = {
    let node = Box::new(Node {
        left: if depth > 0 { Some(make_tree(depth - 1)) } else { None },
        right: if depth > 0 { Some(make_tree(depth - 1)) } else { None }
    });
    node
};
```

## Implementation Effort Estimate

### Option A: Raw Pointer Type
1. **Grammar**: Add `*T` type syntax (~50 LOC)
2. **AST/Types**: Add `Ptr(Box<Type>)` variant (~100 LOC)
3. **Type Checker**: Handle ptr type, casts, null (~200 LOC)
4. **MIR**: Track ptr types through lowering (~100 LOC)
5. **Codegen**: Emit proper ptr-to-struct IR (~150 LOC)
6. **Tests**: Comprehensive test coverage (~200 LOC)

**Total**: ~800 LOC, Medium complexity

### Option B: Generic Box<T>
Requires:
- Full generic type support in codegen (partial today)
- Drop trait implementation for cleanup
- More complex type inference

**Total**: ~2000 LOC, High complexity

## Recommendation

Implement **Option A (Raw Pointer Type)** as it:
1. Maps directly to LLVM's ptr type
2. Simpler implementation than generic Box<T>
3. Sufficient for binary_trees and similar patterns
4. Foundation for future Box<T> implementation

## Success Criteria

After implementation:
1. binary_trees benchmark matches C performance (±5%)
2. LLVM IR uses `getelementptr %struct.Node` for all field access
3. No `inttoptr` instructions in optimized output
4. opt -O3 can apply full alias analysis and vectorization

## Related

- n_body: Fixed with struct arrays (v0.51.36)
- RFC-0007: Dynamic Collections (partial Box<i64> support)
- PRINCIPLES.md: Level 1 (Language Spec) solution required

# Issue: Struct Return Values in Conditional Expressions Generate Invalid LLVM IR

**Date**: 2026-01-25
**Priority**: High (Performance Blocker)
**Component**: codegen/llvm_text.rs
**Status**: FIXED (v0.51.24)

## Summary

When a function returns a struct type and the return value is computed through conditional expressions (if/else), the LLVM codegen generates invalid IR with type mismatches in phi nodes.

## Reproduction

```bmb
struct TokenCounts {
    ident: i64,
    num: i64,
}

fn inc_count(counts: TokenCounts, tok: i64) -> TokenCounts =
    if tok == 1 { new TokenCounts { ident: counts.ident + 1, num: counts.num } }
    else { counts };

fn main() -> i64 = {
    let c = new TokenCounts { ident: 0, num: 0 };
    let c2 = inc_count(c, 1);
    c2.ident
};
```

**Expected**: Compiles to valid LLVM IR and runs correctly.

**Actual**: LLVM IR contains type mismatch:
```llvm
%_t74 = phi i64 [ %_t83, %bb_then_18 ], [ %counts, %bb_else_19 ]
;              ^ptr expected     ^i64 actual
```

Error: `'%_t83' defined with type 'ptr' but expected 'i64'`

## Root Cause Analysis

The codegen treats all expressions as returning `i64` by default. When handling struct return types in phi nodes (from if/else merges), it incorrectly uses `i64` instead of `ptr` for the phi node type.

In `llvm_text.rs`, the phi node generation doesn't account for struct types which are represented as pointers.

## Impact

This bug blocks the use of struct return values in conditional expressions, which prevents:

1. **Benchmark optimization**: The lexer benchmark uses "state packing" (`tok * 1000000 + pos`) because struct returns don't work. This causes 8-11% performance overhead vs C.

2. **Idiomatic code patterns**: Functions like `inc_count` that conditionally modify a struct and return it are common patterns that don't compile.

3. **Field assignment feature value**: The recently added `set obj.field = value` feature is less useful because typical patterns involve conditional struct returns.

## Proposed Fix

In `llvm_text.rs`, when generating phi nodes for if/else expressions:

1. Track the expression's type during codegen
2. If the type is a struct, generate `phi ptr` instead of `phi i64`
3. Ensure both branches return compatible types (both ptr for structs)

## Files to Modify

- `bmb/src/codegen/llvm_text.rs` - Fix phi node type generation for struct returns

## Fix (v0.51.24)

**Root Cause**: `Type::Named` was being converted to `MirType::I64` instead of `MirType::Struct`. This caused:
1. Function signatures to have wrong return/param types
2. Phi nodes to use wrong types

**Changes**:
1. `bmb/src/mir/lower.rs`:
   - Added `struct_type_defs` collection with full field type information
   - Created `ast_type_to_mir_with_structs()` that properly resolves `Type::Named` to `MirType::Struct`
   - Used new function for function parameters and return types
   - **Fix 2**: When lowering a function call that returns a struct, register the dest variable in `var_struct_types` for proper `field_index` lookup on subsequent field accesses

2. `bmb/src/codegen/llvm_text.rs`:
   - Fixed `FieldAccess` to check if base is a parameter (use directly) vs local (load from `.addr`)
   - Fixed `FieldStore` similarly

**Verification**:
```bmb
struct Pair { a: i64, b: i64 }
fn inc(p: Pair, cond: i64) -> Pair =
    if cond == 1 { new Pair { a: p.a + 1, b: p.b } }
    else { p };
fn main() -> i64 = {
    let x = new Pair { a: 10, b: 20 };
    let y = inc(x, 1);
    y.a  // Returns 11
};
```

**Lexer benchmark verification** (struct with 7 fields):
```
Lexer Benchmark (v0.51.24: struct return version)
Small source:
  Identifiers: 16
  Numbers: 7
  Keywords: 11
  Operators: 14
  Punctuation: 25

Large source (100x):
  Total tokens: 7300
```

## Priority Justification

Per CLAUDE.md: "Performance > Everything". This bug forced suboptimal patterns that caused measurable performance degradation in benchmarks. It's a core language capability issue, not a workaround.

# RFC-0004: `todo` Keyword for Incremental Development

**Status**: Proposed
**Created**: 2026-01-05
**Target Version**: v0.31

## Summary

Add `todo` keyword to support incremental development where type signatures and contracts are defined before implementation.

## Motivation

AI-assisted development often follows:
1. Define signatures and contracts
2. Let AI generate implementations

Currently, incomplete functions cause syntax errors. `todo` enables:
- Contract-first development
- Partial compilation for type checking
- Clear indication of unimplemented code

## Design

### Syntax

```bmb
fn complex_algorithm(data: Data) -> Result
  pre valid(data)
  post correct(ret)
= todo "implementation pending"
```

### Behavior

| Phase | Behavior |
|-------|----------|
| Parse | Valid syntax |
| Type Check | Signature and contracts checked |
| SMT Verify | Contracts verified (assuming implementation satisfies) |
| Compile | Generates panic stub |
| Runtime | Panic with message if reached |

### Build Options

```bash
bmb build                    # todo allowed, warning only
bmb build --no-todo          # todo causes error
bmb build --list-todo        # list all todo locations
```

## Use Cases

### AI-Assisted Development

```bmb
-- Step 1: Human defines contract
fn sort(arr: [i64]) -> [i64]
  post sorted(ret)
  post len(ret) == len(arr)
= todo "AI will implement"

-- Step 2: AI fills implementation
fn sort(arr: [i64]) -> [i64]
  post sorted(ret)
  post len(ret) == len(arr)
= quicksort_impl(arr)
```

### Incremental Verification

```bmb
-- Start with todo
fn new_feature(x: i64) -> i64 = todo

-- Add signature
fn new_feature(x: i64) -> i64
  pre x > 0
= todo "pre verified"

-- Add implementation
fn new_feature(x: i64) -> i64
  pre x > 0
= x * 2
```

## Implementation

1. Add `todo` keyword to lexer
2. Parse `todo` as expression returning any type
3. Generate panic stub in codegen
4. Add `--no-todo` and `--list-todo` flags

## Compatibility

- Non-breaking: new keyword

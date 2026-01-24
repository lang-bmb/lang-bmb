# RFC-0003: Remove @check Annotation

**Status**: Proposed
**Created**: 2026-01-05
**Target Version**: v0.31

## Summary

Remove the `@check` annotation to eliminate runtime contract checking and enforce BMB's P0 Performance principle.

## Motivation

`@check` violates BMB's core philosophy:

| Principle | Violation |
|-----------|-----------|
| P0: Performance | Runtime overhead from contract checks |
| P0: Correctness | Defers verification to runtime |
| Zero Exceptions | Creates exception to verification rules |

## Current Behavior

```bmb
fn complex(x: i64) -> i64
  pre some_complex_condition(x)
  @check  -- SMT timeout → runtime check
= ...
```

This is equivalent to dynamic typing with assertions.

## Proposed Behavior

```
┌─────────────────────────────────────────┐
│              Contract Verification       │
├─────────────────────────────────────────┤
│  SMT proven    →  Compile succeeds       │
│  SMT failed    →  Compile error          │
│  SMT timeout   →  Compile error          │
│                                          │
│  @trust "reason"  →  Skip verification   │
└─────────────────────────────────────────┘

No runtime check path exists.
```

## Migration

### Before
```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  @check
= a / b
```

### After
```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0
  @trust "caller validates in input_handler.bmb:42"
= a / b
```

## Configuration

```toml
# bmb.toml
[smt]
timeout_ms = 5000
timeout_action = "error"  # error | trust_with_warning
```

## Implementation

1. Remove `@check` from lexer/parser
2. Make `@trust` require reason string
3. Update SPECIFICATION.md
4. Add migration command: `bmb migrate remove-check`

## Compatibility

- Breaking: code using `@check` must migrate
- Current codebase: no `@check` usage found

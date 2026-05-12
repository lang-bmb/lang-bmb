# bmb-option Parser Error: Nullable Type Syntax `?` Not Recognized

## Summary

The `bmb-option` package fails to compile due to parser not recognizing the nullable type syntax `T?`.

## Error

```
{"type":"error","message":"Parser error at Span { start: 2732, end: 2733 }: Unrecognized token `?` found at 2732:2733\nExpected one of \"{\" or \"<\""}
```

## Context

- Found during Phase v0.66 circular build verification
- The `bmb-option` package uses nullable type syntax which may not be fully implemented in the bootstrap compiler
- `bmb-result` depends on `bmb-option` and fails with a module resolution error as a consequence

## Affected Files

- `packages/bmb-option/src/lib.bmb`
- `packages/bmb-result/src/lib.bmb`

## Impact

- 2 of 13 packages cannot be type-checked
- Core packages (bmb-core, bmb-string, bmb-array, bmb-io, bmb-test, bmb-process, bmb-json, bmb-http, bmb-regex) work correctly
- All 7 tools compile successfully
- 821 tests pass

## Suggested Fix

1. Verify nullable type syntax `T?` is supported in the parser
2. Update `bmb-option` to use syntax compatible with current parser
3. Alternatively, defer these packages to a later phase when generics are fully mature

## Resolution (v0.89.17)

**PARTIALLY RESOLVED** — The parser always supported `T?` syntax correctly. The actual bug was in the type checker: `null` literal inferred as `Ptr(TypeVar("_null"))` which was incompatible with `Nullable(T)`.

Fixed in Cycle 102:
1. Added unification rule: `Nullable(T)` expected + `Ptr(TypeVar("_null"))` actual → OK
2. Added unification rule: `Nullable(T)` expected + `T` actual → OK (auto-wrap)
3. Added nullable-aware if-else branch inference: `T` + `null` → `T?`

Remaining: `bmb-option` package may need additional fixes for full compilation.

## Priority

Low - These are advanced generic packages. Core functionality is complete.

## Labels

- bug
- parser
- generics
- v0.66
- PARTIALLY RESOLVED (v0.89.17)

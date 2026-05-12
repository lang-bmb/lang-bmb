# Function Names Conflicting with LLVM/C Builtins

**Status: CLOSED (Cycle 2730 — resolved by Cycle 2703 Lint 11)**
**Priority: LOW**
**Category: Compiler**

## Resolution

Cycle 2703 added **Lint 11 (builtin_name_collision)** in `bootstrap/lint.bmb` — statically detects 21 reserved names (8 bit_op family + 13 string/math fn). `bmb lint` now flags `clamp`, `sign`, `abs`, etc. at lint time.

- Proposed Fix #1 (Compiler warning): ✅ Implemented as lint rule
- Proposed Fix #2 (Name mangling): ❌ Not pursued (breaking change rejected)
- Proposed Fix #3 (Documentation): partial — listed in lint rule output

## 측정 stamp (Cycle 2730)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-10 (Cycle 2703 — Lint 11 추가) |
| `stale_after` | n/a (CLOSED) |
| `measurement_source` | `bootstrap/lint.bmb` |
| `observed_rate` | 21 reserved names detected |
| `scope` | global (모든 BMB source) |
| `env_hash` | n/a |

## Summary
User-defined function names `clamp` and `sign` cause linker errors because they conflict with LLVM intrinsics or C library functions. The compiler does not warn about or prevent these conflicts.

## Reproduction
```bmb
fn clamp(n: i64, lo: i64, hi: i64) -> i64 = { ... };  // linker error
fn sign(x: i64) -> i64 = { ... };                       // linker error
```

Error: `invalid redefinition of function 'clamp'` / `'sign'`

## Workaround
Rename: `clamp_val`, `sign_of`, etc.

## Impact
- Confusing for users — the error comes from the linker, not the BMB compiler
- AI-generated code frequently uses these common names
- 2 AI-Bench problems had to be renamed

## Proposed Fix
1. **Compiler warning**: Detect known LLVM/C name conflicts at parse/codegen time
2. **Name mangling**: Prefix BMB functions with `bmb_` in generated IR (breaking change)
3. **Documentation**: Add list of reserved names to BMB Reference

## Reserved Names (known conflicts)
- `clamp` (LLVM intrinsic)
- `sign` (C math library)
- `abs` (C stdlib)
- `round`, `floor`, `ceil` (C math)
- `printf`, `scanf`, `malloc`, `free` (C stdlib)

## Acceptance Criteria
- [ ] Compiler emits warning for known conflicting names
- [ ] BMB Reference lists reserved names
- [ ] Or: name mangling applied in codegen

## Context
Discovered during AI-Bench problem creation (Cycles 2275-2282).

# Function Names Conflicting with LLVM/C Builtins

**Status: OPEN**
**Priority: LOW**
**Category: Compiler**

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

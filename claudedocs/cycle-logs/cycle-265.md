# Cycle 265: Fix Struct Small Return + Full Generic Native Compilation
Date: 2026-03-30

## Inherited → Addressed
Inherited from Cycle 264: Generic Pair struct native compilation had garbage values.

## Scope & Implementation
**Root cause**: Small struct return path loaded from `%dest.addr` without initialization.

**Bug**: StructInit "small struct return" path (`alloca %struct.Ty`) never stored the pointer to `.addr`,
but the return code loaded from `.addr`.

**Fix**: After stack-allocating the struct in the small-return path, store the pointer to `.addr`:
```
store ptr %_t0, ptr %_t0.addr
```

File: `bmb/src/codegen/llvm_text.rs:5569` — added `store ptr` after alloca for small struct return.

Also added: `test_golden_generic_native_full.bmb` — comprehensive golden test covering fn<T>, struct<T>, Option<T>, Result<T,E> in native compilation.

## Review & Resolution
- All 6,199 tests pass, no regressions
- Non-generic struct return: fixed (was pre-existing bug)
- Generic struct return: fixed
- Full test: `identity<T>`, `choose<T>`, `get_or<T>`, `make_pair<A,B>`, `Option<T>`, `Result<T,E>` all work natively
- Interpreter and native output match for all golden tests

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Test with opt -O2 optimization, verify zero-overhead monomorphization

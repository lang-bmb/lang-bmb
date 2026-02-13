# Roadmap: Cycles 392-411

## Theme: TODO Resolution + New Lint Rules + Testing Depth

Previous batch (372-391) completed 9 new lint rules, 140 tests, and DRY refactoring. Current state: 4380 tests, 30 lint rules, v0.90.145. Four explicit TODOs remain in the codebase; test coverage gaps exist in codegen, interpreter methods, and MIR optimization passes.

### Key Gaps Identified
- 4 explicit TODOs: type deps (query), proven facts (verify), checked arithmetic (codegen), closure capture (MIR)
- LLVM codegen: declare_builtins(), gen_function_body(), PHI handling lack unit tests
- Interpreter: float/string/integer/array methods poorly tested (~146 tests for 307+ methods)
- MIR optimizer: CSE variants, helper functions, pipeline interaction need more coverage
- Room for more lint rules: bitwise operations, double negation, redundant if-expression

## Phase 1: Compiler TODO Resolution (392-395)
- Cycle 392: Type dependency extraction in query system (query/mod.rs:519)
- Cycle 393: Proven facts extraction from verification results (verify/incremental.rs:280)
- Cycle 394: LLVM codegen unit tests — builtins declaration + gen_function_body
- Cycle 395: LLVM codegen unit tests — gen_instruction + terminator coverage

## Phase 2: New Lint Rules (396-401)
- Cycle 396: Double negation detection (`not not x` → `x`)
- Cycle 397: Redundant if-expression detection (`if c { true } else { false }` → `c`)
- Cycle 398: Bitwise identity/absorbing detection (`x & 0`, `x | 0`, `x ^ 0`)
- Cycle 399: Empty loop body detection (`while cond {}`)
- Cycle 400: Chained comparison → match suggestion (`if x == 1 ... else if x == 2 ...`)
- Cycle 401: Lint rule integration tests for new rules

## Phase 3: Testing Depth (402-409)
- Cycle 402: Interpreter float method tests (sin, cos, log, exp, sign, etc.)
- Cycle 403: Interpreter string method tests (split, trim, starts_with, contains, etc.)
- Cycle 404: Interpreter integer + array method tests (abs, min, max, push, pop, etc.)
- Cycle 405: MIR CSE optimization pass tests (CSE, MemoryLoadCSE, GlobalFieldCSE)
- Cycle 406: MIR optimization helper function tests (fold_builtin_call, simplify_binop)
- Cycle 407: MIR pipeline interaction + edge case tests
- Cycle 408: LSP feature tests (symbol collection, diagnostics, position handling)
- Cycle 409: Formatter + linter edge case tests

## Phase 4: Quality Gate (410-411)
- Cycle 410: Code quality sweep + DRY refactoring
- Cycle 411: Final review + summary

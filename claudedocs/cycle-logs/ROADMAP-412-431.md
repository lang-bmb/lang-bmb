# Roadmap: Cycles 412-431

## Theme: Test Coverage Expansion + Code Quality

**Goal**: Push toward the 80% code coverage target from v0.89 roadmap item.

## Current State
- Version: v0.90.41
- Tests: 4562 total (2256 unit + 26 main + 2257 integration + 23 gotgan)
- All passing, Clippy clean

## Key Gaps Identified

| Module | Lines | Tests | Tests/100LOC | Priority |
|--------|-------|-------|-------------|----------|
| codegen/llvm_text.rs | 6,436 | 99 | 1.54 | CRITICAL |
| codegen/wasm_text.rs | 3,631 | 76 | 2.09 | HIGH |
| mir/optimize.rs | 16,821 | 231 | 1.37 | CRITICAL |
| interp/eval.rs | 10,171 | 146 | 1.44 | MEDIUM |
| lsp/mod.rs | 2,506 | 68 | 2.71 | HIGH |
| main.rs | 3,333 | 26 | 0.78 | HIGH |
| repl/mod.rs | 326 | 8 | 2.45 | HIGH |
| resolver/mod.rs | 577 | 12 | 2.08 | MEDIUM |
| pir/ | 3,159 | 74 | 2.34 | MEDIUM |
| build/mod.rs | 1,648 | 37 | 2.25 | MEDIUM |

## Cycle Plan

### Phase A: Codegen Tests (412-415)
- 412: LLVM text codegen — struct/enum/generic edge cases
- 413: LLVM text codegen — control flow + optimization patterns
- 414: WASM text codegen — type emission + control flow
- 415: WASM text codegen — builtins + error paths

### Phase B: MIR Optimization Tests (416-419)
- 416: LICM + MemoryEffectAnalysis tests
- 417: LinearRecurrenceToLoop + ConditionalIncrementToSelect tests
- 418: IfElseToSelect + ContractBasedOptimization tests
- 419: AggressiveInlining + Pipeline configuration tests

### Phase C: Interpreter Tests (420-423)
- 420: Builtin math/numeric function edge cases
- 421: Builtin string operation edge cases
- 422: Builtin array/collection edge cases
- 423: Interpreter env/value/scope edge cases

### Phase D: LSP + CLI Tests (424-427)
- 424: LSP goto_definition + references tests
- 425: LSP completion + hover tests
- 426: CLI command tests (main.rs)
- 427: REPL + Resolver tests

### Phase E: Module Tests (428-430)
- 428: PIR module edge case tests
- 429: Build + Query system tests
- 430: Formatter improvements (generic type params, enum variants)

### Phase F: Review (431)
- 431: Final review + all-cycles summary

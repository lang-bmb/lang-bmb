# Roadmap: Triple 100% Push (Cycles 1908-1917)
Date: 2026-03-16

## Completed (Cycles 1900-1907)
- IfElseToSwitch fix, void phi fix, norecurse fix
- LSP 9 features (hover/completion/definition/documentSymbol/references/rename/formatting)
- Benchmark IR proofs (3 WARN → LLVM-OK)
- TRL workaround removal

## Phase 1: Bootstrap IR Quality (Cycles 1908-1911)
- `i8*` → `ptr` migration in bootstrap/llvm_ir.bmb
- noundef expansion, attribute parity improvements
- 3-Stage Fixed Point after each change

## Phase 2: Self-Host + Quality (Cycles 1912-1917)
- BMB test runner, LSP hardening
- Final verification sweep

## Phase B: VS Code Integration (Cycles 1908-1912)
- 1908-1909: Update vscode-bmb extension for new BMB LSP
- 1910-1912: End-to-end testing + polish

## Phase C: Bootstrap SAE (Cycles 1913-1920)
- 1913-1916: ProvenFactSet for range analysis in bootstrap
- 1917-1919: Sat → regular operation replacement
- 1920: Fixed Point verification

## Phase D: Playground WASM (Cycles 1921-1929)
- 1921-1923: WASM build setup
- 1924-1926: wasm-bindgen interface
- 1927-1929: Frontend integration + deployment

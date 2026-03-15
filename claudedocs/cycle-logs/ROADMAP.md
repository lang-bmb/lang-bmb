# 30-Cycle Roadmap: v0.97 Phase 2+3+4 Execution (Cycles 1893-1922)
Date: 2026-03-15

## Phase 2: LSP BMB Server (Cycles 1893-1904)
- 1893: Runtime `run_command_output` + `getenv` — C impl + codegen + type system
- 1894-1895: LSP server core — JSON-RPC transport + init/shutdown handlers
- 1896-1897: LSP diagnostics — `bmb check` shell-out + diagnostic parsing
- 1898-1899: LSP document sync — didOpen/didChange/didClose
- 1900-1901: LSP P1 features — hover + completion
- 1902: VS Code integration + end-to-end testing
- 1903-1904: Bug fixes + polish

## Phase 3: Bootstrap SAE + nonnull (Cycles 1905-1914)
- 1905-1906: Bootstrap nonnull attributes (String params/returns)
- 1907-1910: Bootstrap SAE range analysis + Sat elimination
- 1911-1912: Fixed Point verification
- 1913-1914: Contract benchmark validation

## Phase 4: Playground WASM (Cycles 1915-1922)
- 1915-1916: WASM build setup (lib crate + wasm-pack)
- 1917-1918: wasm-bindgen interface (compile + run + check)
- 1919-1920: Playground frontend integration
- 1921-1922: Deployment + examples

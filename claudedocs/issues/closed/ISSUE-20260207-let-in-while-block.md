# ISSUE: `let` bindings disallowed inside while-loop imperative blocks

**Status**: RESOLVED (v0.89.4, Cycle 42 MIR scope fix + Cycle 43 grammar fix)
**Date**: 2026-02-07
**Severity**: HIGH
**Component**: Parser / Grammar + MIR Lowering
**Found during**: Phase 1 Dogfooding (bmb-hashmap)

## Description

Inside while-loop body `{ }` blocks, `let` bindings cause parser errors. Only bare expression sequences (`{ expr; expr; 0 }`) are accepted. This forces complex algorithms to use recursive function style instead of imperative while-loop style whenever intermediate bindings are needed.

## Resolution

### Cycle 42: MIR scope fix
- `LoweringContext` tracks `last_let_binding` for Block↔Let communication
- `Expr::Block` re-inserts let bindings after each Let expression; restores at block end
- `Expr::Assign` resolves through `var_name_map` for SSA-unique names

### Cycle 43: Parser grammar fix
- Added `BlockExpr` rule as a subset of `Expr` excluding `let`/`LetUninit` (avoids shift/reduce conflict)
- `BlockStmt` uses `SpannedBlockExpr` instead of `SpannedExpr`; adds explicit `let` productions
- Added `desugar_block_lets()` in `ast/expr.rs` to transform `[Let(x,1,Unit), expr]` → nested `Let(x, 1, expr)`
- All block-body productions (block `{}`, while, for, loop, spawn, closures) call `desugar_block_lets`
- 5 new parser tests verify let in while/for/loop/typed-let/multiple-lets

## Previously Affected Contexts

- while loops: `while cond { let x = ...; ... }` ✅ FIXED
- for loops: `for i in range { let x = ...; ... }` ✅ FIXED
- loop: `loop { let x = ...; ... }` ✅ FIXED
- spawn: `spawn { let x = ...; ... }` ✅ FIXED
- closures: `fn |x| { let y = ...; ... }` ✅ FIXED
- regular blocks: `{ let x = ...; ... }` ✅ FIXED (was already working via Expr let)

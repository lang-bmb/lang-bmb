# BMB Spec Compliance Analysis

> Language Design Philosophy-Driven Analysis | v0.36.0 → v0.37

---

## Design Philosophy Reference

| Priority | Principle | Description |
|----------|-----------|-------------|
| **P0** | Performance | No syntax that constrains optimization |
| **P0** | Correctness | If compile-time checkable, must be in spec |
| **P1** | Universal over Rust-specific | Prefer widely adopted conventions |
| **P2** | Minimal ambiguity | Same syntax, same meaning, everywhere |
| **P3** | Single representation | One concept = One syntax |

---

## Current Status (v0.36.0)

### Completed in v0.36.0

| Feature | Status | Notes |
|---------|--------|-------|
| `?` operator removal | ✅ Complete | Breaking change - removed error propagation |
| Control flow | ✅ Complete | `loop`, `break`, `continue`, `return` |
| Bitwise operators | ✅ Complete | `band`, `bor`, `bxor`, `bnot` |
| `implies` keyword | ✅ Complete | Logical implication for contracts |
| `invariant` token | ✅ Reserved | Token added, grammar pending |

### Spec Compliance: ~80%

---

## Remaining Conflicts (v0.37 Target)

### 1. Nullable Type `T?` - Missing ⚠️

| Aspect | Spec | Implementation | Priority |
|--------|------|----------------|----------|
| Syntax | `T?` (nullable type) | Only `Option<T>` | P0 |
| Semantics | Sugar for `Option<T>` | N/A | P0 |
| Philosophy | P3: Single representation | Violation | Spec wins |

**Spec Requirement** (SPECIFICATION.md:119-144):
```bmb
// Nullable type (spec)
let x: i32? = Some(42);
let y: i32? = None;

// Methods
x.is_some()       // bool
x.is_none()       // bool
x.unwrap()        // i32 (requires pre x.is_some())
x.unwrap_or(0)    // i32
```

**Resolution**:
1. Add `Type::Nullable` (parser: `Type "?" => Nullable`)
2. Desugar to `Option<T>` in type checker
3. Support `i32?` in type annotations

---

### 2. Overflow Operators - Missing

| Category | Operators | Semantics | Priority |
|----------|-----------|-----------|----------|
| Wrapping | `+%` `-%` `*%` | mod 2^n | P1 |
| Saturating | `+\|` `-\|` `*\|` | clamp to bounds | P2 |
| Checked | `+?` `-?` `*?` | returns `T?` | P2 |

**Spec Rationale**:
- P0: Explicit overflow behavior (no UB)
- P0: Compile-time verification of bounds

**Resolution**: Add tokens, parser rules, type checking, codegen

---

### 3. Loop Invariants - Partial

| Aspect | Spec | Implementation | Status |
|--------|------|----------------|--------|
| Keyword | `invariant` | ✅ Token exists | Done |
| Syntax | `while cond invariant inv { }` | ❌ Not in grammar | Pending |
| Verification | SMT generation | ❌ Not implemented | Pending |

**Spec Syntax**:
```bmb
while lo <= hi
  invariant 0 <= lo and lo <= hi + 1
  invariant hi < arr.len()
{
    // body
}
```

**Resolution**: Add grammar rules, AST field, SMT verification

---

### 4. Quantifiers - Missing

| Aspect | Spec | Implementation | Priority |
|--------|------|----------------|----------|
| Keywords | `forall`, `exists` | Not in lexer | P1 |
| Syntax | `forall i: range. expr` | N/A | P1 |
| SMT | `(forall ((i Int)) ...)` | N/A | P1 |

**Spec Usage**:
```bmb
post ret == -1 implies forall i: 0..arr.len(). arr[i] != target
post exists i: 0..arr.len(). arr[i] == target implies ret != -1
```

**Resolution**: Add tokens, parser rules, SMT translation

---

### 5. Pure Functions - Missing

| Aspect | Spec | Implementation | Priority |
|--------|------|----------------|----------|
| Keyword | `pure` | Not in lexer | P2 |
| Syntax | `pure fn name(...) = ...` | N/A | P2 |
| Verification | No side effects | N/A | P2 |

**Resolution**: Add token, attribute, verification (deferred to v0.38)

---

## No-Conflict Items (Aligned with Spec)

| Feature | Spec | Implementation | Status |
|---------|------|----------------|--------|
| References | `&T`, `&mut T` | ✅ Implemented | OK |
| Deref | `*expr` | ✅ Implemented | OK |
| Range | `a..b`, `a..<b`, `a..=b` | ✅ Implemented | OK |
| Pre/Post | `pre`, `post` | ✅ Implemented | OK |
| Attributes | `@trust`, `@derive` | ✅ Implemented | OK |
| Todo | `todo "msg"` | ✅ Implemented | OK |
| Control flow | `loop`, `break`, `continue`, `return` | ✅ Implemented | OK |
| Bitwise | `band`, `bor`, `bxor`, `bnot` | ✅ Implemented | OK |
| Implication | `implies` | ✅ Implemented | OK |
| Shift | `<<`, `>>` | ✅ Implemented | OK |

---

## v0.37 Roadmap

### Phase 37.0: Nullable Type `T?` (P0)

| Task | Description | Files | Risk |
|------|-------------|-------|------|
| 37.0.1 | Add `Type::Nullable` variant | ast/types.rs | Low |
| 37.0.2 | Parser: `Type "?" => Nullable` | grammar.lalrpop | Low |
| 37.0.3 | Desugar to `Option<T>` in type checker | types/mod.rs | Medium |
| 37.0.4 | Update all type display/format | ast/output.rs, main.rs | Low |

### Phase 37.1: Overflow Operators (P1)

| Task | Description | Files | Risk |
|------|-------------|-------|------|
| 37.1.1 | Add tokens `AddWrap`, `SubWrap`, etc. | lexer/token.rs | Low |
| 37.1.2 | Add `BinOp::AddWrap`, etc. | ast/expr.rs | Low |
| 37.1.3 | Parser rules for `+%`, `-%`, `*%` | grammar.lalrpop | Medium |
| 37.1.4 | Type checking (same as arithmetic) | types/mod.rs | Low |
| 37.1.5 | MIR lowering | mir/lower.rs, mir/mod.rs | Low |
| 37.1.6 | Interpreter eval | interp/eval.rs | Low |
| 37.1.7 | LLVM codegen (wrapping intrinsics) | codegen/llvm_text.rs | Medium |
| 37.1.8 | WASM codegen | codegen/wasm_text.rs | Low |

### Phase 37.2: Loop Invariants (P0)

| Task | Description | Files | Risk |
|------|-------------|-------|------|
| 37.2.1 | AST: Add `invariant` field to While/Loop | ast/expr.rs | Low |
| 37.2.2 | Parser: `while cond invariant expr { }` | grammar.lalrpop | Medium |
| 37.2.3 | SMT: Generate verification conditions | smt/translator.rs | High |
| 37.2.4 | Verifier: Check invariant validity | verify/contract.rs | High |

### Phase 37.3: Quantifiers (P1)

| Task | Description | Files | Risk |
|------|-------------|-------|------|
| 37.3.1 | Add `Forall`, `Exists` tokens | lexer/token.rs | Low |
| 37.3.2 | Add `Expr::Forall`, `Expr::Exists` | ast/expr.rs | Low |
| 37.3.3 | Parser rules | grammar.lalrpop | Medium |
| 37.3.4 | SMT translation | smt/translator.rs | High |

---

## Exit Criteria for v0.37

1. ✅ `T?` nullable type syntax works
2. ✅ Overflow operators (`+%`, `-%`, `*%`) work
3. ✅ Loop invariants verified by SMT
4. ✅ Quantifiers translate to SMT-LIB
5. ✅ All 122+ tests passing
6. ✅ Spec compliance >= 90%

---

*Last updated: 2026-01-10 | v0.36.0 → v0.37 planning*

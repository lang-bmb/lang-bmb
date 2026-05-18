# Cycle 2939: `let (a, b) = expr` Tuple Destructuring
Date: 2026-05-19

## Re-plan
Cycle 2938 Carry-Forward: `let (a, b) = expr` Rust interpreter grammar 지원 (복잡한 grammar 변경).
이번 사이클에서 구현.

## Scope & Implementation

### Tuple Destructuring 구현

기존: `let (a, b) = expr` — bootstrap만 지원, Rust interpreter 미지원 (타입 에러)
추가: 블록 컨텍스트에서 `let (a, b) = expr;` 지원

**전략**: `Expr::LetTuple` 마커 노드를 도입, `desugar_block_lets`에서 완전 변환:
```
let (a, b) = expr; rest
→ let __tup_N = expr; let a = __tup_N.0; let b = __tup_N.1; rest
```
이 방식으로 type checker, interpreter, MIR lowering에는 `LetTuple`이 도달하지 않음.

**LALR 충돌**: 표현식 컨텍스트 (`Expr` rule)에서 `let (RawIdent,` 시퀀스가
`WhileLetPattern = "(" SpannedPattern "," ...`와 상태를 공유하여 shift-reduce 충돌 발생.
→ 블록 컨텍스트 (`BlockStmt`)에만 지원. `{ let (a, b) = ...; }` 패턴으로 충분.

**변경 파일**:
- `bmb/src/ast/expr.rs`: `Expr::LetTuple` 추가 + `desugar_stmts`에서 확장 로직
- `bmb/src/grammar.lalrpop`: `BlockStmt`에 `"let" "(" names ")" "="` 규칙 추가
- `bmb/src/ast/output.rs`, `cir/lower.rs`, `interp/eval.rs`, `lsp/mod.rs`, `mir/lower.rs`,
  `smt/translator.rs`, `types/mod.rs`, `verify/contract.rs`, `main.rs`:
  `Expr::LetTuple { .. } => unreachable!()` arm 추가 (8개 파일)
- `tests/golden/tuple_destructuring.bmb` + `.out` 신규

### 검증 예시

```bmb
fn main() -> i64 = {
    let t = (10, 20);
    let (a, b) = t;
    println(a + b);         // 30

    let (x, y, z) = (1, 2, 3);
    println(x + y + z);     // 6

    let (lo, hi) = min_max(42, 17);
    println(lo);             // 17
    println(hi);             // 42
    0
};
```

## Verification & Defect Resolution

```
cargo test --release -p bmb: 2388 passed ✅
tuple_destructuring.bmb: 30 / 6 / 17 / 42 / 100 / 200 ✅
```

### 결함 없음

## Reflection

### Scope fit
- ✅ 블록 컨텍스트 tuple destructuring 완전 동작
- ⚠️ 표현식 컨텍스트 미지원 (LALR 충돌) — 설계상 허용 가능한 제한

### 의의
CLAUDE.md의 "BMB가 지원하지 않는 문법" 목록에서 제거됨:
`let (a, b) = expr` — ~~⚠️ bootstrap만 지원, Rust interp. 미지원~~ → ✅ Cycle 2939 추가

## Carry-Forward

- Actionable: 없음 (LALR 충돌로 Expr 컨텍스트 제외는 의도된 설계)
- Structural Improvement Proposals:
  1. **CLAUDE.md 갱신**: "BMB가 지원하지 않는 문법" 섹션에서 `let (a, b)` 제거
  2. **str_byte_at native codegen**: C runtime `bmb_str_byte_at` 함수 추가
  3. **println(String) native dispatch**: native codegen에서 `println_str` 호출 전환
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2940 — native codegen 지원 (str_byte_at + println(String)) 또는 추가 언어 갭

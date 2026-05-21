# Cycle 2834: while let 언어 기능 구현
Date: 2026-05-14

## Re-plan
Plan valid. Cycle 2833 str_split+svec_* 완료 후 inherited carry-forward: `while let PAT = expr { body }` 구현.
Scope: 문법(grammar.lalrpop) + AST(expr.rs) + 타입체커(types/mod.rs) + 인터프리터(interp/eval.rs) + 6개 match 누락 파일 처리.

## Scope & Implementation

**LALR(1) Conflict 해결**:
`while let` 추가 시 `"while" "let" RawIdent "="` prefix에서 LALR(1) conflict 발생:
- WhileLet 방향: RawIdent → AtomicPattern → SpannedPattern 리듀스
- while-expr 방향: `let x = expr` 렉시컬 표현식으로 시프트

**해결책**: `WhileLetPattern` / `SpannedWhileLetPattern` 신규 비단말 추가.
AtomicPattern에서 `<n:RawIdent> => Pattern::Var(n)` case 제외.

**지원되는 while let 패턴** (WhileLetPattern):
- `Enum::Variant(binding)` — 가장 일반적인 용도 (예: `Option::Some(x)`)
- `Enum::Variant` — 인수 없는 variant
- `Struct { field: pat, .. }` — 구조체 패턴
- `Tuple(pat1, pat2)` — 튜플 패턴
- `_` wildcard, 리터럴, 범위 — 지원

**미지원** (의도적 제외):
- `while let x = expr { }` — 항상 매칭, 무한루프 → LALR conflict 유발 + 의미 없음

**변경 파일**:
| 파일 | 변경 내용 |
|------|---------|
| `bmb/src/grammar.lalrpop` | `WhileLetPattern`/`SpannedWhileLetPattern` 비단말 추가 + while-let production 2곳 수정 |
| `bmb/src/ast/expr.rs` | `Expr::WhileLet { pattern, expr, body }` variant 추가 |
| `bmb/src/types/mod.rs` | WhileLet 타입체킹 추가 |
| `bmb/src/interp/eval.rs` | WhileLet 인터프리터 처리 양쪽 추가 |
| `bmb/src/ast/output.rs` | WhileLet format_expr 추가 |
| `bmb/src/cir/lower.rs` | interpreter-only fallback (CirExpr::Unit) |
| `bmb/src/smt/translator.rs` | UnsupportedFeature 반환 |
| `bmb/src/verify/contract.rs` | 재귀 체크 추가 |
| `bmb/src/lsp/mod.rs` | format_expr 추가 |
| `bmb/src/mir/lower.rs` | interpreter-only fallback (Operand::Constant::Unit) |
| `bmb/src/main.rs` | format_expr 추가 |
| `bmb/tests/integration.rs` | test_interp_while_let 추가 (2 cases) |
| `ecosystem/bmb-ai-bench/protocol/bmb_reference.md` | while let 패턴 문서화 |
| `claudedocs/ROADMAP.md` | while let done 마킹, 다음 항목 갱신 |

## Verification & Defect Resolution

```
cargo test --release -p bmb
2361 tests PASSED, 0 FAILED
```

새 테스트 2개 모두 통과:
- `test_interp_while_let` PASS (즉시 종료 case + 반복 누적 case)
- `test_interp_str_split` PASS (Cycle 2833 이월, 이번 첫 실행 확인)

## Reflection

**Scope fit**: while let 핵심 기능 완전 구현 — 문법 파싱, 타입체킹, 인터프리터 실행, 모든 match 누락 처리.

**LALR conflict 분석**: Pattern::Var를 WhileLetPattern에서 제외한 것은 언어 설계적으로도 올바름.
`while let x = expr { }` 는 항상 무한루프이므로 언어 기능으로서 가치 없음.

**Interpreter-only 일관성**: CIR/MIR lowering에서 Unit fallback은 현 정책에서 수용 가능.
향후 native 지원 시 proper lowering 필요 (carry-forward).

**Roadmap impact**: while let done으로 M4 언어 갭 리스트 갱신. 남은 항목: string interpolation / for-in-vec.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - mir/cir lower의 WhileLet => Unit fallback: 향후 native 지원 시 proper lowering으로 교체 필요.
- Pending Human Decisions: None
- Roadmap Revisions: ROADMAP.md M4 while-let done 마킹
- Next Recommendation: Cycle 2835 — String interpolation (`"hello {name}!"`) 또는 `for x in vec {}` 구현. 두 기능 모두 고복잡도. String interpolation이 AI-native 사용에서 더 자주 필요하므로 우선 검토 권장.

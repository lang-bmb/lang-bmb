# Cycle 2822: if-without-else 구현

Date: 2026-05-14

## Re-plan

Plan valid. Inherited scope from Cycle 2821 carry-forward:
- `ISSUE-20260326-integration-category-weakness` → BMB 언어 기능 개선
- 선택 대상: `if cond { body }` (else 없는 if) — integration 카테고리 BMB 63% → Python 76% 격차 해소에 직접 기여

## Scope & Implementation

### 목표
`if cond { body }` (else 절 없는 if 표현식) 파서/타입체커/인터프리터 지원.
기존에는 `if x < 0 { return 99 }` 작성 시 파서 오류 발생 → LLM 코드 생성 마찰 원인.

### 변경 파일

**`bmb/src/grammar.lalrpop`**
- `IfExprOpt: Expr` 신규 nonterminal 추가 (dangling-else 분리 패턴)
- `#[precedence(level="1")]`: else 없는 if (implicit `Unit` else)
- `#[precedence(level="2")]`: `else { ... }` 및 `else SpannedIfExpr` 체인
- `BlockExpr`, `Expr` 에서 기존 if/if-else 규칙 2개 → `IfExprOpt,` 단일 참조로 교체
- `SpannedIfExpr` (else-if 체인용) 미수정

**`bmb/src/types/mod.rs`**
- `unify` 함수에 Never 바텀 타입 처리 추가
- `if x < 0 { return 99 }; 42` 패턴에서 then-branch가 `Never`여도 타입 오류 없이 통과

**`bootstrap/compiler.bmb`**
- Line 1723: implicit else AST 표현을 `(int 0)` → `(unit)` 로 수정
- Line 1795: 동일 수정

**`bmb/tests/integration.rs`**
- `test_interp_if_no_else_side_effect`: 조건 true/false + 연속 if-else 3케이스
- `test_interp_if_no_else_never_branch`: Never 브랜치(guard 함수) 2케이스

### 핵심 설계 결정

**dangling-else 충돌 해소**: lalrpop 0.22에서 `#[precedence]`는 동일 nonterminal의 모든 대안에 적용해야 함. `BlockExpr`/`Expr`는 수십 개 대안을 가지므로 전체 주석 불가. → `IfExprOpt` 전용 nonterminal(3개 대안만 포함) 생성 후 상위에서 단일 참조 방식으로 해결.

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| `cargo test --release -p bmb` | ✅ 2355 passed (new test 포함) |
| Stage 1 bootstrap | ✅ (39443ms, `bootstrap/compiler.bmb` 변경) |
| 3-Stage Fixed Point | 🔄 진행 중 (bf5cnkpio) |
| Never-branch 수동 검증 | ✅ `guard(5)=42`, `guard(-1)=99` |

### 발견 및 수정된 결함

1. **dangling-else 우선순위 역전**: 첫 시도에서 else 없는 if를 단순 추가 → shift/reduce conflict → lalrpop이 REDUCE 선택 → `if-else`가 파싱 오류. `IfExprOpt` + `#[precedence]` 패턴으로 해결.
2. **Never 타입 미처리**: `unify`에서 `Never` vs 임의 타입 비교 시 타입 오류 발생. `matches!(&T, Type::Never)` 조기 반환으로 수정.
3. **bootstrap (unit) 표현 오류**: 묵시적 else가 `(int 0)`으로 표현되어 의미 불일치. `(unit)`으로 수정.

## Reflection

**Scope fit**: 완전히 충족. `if cond { body }` 파싱/타입/인터프리터 모두 동작.

**Latent defects**: `SpannedIfExpr` (else-if 체인 `if a { } else if b { }`) 의 최종 else 없는 경우는 여전히 불가. 이는 별도 ISSUE 범위이며 이 사이클 scope 외.

**Philosophy drift**: 없음. 언어 스펙 개선 → 컴파일러 구현 순서 준수.

**Roadmap impact**: `ISSUE-20260326-integration-category-weakness` 진척. LLM이 `if cond { }` 패턴 사용 가능해져 integration 점수 개선 기대.

**3-Stage**: `bootstrap/compiler.bmb` 변경(unit 표현 수정)으로 Rule 3 적용. Stage 1 ✅, S2/S3 검증 진행 중.

## Carry-Forward

- **Actionable**: 3-Stage Fixed Point 결과 확인 (bf5cnkpio 태스크)
- **Structural Improvement Proposals**: `SpannedIfExpr`도 선택적 else 허용으로 확장 (별도 사이클, 예: `else if` 체인의 마지막에 else 없어도 됨). 설계 복잡도 증가.
- **Pending Human Decisions**: B축 재측정 (API key 필요) — integration 카테고리 if-without-else 개선 효과 수치화
- **Roadmap Revisions**: None
- **Next Recommendation**: B축 재측정 후 integration 카테고리 실패 패턴 재분석 → `SpannedIfExpr` 확장 여부 결정. 또는 다음 자율 사이클에서 `bmb_reference.md` else-if-chain 패턴 보완.

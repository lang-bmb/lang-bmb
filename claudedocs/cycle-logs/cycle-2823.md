# Cycle 2823: SpannedIfExpr 선택적 else 확장 (else-if 체인 최종 else 불필요)

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2822 carry-forward: `SpannedIfExpr` (else-if 체인용) 의 마지막 else 없는 형태 지원. 예: `if a { } else if b { }`.

조사 결과:
- Bootstrap parser (`bootstrap/compiler.bmb`) 는 이미 else 없는 else-if 체인 처리 가능 (line 1722-1724: `else` 없으면 `(unit)` emit). **Bootstrap 변경 불필요**.
- Rust grammar (`grammar.lalrpop`) 의 `SpannedIfExpr`만 수정 필요.
- 타입 체커: Cycle 2822 Never 처리로 이미 충분. `Unit` vs `Unit` unify는 문제 없음.

## Scope & Implementation

**`bmb/src/grammar.lalrpop`** — `SpannedIfExpr` nonterminal 확장

기존 2개 대안(else 블록, else if 체인)에 `#[precedence(level="2")]` 추가.
신규 대안 `#[precedence(level="1")]`: `if cond { body }` (else 없음) → `else_branch: Unit`.

Dangling-else 해소 메커니즘: level="2" (else 있는 형태)가 level="1" (else 없는 형태)보다 높은 우선순위 → `else` 토큰 lookahead 시 SHIFT 선택.

**`bmb/tests/integration.rs`** — `test_interp_else_if_no_final_else` 추가

4가지 케이스:
1. else-if에서 두 번째 조건 매치 → x = 42
2. else-if 어느 조건도 매치 안 됨 → x = 5 (변화 없음)
3. 3단 else-if 체인 (마지막 else 없음) → x = 30
4. else-if 후 final else 있는 경우 (기존 동작 회귀 방지)

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| `cargo build --release` (grammar 재컴파일) | ✅ 1분 44초 |
| `test_interp_else_if_no_final_else` | ✅ 4 케이스 |
| `cargo test --release -p bmb` (전체) | ✅ 2357 passed |
| Bootstrap 변경 여부 | 없음 → 3-Stage 불필요 |

발견된 결함: 없음. Grammar 변경으로 바로 정상 동작.

## Reflection

**Scope fit**: 완전히 충족. `if a { } else if b { }` 파싱 가능.

**Latent defects**: 없음.

**Philosophy drift**: 없음. 언어 갭 해소 → 성능 가설 강화 (LLM이 자연스러운 패턴 사용 가능).

**Roadmap impact**: `ISSUE-20260326-integration-category-weakness` 추가 진척 (else-if 체인 지원). LLM 코드 생성 마찰 추가 감소.

## Carry-Forward

- **Actionable**: `bmb_reference.md` — if-without-else + else-if-chain 패턴 LLM 참조 문서 추가 (HANDOFF 1순위 항목)
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: B축 재측정 (API key 필요)
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2824 — `bmb_reference.md` LLM 참조 문서 갱신 + for-loop 설계 착수

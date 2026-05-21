# Cycle 2700: 회귀 fix (source rename)
Date: 2026-05-11

## Re-plan
🟡 SCOPE ADJUST: 컴파일러 fix는 광범위 영향 → Cycle 2702와 통합. 본 사이클은 source rename + 검증.

## Scope & Implementation

### Source rename
- `tests/bootstrap/test_golden_token_scan.bmb`: `tokenize` → `user_tokenize` (2 occurrences: 정의 + 호출)
- `tests/bootstrap/test_golden_tokenizer.bmb`: 동일 (2 occurrences)

manifest expected는 변경 없음 (출력값 자체는 동일: 10, 5).

### 영향 범위 측정
| 위치 | hardcoded 이름 사용 |
|------|------|
| `tests/bootstrap/test_golden_token*.bmb` | ✅ 본 사이클 fix |
| `bootstrap/*.bmb` | compiler 내부 사용 (정상, String 반환) |
| `ecosystem/gotgan-packages/.../bmb-tokenizer/` | 별도 컴파일 단위 |
| `examples/`, `bootstrap/tests/` | 별도 테스트 스위트 |

골든 스위트 manifest에서 충돌은 위 2개로 한정.

## Verification & Defect Resolution

| 테스트 | 변경 후 |
|--------|---------|
| test_golden_token_scan | ✅ rc=0, stdout="10" (manifest 일치) |
| test_golden_tokenizer | ✅ rc=0, stdout="5" (manifest 일치) |

## Reflection

**핵심 통찰**:
- 5분 source rename으로 2개 회귀 즉시 해소. 그러나 컴파일러 결함은 잔존 (Cycle 2702 처리)
- 사용자가 test 작성 시 "tokenize"라는 일반 이름 회피해야 한다는 제약은 BMB 사용성 손상 — 컴파일러 fix 우선순위 올림

**도그푸딩 가치**:
- 골든 스위트 회귀 감지 게이트 정상 작동 확인 (12 → 1-2 잔여 예상)

**Roadmap impact**:
- Cycle 2702: builtin arity + hardcoded list 정정 통합 작업
- Cycle 2703: Track Q lint 강화 (user fn 이름 = builtin/hardcoded 충돌 감지)

## Carry-Forward
- Actionable:
  - Cycle 2701: 골든 manifest audit 실행 (잔여 mismatch 식별)
  - Cycle 2702: hardcoded String-fn 리스트 정정 (dynamic 우선 또는 제거)
- Structural Improvement Proposals:
  - **테스트 컨벤션**: 골든 테스트에서 hardcoded 이름 회피 가이드 (CLAUDE.md 또는 docs/)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2701 audit (manifest 정정 가능 항목 식별)

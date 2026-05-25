# Cycle 3101: Track B 계약 추가 — collect_/index_/trl_ 배치
Date: 2026-05-25

## Re-plan
계획 유효. M7-4 마지막 Track B 계약 추가 사이클.

## Scope & Implementation

**collect_* 12개**:
- i64 반환 7개 (refs/params/names): `pre pos/start >= 0`, `post it >= 0`
- String 반환 5개: `pre pos/idx >= 0`

**index_* 7개**:
- i64 반환 5개 (skip/find/read): `pre pos >= 0`, `post it >= 0`
- String 반환 3개: `pre pos >= 0`
- bool 반환 (`index_has_name_search`): 건너뜀

**trl_* 6개**:
- i64 반환 2개: `pre pos >= 0`, `post it >= 0/-1`
- String 반환 4개: `pre pos >= 0`
- bool 반환 (`trl_scan_block_for_return`): 건너뜀

**합계**: 24개 계약 추가. 1366 → 1342 미계약 함수.

**M7-4 누적**: 1467 → 1342 = **125개 계약 추가** (cycles 3097-3101)
**전체**: 1513 총 함수 중 171개 계약 = **11.3% 커버리지**

## Verification & Defect Resolution

- `bmb check bootstrap/compiler.bmb`: ✅ (3232 warnings, 0 errors)
- `bmb verify --list-uncontracted`: 1342 ✅

## Reflection

- Scope fit: 100%
- bool 반환 함수 계약: trivially true postcondition → 추가 불필요 (올바른 판단)
- Python 배치 패치 방법이 효율적이고 일관성 있음 — 향후 M8에도 활용 가능
- 125개 추가 증명: list-uncontracted.bmb + suggest_contracts 파이프라인 실제 작동 검증

## Carry-Forward

- Actionable: Cycle 3102 — M7-4 COMPLETE 선언 + ROADMAP/HANDOFF 업데이트
- Structural Improvement Proposals: 배치 계약 추가 스크립트 일반화 (향후 M8용)
- Pending Human Decisions: None
- Roadmap Revisions: M7-4 ✅ COMPLETE 마킹
- Next Recommendation: M8 계획 수립 및 commit

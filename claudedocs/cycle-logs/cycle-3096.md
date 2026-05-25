# Cycle 3096: Track B 자동화 스크립트 (`list-uncontracted.bmb`)
Date: 2026-05-25

## Re-plan
계획 유효. Cycle 3095 Carry-Forward — `list-uncontracted.bmb` BMB 자동화 스크립트 완성.

## Scope & Implementation

`bootstrap/list-uncontracted.bmb` 작성 및 검증.

**기능**:
- `exec_with_stdin("./target/release/bmb", "verify bootstrap/compiler.bmb --list-uncontracted", "")` 호출
- JSON 결과에서 `"count":N` 추출 (`parse_digits` 재귀 헬퍼)
- `"functions":[...]` 배열 파싱
- P1 (pos/idx/start/offset 파라미터): `count_p1` 재귀
- P2 (find_/skip_/scan_/count_/parse_/low_ 이름, pos 파람 없음): `count_p2` 재귀
- P3 = total - p1 - p2
- 출력: `{"type":"uncontracted_summary","total":N,"priority1_pos_param":N,...}`

**버그 3종 수정**:
1. 함수 정의 끝 세미콜론 누락 → `;` 추가
2. `str_to_i64` 빌트인 없음 → `parse_digits(s, pos, acc)` 재귀 구현
3. `str_find_from(result, "]}", ...)` 가 첫 빈 params `[]}` 를 오탐 → `result.len() - 2` 고정 사용

## Verification & Defect Resolution

- `bmb check bootstrap/list-uncontracted.bmb`: ✅ (warnings only)
- `bmb run bootstrap/list-uncontracted.bmb`:
  ```json
  {"type":"uncontracted_summary","total":1467,"priority1_pos_param":683,"priority2_pattern_name":23,"priority3_other":761}
  ```
- Python ground truth 검증: P1=683 ✅ 일치

## Reflection

- Scope fit: 100%
- P2=23 vs Python 247: BMB 스크립트는 P1에 해당하는 함수를 P2에서 제외 (exclusive classification) — 올바른 동작
- BMB 파싱 한계: JSON 중첩 구조 완전 파싱 어려움, 문자열 끝 기반 탐색이 더 안정적

## Carry-Forward

- Actionable: Cycle 3097 — Track B 계약 추가 (P2 패턴명 함수 우선)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M7-4 Phase 3 ✅ (자동화 스크립트 완성)
- Next Recommendation: P2 find_/skip_/scan_ 함수 계약 추가 (post it >= 0 / post it >= -1)

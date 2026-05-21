# Cycle 2864: bmb_reference 정비 (stale 섹션 수정 + 신규 패턴)
Date: 2026-05-15

## Re-plan
Carry-Forward 없음. 최근 여러 사이클에서 추가된 기능들이 bmb_reference.md에 반영되지 않은 stale 내용 발견.
Scope: bmb_reference 문서 정비 (코드 변경 없음, 문서 전용).

## Scope & Implementation
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - Line 118: `{fn_call()}` 불가 → 가능 (v0.98.6+) 로 수정
  - Line 742 (String interpolation 섹션): 동일 stale 수정
  - String Operations 섹션: `str_to_f64`, `read_f64`, `str_lines` 추가 (v0.98.7+)
  - "Iterate str_hashmap keys" 패턴: 구 인덱스 루프 → for-in-svec 직접 반복으로 업데이트
  - `for` pitfall 항목: svec handle iteration (v0.98.7+) 추가
  - Common Pitfalls 문자열 빌틴 목록: `str_to_f64`, `str_lines` 추가
  - 신규 패턴 섹션: "Float parsing and line-by-line text (v0.98.7+)" — parse_two_floats, count_nonempty_lines, sum_n_floats 예제

## Verification & Defect Resolution
- `cargo test --release -p bmb`: **2384 PASS** ✅ (코드 변경 없음)
- defect 없음

## Reflection
- Scope fit: ✅ 3사이클 분량의 stale 내용(Cycle 2855/2861/2863) 일괄 해소
- Line 544의 "for-in-svec not yet supported" 주석이 3사이클 동안 stale 상태였음 — 패턴 기반 발굴의 중요성 재확인
- Roadmap: bmb_reference는 AI가 BMB 코드를 작성할 때 참조하는 1차 자료 — 정확도가 B축 성공률에 직결

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: bmb_reference 자동 검증 (테스트 코드 → 패턴 동기화 체크) — 장기 개선 아이디어
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2865 — 새 언어 갭 발굴 (예: str_trim_left/right, char_is_digit/alpha, vec_slice, 혹은 B축 실패 패턴 분석)

# Cycle 2868: str_reverse + popcount + svec_index_of
Date: 2026-05-15

## Re-plan
Cycle 2867 Next Recommendation 수행. 남은 유용한 갭:
- `str_reverse(s)` — 회문(palindrome) 체크에 필수, 메서드로만 있었음
- `popcount(n)` — 비트 문제에서 set bit 개수 계산
- `svec_index_of(sv, s)` — svec에서 첫 번째 위치 검색

## Scope & Implementation
- `bmb/src/interp/eval.rs`:
  - `builtin_str_reverse`: `chars().rev().collect()`
  - `builtin_popcount`: `count_ones() as i64`
  - `builtin_svec_index_of`: SVEC_REGISTRY에서 position 검색 → i64 or -1
  - 3종 등록
- `bmb/src/types/mod.rs`:
  - `str_reverse(String) -> String`
  - `popcount(i64) -> i64`
  - `svec_index_of(SvecHandle, String) -> i64` (svec_t 이후 위치에 배치)
- `bmb/tests/integration.rs`: `test_interp_str_reverse_popcount_svec_indexof` (8개 assert)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 3종 문서화

## Verification & Defect Resolution
- 첫 컴파일 에러: `svec_t` scope 문제 (str_trim_left 위치 ≈ L421, svec_t 정의 ≈ L425)
  - 수정: `str_reverse`, `popcount`은 str_trim_right 뒤에, `svec_index_of`는 svec_clear 뒤에 배치
- `cargo test --release -p bmb`: **2388 PASS** ✅
- defect 없음

## Reflection
- Scope fit: ✅ 회문 체크 / 비트 조작 / svec 선형 검색 완성
- `svec_t` 스코프 분리 이슈는 현재 types/mod.rs 구조의 특성 — svec_t 이후 위치에 svec 관련 추가 필요 (기억할 것)

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2869 — 마지막 2 사이클: bitwise docs + 새 알고리즘 패턴 추가 or HANDOFF/ROADMAP 정리 + 전체 커밋 준비

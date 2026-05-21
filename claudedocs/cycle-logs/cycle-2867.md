# Cycle 2867: str_split_whitespace + int_to_hex + int_to_bin
Date: 2026-05-15

## Re-plan
Carry-Forward 없음. Next Recommendation 수행.
주요 갭: `str_split("  1 2  3  ", " ")` → 빈 토큰 포함 문제. str_split_whitespace 미존재.
int_to_hex/int_to_bin은 i64 메서드(`n.to_hex()`)로만 존재, free function 없음.

## Scope & Implementation
- `bmb/src/interp/eval.rs`:
  - `builtin_str_split_whitespace(s)` → `split_whitespace()` + SVEC_REGISTRY
  - `builtin_int_to_hex(n)` → `format!("{:x}", n)`
  - `builtin_int_to_bin(n)` → `format!("{:b}", n)`
  - 3종 등록
- `bmb/src/types/mod.rs`:
  - `str_split_whitespace(String) -> SvecHandle`
  - `int_to_hex(i64) -> String`
  - `int_to_bin(i64) -> String`
- `bmb/tests/integration.rs`: `test_interp_split_whitespace_and_int_convert` (7개 assert)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - String Operations: `int_to_hex`, `int_to_bin` 추가
  - str_split 섹션: `str_split_whitespace` 설명 추가

## Verification & Defect Resolution
- `cargo test --release -p bmb`: **2387 PASS** ✅
- defect 없음

## Reflection
- Scope fit: ✅ 경쟁 프로그래밍 입력 파싱에 필수적인 whitespace split 완성
- `str_split(line, " ")`의 빈 토큰 문제는 AI가 자주 겪는 패턴 — str_split_whitespace가 근본 해결
- 언어 갭 해소 속도가 빨라지는 중 (패턴이 명확)

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2868 — 마지막 3 사이클: (1) 남은 갭 탐색 (vec_slice? str_reverse? i64 비트연산?) (2) 또는 HANDOFF/ROADMAP 정리 + 전체 커밋

# Cycle 2843: bmb_reference 패턴 추가 (M4-10/11 문서화)
Date: 2026-05-14

## Re-plan
M4-10, M4-11 완료 후 문서화 단계. 계획 유효.

## Scope & Implementation
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md` 갱신:
  * "Iterate vec by index (no for-in-vec)" → "Iterate vec directly (for-in-vec, v0.98.4+)" + 새 패턴
  * 기존 index-loop 패턴 보존 (native build 호환)
  * Common Pitfalls 갱신 (`for x in vec` 이제 가능)
  * String interpolation 문서 추가
  * 새 패턴 2개: "Vec iteration and transformation" + "String interpolation"

## Verification & Defect Resolution
- cargo test --release: ✅ ALL PASS (no regression)

## Reflection
- ✅ 문서 일관성 회복 (기존 "for x in my_vec does NOT work" 오류 수정).
- ✅ AI가 자연스럽게 for-in-vec + string interpolation 사용하도록 유도.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `+=` 연산자 (현재 `set s = s + x` 필요) — ergonomics 개선
  * HashMap<String, Value> 지원 (현재 i64 key/value only)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `+=` / `-=` 복합 할당 연산자 구현 (빈번한 패턴, 적은 복잡도)

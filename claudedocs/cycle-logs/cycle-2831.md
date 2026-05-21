# Cycle 2831: 알고리즘 패턴 + Common Pitfalls 보강

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2830 carry-forward: 추가 언어 갭 조사 / 문서 패턴 보강.

**범위 결정**: 언어 기능 추가 없이 `bmb_reference.md` 패턴 보강에 집중.
- `for x in vec` 불가 → 명시적 Pitfall + "Iterate vec by index" 패턴 추가
- BFS, prefix sum, find-max — 알고리즘 문제에서 반복 등장
- HashMap 카운팅 패턴
- to_string 관련 Pitfall 추가

## Scope & Implementation

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**:
- "Pattern: Find max/min in vec" 추가
- "Pattern: Prefix sum" 추가
- "Pattern: BFS" 추가
- "Pattern: Iterate vec by index (no for-in-vec)" 추가 — `for x in vec` 불가 명시
- "Pattern: Count occurrences with HashMap" 추가
- Common Pitfalls 5개 추가:
  - `for x in my_vec` 불가 — vec은 i64 핸들
  - `hashmap_get` returns `i64::MIN` 부재 시
  - `to_string(x)` v0.98.2 안내
  - `int_to_string` i64 전용 주의

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| Stage 1 bootstrap | ✅ compiler.bmb 빌드 성공 |
| `cargo test --release -p bmb` (전체) | ✅ 2359 passed (Cycle 2830에서 확인) |

## Reflection

**Scope fit**: 완전히 충족 — 문서화 전용 사이클.

**Latent defects**: 없음. BFS 패턴이 추상적 (인접 리스트 표현 생략) — 그러나 reference는 패턴 가이드이지 완전한 구현이 아님.

**Philosophy drift**: 없음. 문서 보강은 B축 개선에 직접 기여.

**Roadmap impact**: `bmb_reference.md` 패턴이 풍부해짐 → B축 재측정 시 AI 정확도 상승 기대.

## Carry-Forward

- **Actionable**: Cycle 2832 — HANDOFF/ROADMAP 최종 갱신 + 전체 세션 커밋
- **Structural Improvement Proposals**: `split` 함수(문자열 → vec) 구현 — LLM이 자주 필요로 하나 BMB에 미존재 (P3)
- **Pending Human Decisions**: B축 재측정
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2832 — 문서 갱신 + 최종 커밋

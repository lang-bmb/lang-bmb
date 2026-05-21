# Cycle 2824: bmb_reference.md LLM 참조 문서 갱신

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2823 carry-forward: `bmb_reference.md` — if-without-else + else-if-chain 패턴 추가.

추가 발견: HANDOFF에서 "for-loop 미지원"이라고 했으나, `for i in 0..n { }` (range 기반)은 이미 지원됨. `bmb_reference.md`에도 이미 존재. HANDOFF의 "for-loop" 갭은 iterator protocol (컬렉션 순회)을 가리킴.

## Scope & Implementation

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**

1. **CRITICAL: if-else Rules** — 완전 재작성
   - `if` as VALUE: 여전히 `else` 필수 (값 반환)
   - `if` as STATEMENT: `else` 선택적 (v0.98.1+)
   - `else if` 체인: 최종 `else` 선택적 (v0.98.1+)
   - `else if` chain as VALUE: 여전히 final `else` 필수

2. **Common Pitfalls** — 갱신
   - "if/else used as statement MUST have `else { () }`" → 제거
   - 신규: "if as VALUE needs else, if as STATEMENT else optional"
   - 신규: "for only supports ranges, not arbitrary iterators"

3. **패턴 정리** — 불필요한 `else { () }` 제거:
   - Multi-way dispatch (side-effect) → `else { () }` 제거
   - Key-value store → `else { () }` 제거
   - Selection sort → `else { () }` 제거
   - Print space-separated array → `if i > 0 { ... } else { () }` → `if i > 0 { ... }`
   - Read until n commands → trailing `()` 정리

## Verification & Defect Resolution

- `else { () }` 잔존 여부: 0건 (grep 확인)
- 나머지 `else { }` 패턴: 모두 값 반환 위치 (정확)
- Bootstrap/cargo test: 문서 변경 → 불필요

## Reflection

**Scope fit**: 완전히 충족.

**Latent defects**: 없음.

**HANDOFF 정정**: "for-loop 미지원" → range 기반은 지원됨. iterator protocol이 미지원. 다음 세션 HANDOFF 갱신 필요.

**Roadmap impact**: LLM 참조 문서 품질 향상. 2822+2823 변경 사항이 LLM에 전달될 수 있는 경로 확보.

## Carry-Forward

- **Actionable**: HANDOFF "for x in iter { }" 설명 정확화 (range 기반은 지원, iterator protocol 미지원 명시)
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: None
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2825 — for-loop 범위 명확화 후 while-let 또는 string interpolation 착수. 또는 `for x in arr` (vec 순회) 검토.

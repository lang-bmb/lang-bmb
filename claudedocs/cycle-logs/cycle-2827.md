# Cycle 2827: HashMap Builtins 문서화 + 다음 방향 결정

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2826 carry-forward: hashmap builtins 문서화.

추가 조사:
- `hashmap_get` → 키 없으면 `i64::MIN` 반환 (not -1!)
- `hashmap_contains` → 1/0 반환
- `str_len` 외 문자열 builtins 없음 (`str_contains`, `str_starts_with` 등 부재)

## Scope & Implementation

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**

새 **HashMap** 섹션 추가:
- `hashmap_new`, `hashmap_insert`, `hashmap_contains`, `hashmap_get`, `hashmap_len`, `hashmap_free`
- `hashmap_get` 의 정확한 not-found sentinel: `i64::MIN` (not -1)
- get-with-default 패턴: `hashmap_contains` 체크 후 분기

## Verification & Defect Resolution

문서 변경. 발견된 중요 사항: `hashmap_get`의 not-found sentinel이 `-1`이 아닌 `i64::MIN`.
85_registry_pattern 해결 시 LLM이 `hashmap_contains` + 분기 패턴 필수.

## Reflection

**다음 사이클 방향** (5 사이클 남음, 2828-2832):
- 가장 가치 있는 구현: 문자열 처리 builtins 추가 (`str_contains`, `str_starts_with`, `str_ends_with`, `str_find`, `str_substr`)
- 이들은 interpreter runtime 추가 (Rule 6 적용 범주 외 — runtime library, not compiler)
- 2828-2829: 문자열 builtins 구현 + 문서화
- 2830-2831: format_str 또는 추가 언어 기능
- 2832: 최종 통합 + HANDOFF

## Carry-Forward

- **Actionable**: `str_contains(s, sub)`, `str_starts_with(s, prefix)`, `str_ends_with(s, suffix)`, `str_find(s, sub)` 구현 — Rust interpreter builtins 추가
- **Structural Improvement Proposals**: None
- **Pending Human Decisions**: B축 재측정
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2828 — 문자열 처리 builtins 구현

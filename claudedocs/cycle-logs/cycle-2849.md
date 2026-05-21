# Cycle 2849: str_hashmap_keys / str_hashmap_sorted_keys
Date: 2026-05-14

## Re-plan
Carry-Forward (2848): str_hashmap keys iterator, `{fn(args)}` 함수 호출 보간, 필드 복합 할당 native 지원.
str_hashmap keys iterator 우선 처리 — AI 생성 word frequency/집계 코드에서 key 순회가 필요.

## Scope & Implementation

**접근법**: 기존 `SVEC_REGISTRY`를 재활용해 `str_hashmap_keys` / `str_hashmap_sorted_keys` 구현.
반환 타입 = `i64` (svec handle) → 기존 `svec_len/svec_get/svec_free` API로 처리 가능.

변경 파일:
- `bmb/src/interp/eval.rs`:
  - `builtin_str_hashmap_keys`: HashMap keys를 SVEC_REGISTRY에 저장, handle 반환
  - `builtin_str_hashmap_sorted_keys`: keys 알파벳 정렬 후 SVEC_REGISTRY 저장
  - `init_builtins()`에 2개 등록
- `bmb/src/types/mod.rs`:
  - `str_hashmap_keys: (i64) -> i64` 신규
  - `str_hashmap_sorted_keys: (i64) -> i64` 신규
- `bmb/tests/integration.rs`: `test_interp_str_hashmap_keys` 2개 케이스 추가
  - keys count 검증
  - sorted_keys 순서 + 값 조회 검증
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - String HashMap 섹션에 keys/sorted_keys 설명 추가
  - "Pattern: Iterate str_hashmap keys" 신규

## Verification & Defect Resolution
- test_interp_str_hashmap_keys: 1/1 (2 케이스) 통과 ✅
- cargo test --release 전체: **2373 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ str_hashmap keys iteration 완성 — AI가 word frequency 결과를 정렬된 순서로 순회 가능
- ✅ SVEC_REGISTRY 재활용으로 최소 코드로 구현 (새 레지스트리 불필요)
- **인사이트**: `str_hashmap_sorted_keys`가 `str_hashmap_keys`보다 AI 생성 코드에 훨씬 유용 (비결정적 순서 회피)
- 제한: interpreter-only. `for x in svec` 미지원 (index loop 사용)

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `{fn(args)}` 함수 호출 보간 지원 (InterpMini에 call 파싱 추가)
  * `for x in svec {}` 지원 (현재 svec index loop만 가능)
  * 필드 복합 할당 native 지원
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `{fn_call(args)}` 보간 or `for x in svec {}` 지원

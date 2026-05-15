# Cycle 2851: str_hashmap_delete + str_hashmap_update
Date: 2026-05-15

## Re-plan
Carry-Forward (2850): `str_hashmap_delete(map, key)` 명시적 actionable. Plan valid, inherited scope.

## Scope & Implementation

**str_hashmap_delete** (Cycle 2851, interpreter-only):
- `str_hashmap_delete(map: i64, key: String) -> ()`: `map.remove(&key)` — absent key는 no-op
- Word frequency 정정, sliding-window cache 등 패턴에서 필요

**str_hashmap_update** (Cycle 2851, interpreter-only):
- `str_hashmap_update(map: i64, key: String, val: i64) -> ()`: `map.insert(key, val)` — 의미론적으로 "덮어쓰기"
- `str_hashmap_insert`와 구현 동일하나 의도 명확화 (insert = new, update = overwrite)

변경 파일:
- `bmb/src/interp/eval.rs`: `builtin_str_hashmap_delete`, `builtin_str_hashmap_update` 구현 + 등록
- `bmb/src/types/mod.rs`: 2종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_str_hashmap_delete_update` (3케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: delete/update 패턴 문서화

## Verification & Defect Resolution
- test_interp_str_hashmap_delete_update: 3/3 통과 ✅
- cargo test --release 전체: **2376 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ str_hashmap API 완성 — new/insert/get/contains/len/free/keys/sorted_keys/inc/delete/update 11종
- ✅ `str_hashmap_update`는 `str_hashmap_insert`의 의미론적 분리 — AI 코드 생성 의도 명확화
- 경고: `InterpMini::consume` dead_code 경고 (Cycle 2848부터 존재, 실기능 영향 없음)

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * `{fn_call(args)}` 보간 — InterpMini에 함수 호출 파싱 추가
  * 필드 복합 할당 native 지원 (codegen)
  * vec_remove/reverse/fill + svec_sort/contains/remove — API 갭
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `str_to_upper` / `str_to_lower` / `str_char_at` (Cycle 2852) 또는 `vec_remove/reverse/fill` (Cycle 2853)

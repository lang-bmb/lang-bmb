# Cycle 2854: svec_sort / svec_contains / svec_remove / svec_clear
Date: 2026-05-15

## Re-plan
Carry-Forward (2853): svec API 갭 해소가 명시적 항목. Plan valid, inherited scope.

## Scope & Implementation

**svec_sort** (Cycle 2854, interpreter-only):
- `svec_sort(handle: i64) -> ()`: SVEC_REGISTRY borrow_mut → `v.sort()` (lexicographic)

**svec_contains** (Cycle 2854, interpreter-only):
- `svec_contains(handle: i64, s: String) -> i64`: 1 if found, 0 otherwise — `v.contains(&s)`

**svec_remove** (Cycle 2854, interpreter-only):
- `svec_remove(handle: i64, idx: i64) -> ()`: `v.remove(idx)` — bounds check 포함

**svec_clear** (Cycle 2854, interpreter-only):
- `svec_clear(handle: i64) -> ()`: `v.clear()` — handle 유효, 내용만 제거

변경 파일:
- `bmb/src/interp/eval.rs`: 4종 함수 구현 + 등록
- `bmb/src/types/mod.rs`: 4종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_svec_sort_contains_remove_clear` (4케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: svec manual-build 섹션 갱신

## Verification & Defect Resolution
- test_interp_svec_sort_contains_remove_clear: 4/4 통과 ✅
- cargo test --release 전체: **2379 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ svec API 완성 — new/push/len/get/join/free/sort/contains/remove/clear 10종
- ✅ vec API도 완성 — new/with_capacity/push/pop/get/set/len/cap/free/clear/sum/max/min/sort/contains/index_of/remove/reverse/fill 19종
- ✅ str_hashmap API 완성 — new/insert/get/contains/len/free/keys/sorted_keys/inc/delete/update 11종
- ✅ str_to_upper/lower/char_at 추가로 string API 대폭 강화

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * `{fn_call(args)}` 보간 — InterpMini에 함수 호출 파싱 추가
  * 필드 복합 할당 native 지원 (codegen)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `{fn_call(args)}` 보간 구현 (Cycle 2855) — InterpMini 확장

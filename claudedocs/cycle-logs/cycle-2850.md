# Cycle 2850: svec_new/push + str_hashmap_inc
Date: 2026-05-14

## Re-plan
Carry-Forward (2849): `{fn_call(args)}` 보간, `for x in svec {}`, str_hashmap_inc, svec_new/push.
`for x in svec {}` 은 svec handle(small index)과 vec handle(raw ptr)이 모두 `i64`라 타입 구분 불가 → 스킵.
`svec_new/push` + `str_hashmap_inc` 구현.

## Scope & Implementation

**svec_new / svec_push** (Cycle 2850, interpreter-only):
- `svec_new() -> i64`: 빈 Vec<String>을 SVEC_REGISTRY에 추가, handle 반환
- `svec_push(handle, str) -> ()`: 기존 svec에 문자열 추가
- 기존 `svec_len/svec_get/svec_free/svec_join`과 함께 완성된 svec API

**str_hashmap_inc** (Cycle 2850, interpreter-only):
- `str_hashmap_inc(map, key, delta) -> ()`: `entry.or_insert(0) += delta` 패턴
- word frequency 문제에서 `contains + get + insert` 3단계 → 1단계로 단순화

변경 파일:
- `bmb/src/interp/eval.rs`: `builtin_svec_new`, `builtin_svec_push`, `builtin_str_hashmap_inc` 구현 + 등록
- `bmb/src/types/mod.rs`: 3종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_svec_new_push` (2케이스) + `test_interp_str_hashmap_inc` (3케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 섹션 갱신 + 인터폴레이션 표현식 지원 명시

## Verification & Defect Resolution
- test_interp_svec_new_push: 2/2 통과 ✅
- test_interp_str_hashmap_inc: 3/3 통과 ✅
- cargo test --release 전체: **2375 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ svec API 완성 — new/push/len/get/join/free 6종으로 완전한 string vector 지원
- ✅ str_hashmap_inc — word frequency 패턴이 3줄 → 1줄로 단순화, AI 생성 코드 품질 향상
- **인사이트**: `for x in svec {}` 스킵 — `i64` 타입에 svec/vec 구분 불가. 향후 `Value::SvecHandle` 별도 값 타입 도입 시 자연스럽게 지원 가능.
- **세션 총계** (Cycle 2846-2850): 5개 사이클 완성. 2370→2375 (+5 tests). 언어 갭 14종 구현.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * `{fn_call(args)}` 보간 — InterpMini에 함수 호출 파싱 추가
  * 필드 복합 할당 native 지원 (codegen)
  * `str_hashmap_delete(map, key)` — 키 삭제 기능
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `for x in svec {}` (Value::SvecHandle 타입 추가) or `{fn_call()}` 보간

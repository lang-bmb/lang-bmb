# Cycle 2853: vec_remove / vec_reverse / vec_fill
Date: 2026-05-15

## Re-plan
Carry-Forward (2852): None. 계획대로 vec API 갭 해소.

## Scope & Implementation

**vec_remove** (Cycle 2853, interpreter-only):
- `vec_remove(vec: i64, idx: i64) -> i64`: 인덱스 요소 제거 후 좌측 shift, 제거 값 반환
- 범위 밖 idx → RuntimeError

**vec_reverse** (Cycle 2853, interpreter-only):
- `vec_reverse(vec: i64) -> ()`: 두 포인터(lo/hi) 방식으로 in-place 역순 정렬

**vec_fill** (Cycle 2853, interpreter-only):
- `vec_fill(vec: i64, val: i64) -> ()`: 모든 기존 요소를 val로 덮어씀 (len 변경 없음)

메모리 모델: header = [ptr, len, cap] (각 8byte), data = ptr이 가리키는 연속 i64 배열.

변경 파일:
- `bmb/src/interp/eval.rs`: 3종 함수 구현 + 등록
- `bmb/src/types/mod.rs`: 3종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_vec_remove_reverse_fill` (3케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: 3종 문서화 + notes 갱신

## Verification & Defect Resolution
- test_interp_vec_remove_reverse_fill: 3/3 통과 ✅
- cargo test --release 전체: **2378 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ vec API 확장 — remove/reverse/fill로 표준 동적 배열 조작 완성
- ✅ vec_remove: 실제 메모리 shift 구현 (memmove 없이 loop) — 올바른 구현
- vec_reverse의 lo/hi 교차 조건: `len=0` 시 `hi = len-1` 언더플로 방지 처리 완료

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * `{fn_call(args)}` 보간 — InterpMini에 함수 호출 파싱 추가
  * 필드 복합 할당 native 지원 (codegen)
  * svec_sort/contains/remove/clear — svec API 갭
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `svec_sort` / `svec_contains` / `svec_remove` (Cycle 2854) — svec API 완성

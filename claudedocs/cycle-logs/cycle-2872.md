# Cycle 2872: vec aggregate builtins native 포팅 (9종)
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2871에서 str 12종 완료. 이번 사이클: vec 집계/변환 builtins 네이티브 포팅.

## Scope & Implementation
`bmb/src/codegen/llvm_text.rs` 3곳 수정:

1. **IR 선언 추가** (vec 섹션): 9종 C 래퍼 함수 declare
2. **runtime_param_type 추가**: vec 집계 함수들의 인수 타입을 i64로 명시
3. **infer_call_return_type 추가**: void 반환 3종 (vec_sort/reverse/fill) 명시

포팅된 함수:
| BMB 함수 | C 런타임 함수 | 반환형 |
|----------|-------------|--------|
| vec_sum | vec_sum | i64 |
| vec_min | vec_min | i64 |
| vec_max | vec_max | i64 |
| vec_contains | vec_contains | i64 |
| vec_index_of | vec_index_of | i64 |
| vec_remove | vec_remove | i64 |
| vec_sort | vec_sort | void |
| vec_reverse | vec_reverse | void |
| vec_fill | vec_fill | void |

주요 발견:
- `vec_free` 는 이미 인라인 IR 생성 특별 경로 존재 → 런타임 함수 불필요
- void 반환 함수: `infer_call_return_type`에 명시적 "void" 추가 필요
- i64 반환 함수: 기본 `_ => "i64"` 가 적용되어 별도 항목 불필요

## Verification & Defect Resolution
- `tests/native_vec_builtins.bmb`: `bmb run` 출력 = `bmb build` 출력 = 13행 동일 ✅
- IR: `call i64 @vec_sum(...)`, `call void @vec_sort(...)` 올바른 타입으로 방출 ✅
- `cargo build --release`: 0 errors ✅

## Reflection
- Scope fit: ✅ vec 9종 네이티브 포팅 완료
- C 런타임에 이미 래퍼 함수 존재 (vec_sum = wrapper for bmb_vec_sum) → 최소 변경
- void 반환 함수 3종은 `infer_call_return_type` 에 명시해야 LLVM IR 올바르게 방출됨
- 이로써 interpreter-only → native 포팅 가속: str 12 + vec 9 = 21종 이번 세션

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2873 — int_to_hex/int_to_bin native 포팅 + 통합 테스트 보강

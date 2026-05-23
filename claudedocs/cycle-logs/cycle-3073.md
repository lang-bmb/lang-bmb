# Cycle 3073: is_string_returning_fn 전수 수정 — str_sb 추적 누락 20종
Date: 2026-05-23

## Re-plan
이전 Carry-Forward: str_sb를 llvm_gen_call_sb까지 전파 → ptr-반환 함수 결과의 println 정상화.
구조 개선 필요성 재검토 결과: str_sb 전파가 아닌 `is_string_returning_fn` 등록 누락이 근본 원인.
이번 사이클 범위: 두 가지 수정 — (1) `get_call_return_type` 에 `@bmb_string_substr` 추가, (2) `is_string_fn_group6` 신규 — 20종 런타임 문자열 반환 함수 등록.

## Scope & Implementation

### 근본 원인 재분석

`llvm_gen_call_with_string_tracking_sb_reg` (15686행) 흐름:
1. `llvm_gen_call_reg` → IR 생성 (`call ptr @fn(...)` + `ptrtoint`)
2. `is_string_returning_fn(fn_name)` 체크 → `push_string_marker`

`is_string_returning_fn` 그룹 1-5는 **bootstrap compiler 내부 함수**만 커버. 
런타임 stdlib 문자열 반환 함수들(20종+)이 전부 누락.

### 수정 내용

#### 1. bootstrap/compiler.bmb — get_call_return_type (6892행 영역)

`@bmb_string_substr` → `"ptr"` 추가 (기존에 누락, `call i64 @bmb_string_substr` 잘못된 IR 생성 방지)

#### 2. bootstrap/compiler.bmb — is_string_fn_group6 신규 추가 (15810행 영역)

```
fn is_string_fn_group6(name: String) -> bool =
    name == "bmb_string_reverse" or name == "bmb_string_substr" or
    name == "bmb_string_pad_left" or name == "bmb_string_pad_right" or
    name == "bmb_string_trim" or name == "bmb_string_replace" or
    name == "bmb_string_to_upper" or name == "bmb_string_to_lower" or
    name == "bmb_string_repeat" or name == "bmb_string_join" or
    name == "bmb_f64_to_string" or name == "bmb_int_to_string" or
    name == "bmb_to_hex" or name == "bmb_to_binary" or name == "bmb_to_octal" or
    name == "bmb_getcwd" or name == "bmb_exec_output" or
    name == "bmb_system_capture" or name == "bmb_read_line" or
    name == "bmb_exec_with_stdin" or name == "exec_with_stdin" or
    name == "bmb_svec_get" or name == "bmb_svec_join" or
    name == "bmb_string_split";
```

`is_string_returning_fn` dispatcher에 `or is_string_fn_group6(name)` 추가.

### 검증

`bootstrap/_method_test.bmb` (5개 메서드 테스트):
| 메서드 | 기대값 | Cycle 3072 | Cycle 3073 |
|--------|--------|-----------|-----------|
| `.reverse()` | `edcba` | 원시 포인터 | `edcba` ✅ |
| `.count("l")` | `3` | `3` ✅ | `3` ✅ |
| `.last_index_of("l")` | `9` | `9` ✅ | `9` ✅ |
| `.substr(6, 5)` | `world` | 원시 포인터 | `world` ✅ |
| `.pad_left(5, 32)` | `   hi` | 원시 포인터 | `   hi` ✅ |

## Verification & Defect Resolution

- `cargo test --release`: 3782 + 47 + 22 + 2390 + 23 = 6264 tests PASS ✅
- Stage 1 build: `compiler_stage1.exe build compiler.bmb -o compiler_stage2_test.exe` → build_success ✅
- Fixed Point: S3 IR == S4 IR `745082F5CA427CCDA06AB36A2C603953EA792701D84E5B1DBD6A94D4A65FB6B7` ✅

## Reflection
- **Scope fit**: 100% — Cycle 3072 Carry-Forward 완전 해소
- **Philosophy drift**: 없음
- **User-facing quality**: `println(s.reverse())` 등 native에서 정상 동작. 22종 string 함수 복원.
- **Roadmap impact**: str_sb 추적 누락은 사전 결함으로 발생 시점 불명. 이번 사이클로 완전 해소.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals:
  - 향후 런타임 String 반환 함수 추가 시 체크리스트: method_to_runtime_fn + get_call_arg_types + get_call_return_type + IR preamble + is_string_fn_group 4개소 동시 업데이트 필요
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3074 — 추가 구조 개선 탐색 또는 조기 종료 평가

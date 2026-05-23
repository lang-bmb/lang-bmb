# Cycle 3072: method_to_runtime_fn 변경 native 검증 + println dispatch 사전 결함 문서화
Date: 2026-05-23

## Re-plan
이전 Carry-Forward: Cycle 3070 allowlist 변경 사항을 실제 native build로 검증.
이번 사이클 범위: `bootstrap/_method_test.bmb` 를 신규 stage2 바이너리로 native 빌드하여 동작 확인.

## Scope & Implementation

### 검증 과정

1. `compiler_stage2_new.exe` (Cycle 3070 반영 바이너리) 로 `_method_test.bmb` native 빌드
2. 결과 실행 후 출력 분석

### 검증 결과

| 메서드 | 반환 타입 | 기대값 | 실제 출력 | 상태 |
|--------|-----------|--------|-----------|------|
| `.count("l")` | i64 | `3` | `3` | ✅ |
| `.last_index_of("l")` | i64 | `9` | `9` | ✅ |
| `.reverse()` | String (ptr) | `edcba` | 원시 포인터 값 | ❌ |
| `.substr(6, 5)` | String (ptr) | `world` | 원시 포인터 값 | ❌ |
| `.pad_left(5, 32)` | String (ptr) | `   hi` | 원시 포인터 값 | ❌ |

### 근본 원인 분석

`bootstrap/compiler.bmb` 의 `llvm_gen_call_sb` 함수 (~7444행):

```
} else if ret_type == "ptr" {
    conversions + "  " + dest + "_ptr = call ptr " + emitted_fn + "(" + formatted_args + ")" + SEP()
              + "  " + dest + " = ptrtoint ptr " + dest + "_ptr to i64"
```

ptr-반환 함수는 `ptrtoint ptr X to i64` 로 변환되어 i64 레지스터에 저장된다.
이때 `str_sb` (문자열 타입 추적 집합) 에 `dest` 가 등록되지 않아 `println` 이 i64 dispatch 경로를 선택.

### 사전 결함 여부 확인

이 문제는 Cycle 3070 이전부터 존재하는 **사전 결함**이다:
- `trim()`, `replace()`, `to_upper()`, `to_lower()` 등 기존 string→ptr 반환 메서드 모두 동일 문제
- Cycle 3070에서 새로 추가한 매핑(reverse/substr/pad_left)이 이 경로를 탔을 뿐
- Cycle 3070 변경은 **회귀가 아님** — 기존 silent link error를 명시적 동작으로 교체한 것이 목표였고 그것은 달성됨

### 수정 가능 여부

수정하려면 `str_sb` 를 `llvm_gen_call_sb` 에 전달해야 함:
- 현재 `str_sb` 는 `gen_fn_lines_structs` (~14710행) 레벨에서만 관리됨
- `llvm_gen_call_sb` 는 sig만 받고 str_sb 접근 없음
- 수정 범위: 최소 3개 함수 시그니처 변경 + 모든 호출부 업데이트 (10+개소)
- 멀티 사이클 구조 변경 → 이번 사이클 범위 초과

## Verification & Defect Resolution

- Cycle 3070 목표(catch-all → allowlist) 검증: ✅ 완료
  - 새 catch-all `__unknown_method_X` 경로가 i64/string 반환 메서드 모두에 작동
  - 기존 catch-all이 생성했을 `bmb_count`, `bmb_reverse` 등 undefined 함수 → 이제 올바른 이름 생성
- println dispatch 사전 결함: 문서화만 (수정은 Carry-Forward)

## Reflection
- **Scope fit**: 100% — 검증 목표 달성, 사전 결함 발견은 부가적 발견
- **Philosophy drift**: 없음
- **User-facing quality**: str-반환 메서드의 native println 오작동은 실사용 상 제약. 단, 해석기에서는 정상 동작.
- **Roadmap impact**: 구조 개선 (str_sb propagation) 이 M7 구조 개선 후보로 추가됨

## Carry-Forward
- Actionable: str_sb를 llvm_gen_call_sb까지 전파 → ptr-반환 함수 결과의 println 정상화
- Structural Improvement Proposals:
  - `llvm_gen_call_sb` 시그니처에 `str_sb` 추가, 호출부 업데이트 (~15개소)
  - 완료 시 native에서도 `println(s.reverse())` 정상 동작
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3073 — str_sb propagation 수정 또는 다른 구조 개선 탐색

# Cycle 3070: method_to_runtime_fn catch-all → allowlist 교체
Date: 2026-05-23

## Re-plan
이전 Carry-Forward: method_to_runtime_fn catch-all 위험 패턴 (M7 scope 구조 개선).
이번 사이클 범위: catch-all 제거 + explicit allowlist 완성 + 3-Stage Fixed Point 검증.

## Scope & Implementation

### 문제 분석

`method_to_runtime_fn` 함수의 catch-all `else { "bmb_" + method }` 패턴은:
1. 존재하지 않는 함수 이름 생성 가능 (`char_code_at` → `bmb_char_code_at`, 실제 없음)
2. 해석기에서는 동작하지만 native build에서 link error 발생
3. 오류 메시지가 불명확 (링크 오류만 발생, 원인 추적 어려움)

### 수정 내용

**bootstrap/compiler.bmb 3개소**:

#### 1. method_to_runtime_fn (5921행 영역) — explicit 매핑 10개 추가 + catch-all 교체
```
// 추가:
split → bmb_string_split
reverse → bmb_string_reverse
pad_left → bmb_string_pad_left
pad_right → bmb_string_pad_right
count → bmb_string_count
last_index_of → bmb_string_last_index_of
substr → bmb_string_substr
abs → bmb_abs      (catch-all로도 동작했으나 명시화)
min → bmb_min      (catch-all로도 동작했으나 명시화)
max → bmb_max      (catch-all로도 동작했으나 명시화)

// 변경:
else { "bmb_" + method }
→ else { "__unknown_method_" + method }
```

#### 2. get_call_arg_types — @bmb_string_substr 추가
`@bmb_string_substr` → `"pii"` (ptr, start: i64, len: i64)

#### 3. IR preamble — @bmb_string_substr 선언 추가
`declare noalias nonnull ptr @bmb_string_substr(ptr nocapture readonly, i64, i64) nofree nosync nounwind willreturn`

### 조사 결과

- `abs`/`min`/`max`: catch-all → `bmb_abs/min/max` → special-case handler → LLVM intrinsic 경로로 정상 동작했음. 이번에 명시화.
- `split`/`reverse`/`pad_left`/`pad_right`/`count`/`last_index_of`: IR preamble에 이미 선언 있음. get_call_arg_types에도 이미 등록. method_to_runtime_fn 매핑만 누락.
- `substr`: IR preamble + get_call_arg_types 모두 누락 → 3개소 모두 추가.
- 새 catch-all `__unknown_method_X`: 링크 시 "undefined reference to __unknown_method_unknown_name__" 형태로 즉시 진단 가능.

## Verification & Defect Resolution

- `cargo test --release`: 3782 + 47 + 22 + 2390 = 6241 tests PASS (0 failed) ✅
- Stage 1 build: `compiler_stage1.exe build compiler.bmb -o compiler_stage2_test.exe` → build_success ✅
- Fixed Point: `compiler_stage2_test.exe` IR 두 번 생성 → SHA256 동일 ✅ `528975...AE2EF`

## Reflection
- **Scope fit**: 100%
- **Philosophy drift**: 없음 — 컴파일러 결함 수정 (silent error → explicit error)
- **Roadmap impact**: method_to_runtime_fn 구조 개선 완료. M7에서 더 이상 carry-forward 불필요.
- **Latent defect 발견**: `substr` 메서드가 native build에서 link error를 일으켰을 것. 이번 사이클에서 해소.

## Carry-Forward
- Actionable: gotgan build/check PATH 개선 (bmb_exe_path() 활용)
- Structural Improvement Proposals:
  - 향후 builtin 추가 시 체크리스트: method_to_runtime_fn + get_call_arg_types + IR preamble 3개소 동시 업데이트 필요
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3071 — gotgan PATH 의존성 개선 또는 다른 구조 개선 탐색

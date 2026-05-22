# Cycle 3065: bootstrap svec_*/str_lines/make_dir native 지원 추가
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3064): Cycle 3065 — bootstrap/compiler.bmb 4개소 수정.
STEP 0: 계획 유효. gotgan.bmb 네이티브 빌드를 막는 누락 함수들 추가.

## Scope & Implementation

### 수정 1: `get_call_return_type` (line ~6884)
- `@bmb_svec_get`, `@bmb_svec_join` → `"ptr"` 반환 추가

### 수정 2: `map_runtime_fn_full` (line ~7060)
총 16개 매핑 추가:
- `@str_contains` → `@bmb_string_contains`
- `@str_find` → `@bmb_string_index_of`
- `@str_trim` → `@bmb_string_trim`
- `@svec_new/push/len/get/free/join/index_of/contains/sort/remove/clear` → `@bmb_svec_*`
- `@str_lines` → `@bmb_str_lines`
- `@make_dir` → `@make_dir`

### 수정 3: `get_call_arg_types` (line ~7856)
14개 arg type 시그니처 추가:
- `@bmb_svec_new` → `""`
- `@bmb_svec_push` → `"ip"`, `@bmb_svec_len` → `"i"`, `@bmb_svec_get` → `"ii"`
- etc.

### 수정 4: IR preamble (line ~13965)
13개 LLVM declare 추가:
```
declare i64 @bmb_svec_new() ...
declare i64 @bmb_svec_push(i64, ptr ...) ...
... (svec 11종)
declare i64 @bmb_str_lines(ptr ...) ...
declare i64 @make_dir(ptr ...) ...
```

### 수정 5: `method_to_runtime_fn` (line ~5880) — 추가 발견
- `char_code_at` → `bmb_string_char_at` (gotgan.bmb에서 사용, bootstrap catch-all이 `@bmb_char_code_at`으로 잘못 매핑)
- `str_contains`, `str_find`, `str_trim`은 gotgan.bmb에서 free function으로 호출 → map_runtime_fn_full에 추가 필요 (수정 2에 포함)

### 빌드 이터레이션
- 1차 빌드: `@bmb_char_code_at` 미존재 오류 → `char_code_at` method 매핑 추가
- 2차 빌드: `@str_contains` 미존재 오류 → str_ free function 매핑 추가
- 3차 빌드: `{"type":"build_success"}` ✅

## Verification & Defect Resolution

| 검증 항목 | 결과 |
|---------|------|
| Stage 1 빌드 | ✅ |
| gotgan.bmb native 빌드 | ✅ `{"type":"build_success"}` |
| `gotgan.exe help` | ✅ 6개 명령 출력 |
| `gotgan.exe new myapp_test` | ✅ 프로젝트 생성 |
| `gotgan.exe tree` | ✅ dep tree 출력 |
| cargo test --release | ✅ 0 failed |
| 3-Stage Fixed Point S3==S4 | ✅ diff = 0 |
| gotgan golden tests | ✅ 100/100 |

### 알려진 한계 (기존 Known Issue)
- `gotgan.exe check` / `gotgan.exe build`: PATH에 `bmb` binary 필요 — 기존 Known Issue, 이번 사이클 범위 외

## Reflection
- **Scope fit**: 100% — gotgan.bmb 네이티브 컴파일 블로커 완전 해소
- **발견**: bootstrap catch-all `"bmb_" + method`가 `char_code_at`을 `bmb_char_code_at`으로 매핑. `str_contains/find/trim`은 free function으로 호출됨 — 이들의 bootstrap 매핑 누락. 두 경우 모두 이번 사이클에서 수정.
- **Fixed Point 보존**: compiler.bmb는 svec_*/str_*/make_dir 함수를 직접 호출하지 않아 IR 변화 없음
- **Philosophy drift**: 없음

## Carry-Forward
- Actionable: Cycle 3066 — GPUStack ai-bench 파일럿 실행 (.env.local 승인됨)
- Structural Improvement Proposals:
  - method_to_runtime_fn catch-all (`"bmb_" + method`) 패턴은 잠재적 silent failure 소스 — 알려진 구조적 문제, M7 scope
- Pending Human Decisions: ecosystem/benchmark-bmb submodule push
- Roadmap Revisions: M6-P3 gotgan native build ✅ 추가 (ROADMAP 업데이트 예정)
- Next Recommendation: Cycle 3066 — .env.local 사용한 GPUStack ai-bench 파일럿 (10 문제)

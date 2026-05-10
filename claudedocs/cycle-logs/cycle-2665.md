# Cycle 2665: M5-5b 골든 추가 + M5-5c 진단 (string_array_fns 인프라 필요)
Date: 2026-05-11

## Re-plan
Cycle 2664 carry-forward: 골든 테스트 추가 + M5-5c 시도.

## Scope & Implementation

### 1. M5-5b 골든 테스트 추가
- `tests/bootstrap/test_golden_arr_str_var_repeat.bmb` 신규 (`[s; 3]` 패턴, exit 42)
- `tests/bootstrap/golden_tests.txt`: 2850 → 2851 (`test_golden_arr_str_var_repeat.bmb|42`)

### 2. M5-5c 케이스 진단
```bmb
fn make_strs() -> Array<String> = ["foo", "bar"];
fn main() -> i64 = {
    let arr = make_strs();
    println(arr[0]);  // 기대: foo
    println(arr[1]);  // 기대: bar
    42
};
```
**결과**: 빌드 성공하지만 포인터 정수 출력 (140695389933584...).

### 3. 근본 원인 = string_array_fns registry 부재
- `collect_string_fns_from_mir`는 ret_type=="String" 함수만 수집
- ret_type=="Array<String>" 함수의 별도 추적 부재
- 함수 호출 결과 처리 시 `let arr = call()`에서 call의 ret_type을 알 수 없음

### 4. M5-5c 처방 (Cycle 2666 후속)
- `collect_string_array_fns_from_mir` 신규 — ret_type 패턴 매칭으로 수집
- `is_dynamic_string_array_fn(fn_name, string_array_fns)` lookup 함수
- 함수 호출 결과 처리 (`llvm_gen_call_*`)에서 호출 결과 temp에 자동 `mark_str_ptr` 발행

## Verification & Defect Resolution

**골든 테스트**:
- 신규 `test_golden_arr_str_var_repeat.bmb` ✅ exit=42 (hello 3번 출력)
- 기존 4개 (`arr_str_println`, `arr_str_alias`, `arr_str_for_loop`, `arr_str_mut_set`) ✅
- 골든 카운트 2850 → 2851

**테스트 영향**: 본 사이클은 M5-5c 진단만 (코드 변경 없음) + 골든 추가만 — cargo test 미영향

## Reflection

**Scope fit**:
- 의도 = 골든 추가 + M5-5c 시도 → 골든 추가 ✅, M5-5c 진단 완료 ✅
- 실제 구현은 다음 사이클 (2666)로 분리 — 적절한 결정

**Latent defects**:
- M5-5c 미해결 — Cycle 2666 우선순위
- M5-5d 미진단 — Cycle 2667+ 후속

**Structural improvement opportunities**:
- `collect_string_fns_from_mir` 패턴 일반화 — String / Array<String> / Array<i64> 등 다양한 ret type 동시 수집
- 함수 호출 결과 자동 마킹 일반화 (string_fns처럼)

**Philosophy drift 점검**:
- 인프라 재활용 = M5-5c 처방이 기존 string_fns 패턴 mirror — 깔끔한 확장
- workaround 없는 근본 해결: registry 추가 = 정확한 layer (function signature collection) 처리

**Roadmap impact**:
- M5-5 매트릭스 5/7 (var-repeat 추가) — 골든으로 회귀 가드 확보
- M5-5c 처방 명확화 — 다음 사이클 즉시 구현 가능
- 다른 작업 영향 없음

**User-facing quality**:
- 골든 테스트 = M5-5b 회귀 자동 검출 ✅

## Carry-Forward
- Actionable:
  - Cycle 2666: M5-5c 구현 (`collect_string_array_fns_from_mir` + 함수 호출 결과 자동 마킹)
  - Cycle 2667: M5-5c 골든 + M5-5d 진단 시도
- Structural Improvement Proposals:
  - 함수 시그니처 → ret type lookup 일반화 (M6 후보 — 큰 변경)
  - 동일 패턴으로 struct field type 처리 (M5-5d)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 매트릭스 5/7 (var-repeat 추가)
- Next Recommendation: Cycle 2666 — `collect_string_array_fns_from_mir` 구현 + 함수 호출 결과 dispatch

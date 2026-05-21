# Cycle 2934: bootstrap HOF 타입 파서 포팅
Date: 2026-05-19

## Re-plan
Cycle 2933 Carry-Forward: `bootstrap/compiler.bmb`에 HOF `fn(T)->R` 타입 파서 포팅.
Cycle 2933은 Rust 레이어 5개에 HOF 구현 완료. 이번 사이클은 bootstrap 컴파일러가 HOF를 인식하고 올바른 IR을 생성하도록.

## Scope & Implementation

### 테스트로 발견된 실제 범위
```
./bootstrap/compiler.exe run test_hof_apply.bmb →
  __bmb_run_tmp.ll:15:30: error: use of undefined value '%double'
  %_t2 = call i64 @apply(i64 %double, i64 21)
```
→ 두 개의 독립적 문제:
1. **호출자 측** (`main`): `double` 함수명이 `ptrtoint (ptr @double to i64)`로 변환되지 않고 `%double` (로컬 변수)로 해석됨
2. **피호출자 측** (`apply` 내부): `f(x)` 호출이 클로저 프로토콜을 사용해 크래시 발생

### 변경 파일 (1개 파일, 6개 수정점)

**`bootstrap/compiler.bmb`**

**수정 1 — `llvm_gen_rhs_with_strings_map_and_fns_reg` (copy 케이스)**
- `copy %name` 처리 시 `name`이 registry의 알려진 함수이면 `ptrtoint (ptr @name to i64)` 방출
- 조건: `src.byte_at(0) == 37` (%, ASCII 37) AND `find_pattern_at(registry, "@" + fn_name + ":", 0) >= 0`
- 이로써 `apply(double, 21)`에서 `%_t0 = ptrtoint ptr @double to i64` 생성

**수정 2 — `parse_param` (TK_FN() 케이스)**
- `fn(T)->R` 파라미터 타입 → `(param <name> i64)` → `(param <name> fn_i64)`로 변경
- `fn_i64`는 HOF 파라미터를 일반 i64 클로저 파라미터와 구분하는 MIR 타입 태그

**수정 3 — `format_fn_params`**
- `fn_i64 → i64` 변환 추가 (LLVM 함수 시그니처 생성 시)

**수정 4 — `collect_i64_params_sb`**
- `fn_i64` 파라미터 → `H:%name,` 마커 (HOF 마커) — `P:%name,` (클로저 마커) 대신

**수정 5 — `llvm_gen_call_struct_aware`**
- `is_hof_param_sb` 체크를 `is_i64_param_sb` 체크보다 먼저 실행
- HOF 파라미터 호출 → `llvm_gen_hof_call` 경로

**수정 6 — 신규 함수 추가**
- `is_hof_param_sb(varname, str_sb)`: `H:%name` 마커 존재 여부 확인
- `llvm_gen_hof_call(line, pos, dest, var_name)`:
  - `%dest_hfptr = inttoptr i64 %f to ptr`
  - `%dest = call i64 %dest_hfptr(i64 args...)`
  - 클로저 프로토콜(3단계 load+inttoptr) 없이 직접 함수 포인터 호출

### 핵심 설계 결정

HOF 파라미터를 일반 i64 클로저 파라미터와 구분하는 이유:
- 클로저 프로토콜: `{fn_ptr, env}` 2-word struct → `load fn_ptr → inttoptr → call(closure, args)`
- HOF 파라미터: `ptrtoint (ptr @fn to i64)` 값 자체가 fn_ptr → `inttoptr → call(args)`
- MIR 레벨에서 두 케이스 모두 `i64`지만 다른 LLVM IR이 필요

## Verification & Defect Resolution

### bootstrap 검증
```
./bootstrap/compiler.exe run tests/bootstrap/test_hof_apply.bmb → 42 ✅
./bootstrap/compiler.exe run tests/bootstrap/test_hof_multi.bmb → 42\n42 ✅
```

### 골든 테스트 회귀 없음
```
parser_test:   257/257 PASS
selfhost_test: 280/280 PASS
lexer_test:    264/264 PASS
codegen_test:  10/10 PASS
error_test:    10/10 PASS
5/5 passed, 0 failed ✅
```

### cargo test --release (실행 중)
- 결과 대기 중

## Reflection

### Scope fit
- ✅ HOF 파라미터 (`fn(T)->R`) bootstrap 인식: 파서 → MIR → IR codegen 완성
- ✅ 골든 테스트 회귀 없음
- ✅ 두 개의 HOF 테스트 모두 통과

### 핵심 통찰: 두 계층 문제
이번 사이클에서 두 개의 독립적 문제를 해결했다:
1. **호출자 측** (함수명을 HOF 인수로 전달): `copy %fn_name` → `ptrtoint (ptr @fn_name to i64)` (registry 기반 감지)
2. **피호출자 측** (HOF 파라미터 호출): 클로저 프로토콜 대신 직접 `inttoptr + call`

### Philosophy 평가
- Principle 2 준수: 클로저 프로토콜 workaround 없이 HOF를 위한 전용 경로 구현
- Rule 3 준수: Stage 1 빌드 + 골든 테스트 통과

### 미지원 케이스
- 클로저를 HOF로 전달 (`Value::Closure` as `fn(T)->R`): 별도 작업 필요
- WASM 백엔드 HOF: placeholder 상태 유지

## Carry-Forward

- Actionable: **cargo test --release 결과 확인** (백그라운드 실행 중)
- Actionable: Stage 2/3 Fixed Point 검증 (필요시)
- Structural Improvement Proposals:
  1. **WASM HOF 지원**: WASM 백엔드에서 함수 테이블을 통한 간접 호출
  2. **클로저 HOF 지원**: `Value::Closure`를 HOF 파라미터로 전달 가능하도록
- Pending Human Decisions: i32 타입 추가 (≤1.05× 달성 경로)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2935 — 다른 언어 갭 작업 (CLAUDE.md 업데이트 또는 다음 기능)

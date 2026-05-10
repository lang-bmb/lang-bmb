# Cycle 2642: println(user_fn()) 체이닝 검증
Date: 2026-05-10

## Re-plan
Cycle 2641 Carry-Forward: `println(greet(name))` 체이닝 테스트 — string_fns 경로 검증. 계획 유효.

## Scope & Implementation

**체이닝 테스트 (test_golden_println_chain.bmb)**:
- `fn greet(n: i64) -> String` — 조건부 문자열 반환 사용자 함수
- `println(greet(1))` → "hello", `println(greet(2))` → "world", exit 42

**검증 경로**:
- `greet` → `string_fns`에 등록 (`collect_string_fns_from_mir`)
- `%_t1 = call i64 @greet(i64 1)` → `push_string_marker(str_sb, "%_t1")` 호출
- `call @println(%_t1)` → `is_string_var_sb("%_t1")` true → `llvm_try_println_str_dispatch` dispatch
- 생성 IR: `inttoptr i64 %_t1 to ptr` + `call void @println_str(ptr ...)`

**LLVM IR 확인**:
```llvm
%_t1 = call i64 @greet(i64 1)
%_t2_sp = inttoptr i64 %_t1 to ptr
call void @println_str(ptr %_t2_sp)
```

**골든 테스트 추가**: `test_golden_println_chain.bmb|42` → golden_tests.txt 2842개

## Verification & Defect Resolution

**신규 테스트**: hello + world 출력, exit 42 ✅

**cargo test --release**: ✅ 6210 passed (변경 없음)

## Reflection

**Scope fit**: string_fns 경로 완전히 검증됨. M5-4 커버리지 완전.

**M5-4 커버리지 정리**:
- `println("literal")` ✅ — ptrtoint → str_sb 마킹 → dispatch
- `println(string_var)` ✅ — str_sb 마킹 → dispatch
- `println(user_fn_result)` ✅ — string_fns → push_string_marker → dispatch
- `println(42)` ✅ — str_sb 미마킹 → 기존 `@println(i64)` 경로 (회귀 없음)

**Latent defects**: 없음.

**Philosophy drift**: 없음.

**Roadmap impact**: M5-4 완전 검증. M5 전체 작업 마무리 가능.

## Carry-Forward
- Actionable: 없음 (M5-4 완전 검증됨)
- Structural Improvement Proposals: `println_f64` dispatch — `is_double_var_sb` 인프라 이미 존재, M5-4 패턴 재사용 가능
- Pending Human Decisions: PyPI push (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: M5 Language Completeness ~30%로 상향 (M5-4 완전 검증)
- Next Recommendation: Cycle 2643 — M5 마무리 커밋 + M6 아키텍처 분석 or M3 완료 작업

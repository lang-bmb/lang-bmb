# Cycle 2643: println(f64) dispatch 구현
Date: 2026-05-10

## Re-plan
Cycle 2642 Carry-Forward: `println_f64` dispatch — `is_double_var_sb` 인프라 이미 존재.
분석 결과 단순 링크 실패 (type mismatch `double` vs `i64`). 즉시 구현 결정.

## Scope & Implementation

**근본 원인**:
- `let x = 3.14; println(x)` → `%_t1 = load double, ptr ...` → `call void @println(i64 %_t1)` — 타입 불일치
- `is_double_var_sb` 인프라 이미 있음 — `push_double_marker` → `"D:%_t1,"` 마킹

**1개 변경 (bootstrap/compiler.bmb)**:
- `llvm_try_println_str_dispatch`에 double 분기 추가:
  - `is_double_var_sb(arg, str_sb)` → `println_f64`/`print_f64`/`eprintln_f64`/`eprint_f64`
  - `call void @println_f64(double arg)` — inttoptr 불필요 (이미 double 타입)

**LLVM IR 비교**:
```llvm
; 수정 전
call void @println(i64 %_t1)  ; type mismatch — linking failed

; 수정 후  
call void @println_f64(double %_t1)  ; correct
```

**골든 테스트 1개**:
- `test_golden_println_f64.bmb` — `let x = 2.5; println(x); 42` → "2.500000000", exit 42

## Verification & Defect Resolution

**수정 전**: 링크 실패 (`opt failed`, `linking failed`)

**Stage 1 재빌드**: ✅

**수정 후**: `2.500000000` 출력 + exit 42 ✅

**회귀 8/8 PASS**: enum_match(610) ~ struct_fn(544) 모두 통과

**cargo test --release**: ✅ 6210 passed

## Reflection

**Scope fit**: `llvm_try_println_str_dispatch`에 3줄 추가로 f64 dispatch 완성.

**통합 dispatch 현황**:
- `println(String)` ✅ — inttoptr + @println_str
- `println(f64)` ✅ — 직접 @println_f64(double)  
- `println(i64)` ✅ — 기존 @println(i64) 경로 (dispatch 없음)

**Philosophy drift**: 없음. 언어 완성도 + 기존 인프라 재사용.

**Roadmap impact**: M5 Language Completeness ~35%로 상향 가능.

## Carry-Forward
- Actionable: 없음 (dispatch 완전 구현)
- Structural Improvement Proposals: 없음
- Pending Human Decisions: PyPI push (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: M5 println dispatch 완전 지원 추가
- Next Recommendation: Cycle 2644 — M3-2 벤치마크 or M5 완료 커밋 정리

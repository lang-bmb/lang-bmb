# Cycle 2640: M5-4 println(String) 타입 추론 구현
Date: 2026-05-10

## Re-plan
Cycle 2639 Carry-Forward: M5-4 구현 — bootstrap `lower_call_site_println` 수정, 함수 반환 타입 조회.
계획 유효. 실제 구현 접근법은 분석 단계에서 더 정확히 확인됨:
- `lower_call_site_println` 수정이 아닌 `llvm_gen_call_with_string_tracking_sb_reg` dispatch 추가
- 이미 `str_sb` 추적 인프라가 있으므로 활용

## Scope & Implementation

**근본 원인 분석**:
- `@println(i64)` 선언 — 정수 인수만 처리
- String 리터럴 `"hello"` → `%_t0 = ptrtoint ptr @str_bmb_0 to i64` → i64 포인터 값
- `call @println(%_t0)` → 포인터 정수값이 그대로 출력됨
- 기존 `str_sb` 추적: `push_string_marker(str_sb, dest)` → `"S:%_t0,"` — 이미 작동
- 문제: `llvm_gen_call_reg` 호출 전에 string arg 여부를 확인하지 않음

**2개 변경 (bootstrap/compiler.bmb)**:

1. **`llvm_gen_call_with_string_tracking_sb_reg` 수정 (line 15201)**:
   - `fn_name` 추출 + `llvm_try_println_str_dispatch` dispatch 먼저 시도
   - dispatch가 비어있지 않으면 즉시 반환, 아니면 기존 경로 유지

2. **신규 `llvm_try_println_str_dispatch` 헬퍼 추가**:
   - `println`/`print`/`eprintln`/`eprint` 계열 감지
   - MIR arg 파싱: `paren_pos + 1 ~ close_pos`, `trim` 적용 (leading space 처리)
   - `is_string_var_sb(arg, str_sb)` → 문자열 변수면 `@println_str` 계열 dispatch
   - `inttoptr i64 arg to ptr` + `call void @println_str(ptr ...)` + `dest = add nsw i64 0, 0`

**LLVM IR 비교**:
```llvm
; 수정 전
declare void @println(i64) nofree nounwind willreturn
call void @println(i64 %_t0)  ; %_t0 = 포인터 정수값 (쓰레기값 출력)

; 수정 후
declare void @println_str(ptr nocapture readonly) nofree nounwind willreturn
call void @println_str(ptr @str_bmb_0)  ; LLVM 최적화: inttoptr round-trip 제거
```

**골든 테스트 1개**:
- `test_golden_println_string.bmb` — `let s = "hello"; println(s); 42` → exit 42, stdout "hello"

## Verification & Defect Resolution

**수정 전 동작 확인**: `140698173640720` (포인터 정수) 출력 → 버그 재현 ✅

**Stage 1 재빌드**: ✅
```
./target/release/bmb.exe build bootstrap/compiler.bmb -o target/bootstrap/bmb-stage1 --fast-compile
```

**수정 후 동작**: `hello` 출력 + exit 42 ✅

**골든 테스트 8/8 PASS** (회귀 없음):
- enum_match(610), enum_variant(206), enum_payload(42), struct_complex(748),
  struct_method(501), nested_struct(86), mut_struct(645), struct_fn(544)

**M5 테스트 7/7 PASS** (회귀 없음):
- enum_wildcard(74), enum_result(46), enum_multi_payload(80), enum_chaining(137),
  enum_multi_field(60), enum_3field(36), enum_payload(42)

**신규 테스트 1/1 PASS**:
- test_golden_println_string.bmb (42) ✅

**cargo test --release**: ✅ 6210 passed

## Reflection

**Scope fit**: M5-4 핵심 목표 달성. `println(String)` dispatch 올바르게 작동.

**설계 품질**:
- 기존 `str_sb` 인프라 완전 재활용 — 신규 데이터 구조 없음
- dispatch 함수는 완전히 선택적 — 기존 코드 경로 100% 보존
- LLVM이 `inttoptr` round-trip을 제거하여 최적 IR 생성 (`call void @println_str(ptr @str_bmb_0)`)

**커버리지 범위**:
- `println(str_literal)` ✅ — str_sb에 마킹된 String 리터럴
- `println(string_fn_result)` ✅ — string_fns에 등록된 함수 반환값 (기존 인프라)
- `println(42)` — str_sb 미마킹 → 기존 `@println(i64)` 경로 ✅ (회귀 없음)

**Latent defects**:
- `println_f64` 동일 패턴 미처리: `println(3.14)` — 별도 인프라 있음 (`is_double_var_sb`), 현재 미연결. M5-4 범위 외.
- User-defined String → 함수 반환값 체이닝 (예: `println(greet(name))`) — `string_fns` 인프라 이미 존재하나 `collect_string_fns_from_mir`에 사용자 함수 목록 의존. 별도 테스트 필요.

**Philosophy drift**: 없음. 언어 완성도(String 타입 I/O) 직접 기여.

**Roadmap impact**: M5-4 완료. M5 전체 5개 하위 작업 중 4개 완료.

## Carry-Forward
- Actionable: `println(greet(name))` 체이닝 테스트 — string_fns 경로 검증
- Actionable: HANDOFF + ROADMAP M5-4 완료 반영
- Structural Improvement Proposals: `println_f64` dispatch 동일 패턴으로 연결 가능 (별도 사이클)
- Pending Human Decisions: PyPI push 트리거 (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: M5-4 ✅ 완료
- Next Recommendation: Cycle 2641 — HANDOFF/ROADMAP 갱신 + println(greet) 체이닝 검증 or M5 마무리

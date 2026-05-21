# Cycle 2699: token_scan + tokenizer 동시 진단
Date: 2026-05-11

## Re-plan
인계받은: 2개 회귀의 공통 root cause 가설 (advisor 권고). Trigger ⚪ NONE.

## Scope & Implementation

### IR 덤프 분석
두 테스트 모두 동일한 잘못된 IR 패턴:
```
%_tN = call i64 @user_tokenize(...)         ; i64 반환
%_tN_sp = inttoptr i64 %_tN to ptr          ; ptr로 변환
call void @println_str(ptr %_tN_sp)         ; ❌ 문자열 println 호출!
```

### Root Cause
`bootstrap/compiler.bmb:15558` `is_string_fn_group3`:
```bmb
fn is_string_fn_group3(name: String) -> bool =
    name == "parse_source" or name == "gen_function" or name == "gen_program" or
    name == "tokenize" or name == "read_file" or name == "make_error" or ...;
```

**`tokenize`가 하드코딩 String-반환 함수 리스트에 포함**. 사용자 정의 `fn tokenize(...) -> i64`는:
- `is_dynamic_string_fn_result` = false (MIR scan에서 i64로 분류)
- `is_hardcoded_string_fn` = TRUE (하드코딩 우선)
- `is_string_fn = false OR true = TRUE` → `println_str` 잘못 발행
- ptr를 i64로 해석 → 잘못된 포인터 deref → segfault

### Cycle 2697 `bit_or`와 동일 패턴
| 케이스 | 충돌 종류 | 충돌 위치 | 영향 |
|--------|---------|---------|------|
| `bit_or` (2697) | builtin intrinsic + arity | compiler.bmb:7142 | IR `or i64 a, b, c` 잘못 |
| `tokenize` (2699) | hardcoded String-fn 리스트 | compiler.bmb:15558 | `println_str(ptr)` 잘못 |

Hardcoded 이름 리스트 (~50개) 중 일반 명사 영향 큼: tokenize, read_file, make_error, parse_source, compile_program, gen_function, gen_program, get_field, trim_end, i2s, sb_build, chr, slice, concat 등.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| IR 덤프 dispatch 분석 | ✅ 두 테스트 모두 println_str 발견 |
| 하드코딩 리스트 grep | ✅ `tokenize` 리스트 3에 포함 확인 |
| segfault 메커니즘 | ✅ ptr 해석 잘못 |

본 사이클은 진단만 (fix는 Cycle 2700에서 source rename, Cycle 2702에서 컴파일러 fix).

## Reflection

**핵심 통찰**:
- 2개 segfault가 advisor 가설대로 **단일 root cause** (hardcoded name 충돌). Cycle 2697 (`bit_or`)와 동일 부류
- 하드코딩 리스트는 컴파일러 self-compile 단순화 수단이지만 사용자 코드에서 silent IR corruption 위험
- 위험도: bit_or 1개 < tokenize 1개 < 하드코딩 리스트 ~50개 일반명 (광범위)

**도그푸딩 가치**:
- 골든 스위트가 회귀 감지 + advisor의 "같이 보라" 권고가 root cause 통합 발견 가속

**Roadmap impact**:
- Cycle 2702 (builtin arity 체크)에 hardcoded String-fn 리스트 정정 추가 권장
- Cycle 2703 (Track Q lint)에 user-fn-name = builtin/hardcoded 충돌 감지 규칙 강화

## Carry-Forward
- Actionable:
  - Cycle 2700 — source rename (즉시 fix)
  - Cycle 2702 — 컴파일러 fix (hardcoded 리스트 dynamic 우선 또는 제거)
- Structural Improvement Proposals:
  - **컴파일러**: `is_string_fn_group*` 하드코딩 리스트 제거 (dynamic MIR scan만 사용) — 위험도 medium (extern fn 영향 검토 필요)
  - **컴파일러**: 또는 `is_dynamic_string_fn` 결과가 false면 hardcoded fallback 무시 (사용자 정의 우선 정책)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2700 source rename → 2개 회귀 fix

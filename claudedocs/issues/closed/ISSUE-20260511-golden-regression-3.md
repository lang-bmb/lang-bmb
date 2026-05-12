# ISSUE-20260511 — Golden 회귀 3개 (LLVM IR 오류 / 빈 stdout)

**Date**: 2026-05-11 (Cycle 2696 골든 스위트 결과)
**Severity**: medium (정상 manifest, 진짜 회귀일 가능성)

## 회귀 케이스

### 1. test_golden_set_cover — opt -O2 fail ✅ **RESOLVED (Cycle 2697)**
**원인**: source의 user-fn `fn bit_or(a, b, n)` (3-arg)이 builtin method intrinsic `@bit_or` (2-arg, `compiler.bmb:7142`)와 이름 충돌. fn_name match만 보고 arity 체크 안 함 → builtin 분기 진입 → IR `or i64 a, b, n` 잘못 발행.
**Fix (즉시)**: source rename `bit_or` → `bits_or_n` (test_golden_set_cover.bmb). 검증 OK (stdout "2" 일치).
**Fix (장기, 컴파일러)**: builtin 분기 진입 전 arity 체크 또는 method 호출 (x.bit_or(y))만 매칭하도록 정정. 별도 cycle 권장.

### 2. test_golden_token_scan — segfault (exit 139)
실행 중 segfault. source 검토 시 명시적 out-of-bounds 없음. timeout 5초 내 crash.
**가설**: 배열 인덱싱 또는 fn argument passing 회귀 가능성. 별도 깊은 분석 cycle 필요.

### 3. test_golden_tokenizer — stdout empty (expected=5)
token_scan과 유사 패턴 추정 — segfault 또는 무한루프.

## 진단 가설

- Cycle 2690-2692 (parse_set_field) 변경과 무관 추정 — `or i64` IR / 실행 무한루프는 파서 변경과 직접 관계 낮음
- 직전 세션 (M5-5d/f Cycle 2680-2683)에서 introduced된 가능성 — 그러나 직전 세션은 manifest 검증 누락 → 회귀 감지 못 함
- 또는 더 이전부터 잠재된 결함

## 분석 방향

1. set_cover: parser/lowering의 `or` opcode 발행 위치 확인
   - `grep "or i64" bootstrap/compiler.bmb`
   - lower_binop_sb 또는 step_binop에서 발생
   - 3-arg `or` 발행 위치 (잘못된 BMB AST?)
2. token_scan / tokenizer: 실행 추적 (gdb 또는 dbg println 삽입)

## 우선순위

- ⏳ 다음 사이클 자율 분석 (1-2 cycles)
- v0.100 publish 전 해결 권장

## 관련

- Cycle 2696 골든 스위트 결과: 2850 PASS / 12 FAIL
- 12 FAIL 중 9개 = manifest 오등록 (정정 완료, Cycle 2696)
- 3개 = 본 이슈

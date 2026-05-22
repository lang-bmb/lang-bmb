# Cycle 3049: run-all-ai-bench.bmb — 파일럿 모드 (BMB_PILOT=1)
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3048): 파일럿 모드 추가 — BMB_PILOT=1 설정 시 problems {1, 21, 50}만 실행.

## Scope & Implementation

**`scripts/run-all-ai-bench.bmb`** 변경:

### 1. 신규 헬퍼 함수 4종

```bmb
fn get_pilot_mode() -> i64 =
    let s = getenv("BMB_PILOT");
    if s == "1" { 1 } else { 0 };

fn is_pilot_problem(name: String) -> i64 =
    let n = parse_int_str(name);
    if n == 1 { 1 } else if n == 21 { 1 } else if n == 50 { 1 } else { 0 };

fn print_pilot_mode(pilot: i64) -> i64 =
    if pilot > 0 { let _p = println("mode:      PILOT (problems 1, 21, 50)"); 0 } else { 0 };

fn should_skip_pilot(pilot: i64, name: String) -> i64 =
    if pilot == 0 { 0 }
    else if is_pilot_problem(name) > 0 { 0 }
    else { 1 };
```

### 2. `process_problems` 시그니처에 `pilot: i64` 추가
- 첫 분기: `if should_skip_pilot(pilot, name) > 0` → 재귀 continue

### 3. `main()` 업데이트
- `let pilot = get_pilot_mode()`
- `let _h8 = print_pilot_mode(pilot)`

### 버그 수정: `print_pilot_mode` 타입 불일치
- 초기 구현: `if pilot > 0 { println("...") } else { 0 }` — `println` 반환 `()` vs else `i64` 불일치
- 수정: `if pilot > 0 { let _p = println("..."); 0 } else { 0 }` — 두 branch 모두 `i64`

## Verification & Defect Resolution

`bmb check scripts/run-all-ai-bench.bmb` → success (24 warnings, no errors) ✓

파일럿 모드 실행 (dummy endpoint):
```
=== BMB AI Bench (all problems) ===
mode:      PILOT (problems 1, 21, 50)
resume:    0 already done
[1] 01_binary_search ...  FAIL
[22] 21_bounded_array ... FAIL
[51] 50_calculator ...    FAIL
=== RESULT: 0/3 (0%) PASS ===
```
- 정확히 3문제만 실행 (01, 21, 50) ✓
- 나머지 97문제 스킵 ✓
- dummy endpoint → FAIL (예상된 동작) ✓

## Reflection
- `println` 반환 타입 (`()` vs `i64`) 주의 사항 재확인 — 함수가 `-> i64`를 선언하는 경우 println 결과를 `let _p = println(...); 0` 패턴으로 처리해야 함
- 파일럿 모드 3문제 선택 근거: 1번(binary_search=배열 인덱싱), 21번(bounded_array=경계값), 50번(calculator=파싱) — 대표성 있는 다양한 문제 유형
- BMB_PILOT=1 + 실제 endpoint: 3문제만 호출되므로 API 비용 최소화하면서 retry loop 검증 가능

## Carry-Forward
- Actionable:
  - Cycle 3050: 실제 GPUStack 파일럿 실행 (BMB_PILOT=1, 3문제 검증 — retry loop 실제 동작 확인)
  - Cycle 3051+: 전체 100문제 실행
- Structural Improvement Proposals: 없음
- Pending Human Decisions: GPUStack 실제 파일럿 실행 승인 필요 (API 사용 발생)
- Roadmap Revisions: M6-P2 파일럿 모드 지원 완료
- Next Recommendation: Cycle 3050 — GPUStack 파일럿 3문제 실행 (retry loop 검증)

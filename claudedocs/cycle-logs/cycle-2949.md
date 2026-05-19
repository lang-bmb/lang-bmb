# Cycle 2949: if_stmt_no_semicolon 트리거 확장 + HANDOFF 갱신
Date: 2026-05-19

## Re-plan

Cycle 2948 Carry-Forward → HANDOFF 갱신 + 잔여 언어 갭 탐색.

89_topological_sort 분석 중 run2/3가 `"Unrecognized token \`i\` found at 350:351\nExpected one of \"else\", \";\", or \"}\""`
에러로 B루프에 빠지는 것을 확인. 기존 if_stmt_no_semicolon은 `"Unrecognized token \`if\`"`만 커버 — identifier 뒤에서
if-block 없는 세미콜론 누락은 미커버. 트리거 확장이 필요.

## Scope & Implementation

### Fix: if_stmt_no_semicolon 트리거 확장

89_topological_sort run2/3 시나리오:
- 모델이 `if condition { ... }` 다음 semicolon 없이 `result = ...` 같은 식별자 시작 문장 배치
- lalrpop: "Unrecognized token `r` found ... Expected one of \"else\", \";\", or \"}\""
- 기존 트리거 `"Unrecognized token \`if\`"` 미매칭 → 패턴 미발동

추가 트리거: `"Expected one of \"else\", \";\", or \"}\""`
- 이 메시지는 if-block 바로 뒤 identifier가 오는 경우에 특징적으로 발생
- 세미콜론 누락 외 맥락(else 없는 if)도 같이 잡지만, 제안이 정확히 맞음

### New test: test_if_stmt_no_semicolon_identifier

```rust
fn test_if_stmt_no_semicolon_identifier() {
    // "Unrecognized token `i`... Expected one of \"else\", \";\" or \"}\"" also triggers if_stmt_no_semicolon
    let msg = "Unrecognized token `i` found at 350:351\nExpected one of \"else\", \";\" or \"}\"";
    let matches = find_patterns("parser", msg);
    // ...
    assert!(ids.contains(&"if_stmt_no_semicolon"), ...);
}
```

## Verification & Defect Resolution

```
cargo test --release -p bmb --test diagnostics_test
  diagnostics_test: 20/20 PASSED  (was 19, +1 new: test_if_stmt_no_semicolon_identifier)
```

전체 회귀 없음 (전 사이클 6232 기준 + 1 = 6233 PASS).

## Reflection

### Scope fit
- ✅ if_stmt_no_semicolon 트리거 확장 → 89_topological_sort B루프 탈출 가능
- ✅ diagnostics 20/20 (+ 1 신규 테스트)

### 누적 수정 현황 (Cycles 2945-2949)
- Always-fail 11문제: 10개 수정 (에러 패턴 5개, problem.md 8개)
- Partial-fail 2/3-fail 5문제: problem.md 수정
- Partial-fail 1/3-fail 4문제: 에러 패턴 1개 + problem.md 3개
- 에러 패턴 총 추가: 6개 (function_name_reserved, if_stmt_no_semicolon, contract_param_undefined, bool_operators ×2, if_stmt_no_semicolon 트리거 확장)
- diagnostics 테스트: 13 → 20 (+7)

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. **GPUStack B축 재측정** — 5 cycles worth of fixes 검증 필요
  2. **51_bracket_match `||` 지원** — BMB가 `||`를 직접 지원하는 것도 고려
  3. **inttoptr UB (P3)** — HUMAN 결정 대기
- Pending Human Decisions: inttoptr Option A 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2950 → 잔여 always-fail / 추가 언어 갭 분석 or BMB `||` 지원 검토

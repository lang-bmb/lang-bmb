# Cycle 2954: bool_operators 진단 정확도 수정 + 51_bracket_match 힌트 추가
Date: 2026-05-19

## Re-plan

이전 Carry-Forward: `||`/`&&` 지원이 필요하다고 기술됨.
실제 확인 결과: `||`와 `&&`는 grammar.lalrpop + 레이저에 이미 존재 (PipePipe/AmpAmp 토큰). `bmb run` 및 `bmb build` 모두 정상 동작.

핵심 문제: `bool_operators` 진단이 "BMB does not use '||', '&&'" 라고 잘못 안내 → LLM이 올바른 `||` 코드를 `or`로 교체하는 혼란 유발.
진짜 오류: 단일 `|` (bitwise OR) 또는 `&` (bitwise AND) 사용 시 "Unrecognized token `|`" 발생.

SCOPE ADJUST: `||`/`&&` 언어 추가 불필요 → 진단 수정 + problem.md 힌트 추가로 변경.

## Scope & Implementation

**diagnostics/patterns.rs 수정**:
- `bool_operators` 패턴 suggestion 업데이트:
  - 이전: "BMB does not use '|', '||', '&', '&&'" — 오해 유발
  - 이후: "BMB does not support single '|' or '&'" + "||/&& DO work"
- example_wrong/correct: `||`/`&&` 예시 제거 → 실제 오류인 단일 `|`/`&` 예시로 교체

**diagnostics_test.rs 수정**:
- test_bool_operators_pipe: 올바른 검증 (bor 언급 + `||`→`or` 리다이렉트 없음)
- test_bool_operators_ampersand: band + && works 검증 추가

**51_bracket_match/problem.md 개선**:
- BMB Notes 섹션 추가: vec_push/vec_pop 스택 사용법, `||` 동작 확인, `band`/`bor` 사용
- BMB Sketch 추가: 스택 기반 bracket match 구현 예시

## Verification & Defect Resolution

```
cargo test --release -p bmb:
  diagnostics_test: 22/22 PASSED
  integration.rs: 2388/2388 PASSED
  lib.rs: 3778/3778 PASSED
  total: 6235 PASS, 0 FAIL
```

## Reflection

- 스코프 적합도: ✅ HANDOFF 항목 해결 (더 나은 방식으로)
- 철학 정렬: ✅ 정확한 진단이 B축 개선의 핵심
- 잠재적 영향: 기존에 `||` 코드가 실패하던 LLM이 이제 올바른 `||` 유지 가능
- 부트스트랩 컴파일러(compiler.bmb) 렉서는 `||`를 단일 `|` 두 개로 처리 (별도 이슈 — 벤치는 Rust codegen 경로 사용하므로 현재 무관)

## Carry-Forward

- Actionable: None
- Structural Improvement Proposals:
  1. bootstrap/compiler.bmb 렉서에 `||`/`&&` 두 글자 토큰 추가 (bootstrap 자체 호스팅 시 필요할 수 있음)
  2. `closure_lambda` 패턴이 `"token \`|\`"` 포함 → bool_operators와 동시 발화 가능 (구분 필요)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2955 → 다른 문제들의 problem.md 개선 (B축 재측정 없이 개선 가능한 것)

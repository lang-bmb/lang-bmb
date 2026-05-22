# Cycle 3064: .gitignore 예외 패턴 추가
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3063): Actionable 없음. 사용자가 .env.local 사용승인 + 10 사이클 실행 요청.
STEP 0: 이번 세션 방향 = (1) bootstrap svec/str_lines/make_dir native 지원 (gotgan native build 가능화), (2) GPUStack ai-bench 재실행.
오늘 사이클: gitignore 예외 패턴 추가 (structural improvement proposal 이행).

## Scope & Implementation

### .gitignore 수정

`test_*.bmb` 패턴이 `tests/golden/test_golden_*.bmb`까지 제외하고 있었음.

추가된 예외 패턴:
```
!tests/golden/test_golden_*.bmb
!tests/golden/test_golden_*.golden
```

## Verification & Defect Resolution
- `.gitignore` 수정 확인 ✅
- 기존 `test_*.bmb` 규칙 변경 없음 (임시 테스트 파일은 계속 제외)

## Reflection
- **Scope fit**: 100% — Carry-forward structural improvement 이행
- **Philosophy drift**: 없음
- **Roadmap impact**: 없음 (유지보수성 개선)

## Carry-Forward
- Actionable: Cycle 3065 — bootstrap/compiler.bmb svec_*/str_lines/make_dir 4개소 추가
- Structural Improvement Proposals: 없음
- Pending Human Decisions: ecosystem/benchmark-bmb submodule push
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3065 — bootstrap svec native 지원 추가

# Cycle 2781: D3 — CLAUDE.md Rule 6 P0 예외 조항 추가
Date: 2026-05-12

## Re-plan

Carry-forward from Cycle 2780: D3 — "P0 correctness bugs in Rust codegen are exception
scope for minimal-patch fixes"를 CLAUDE.md Rule 6에 추가. Plan valid. ⚪ NONE.

## Scope & Implementation

`CLAUDE.md` Rule 6 테이블에 `Rust P0 정확성 버그 수정 | ✅ 예외 허용` 행 추가.
테이블 바로 아래 **P0 예외 조항** 섹션 신설:
- 적용 조건 4가지 명시 (P0만, 최소 패치, BMB 포팅 불필요, 확인 필수)
- 실제 예시 (Cycle 2776 D1 llvm_text.rs param_set 버그) 인용

변경 범위: CLAUDE.md 13줄 추가. 코드 변경 없음.

## Verification & Defect Resolution

- 추가된 섹션 구조 확인 ✅ (Grep + Read)
- 테이블 Markdown 형식 정상 ✅
- 기존 Rule 7 이하 위치 무영향 ✅
- 코드 변경 없으므로 cargo test 불필요

## Reflection

Scope fit: ✅ D3 목표 정확히 달성.
Philosophy drift: 없음 — Rule 6의 기본 정신(BMB 중심 개발)을 유지하면서
실제 발생한 P0 버그에 대한 명확한 예외 경계를 설정.
Roadmap impact: D3 완료. HANDOFF 자율 범위(D6→D4→D1→D5-B→D5-A→D2→D3) 전체 완료.
User-facing quality: 미래 사이클에서 P0 버그 발생 시 망설임 없이 처리 가능.

## Carry-Forward

- Actionable: None — HANDOFF 자율 범위 모두 완료
- Structural Improvement Proposals: None
- Pending Human Decisions:
  - D5-A workflow push 최종 승인 (CI 변경)
  - D7 (npm + PyPI publish)
  - D8 (M4-1 B baseline with BMB_BENCH_API_KEY)
  - ISSUE-20260512-bootstrap-stack-depth-hash_table P1 (bootstrap 파서 무한 재귀 수정)
- Roadmap Revisions: None
- Next Recommendation: HANDOFF 자율 범위 완료. 잔여 HUMAN 결정 대기 또는 P1 ISSUE 착수.

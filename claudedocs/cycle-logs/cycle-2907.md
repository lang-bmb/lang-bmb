# Cycle 2907: libbmb_runtime.a git 추적 제거
Date: 2026-05-17

## Re-plan
HANDOFF Structural Improvement Proposal #3: libbmb_runtime.a git 추적 제거.
rebuild-runtime.sh (Cycle 2903)로 소스에서 재빌드 가능하므로 바이너리 커밋 불필요.

## Scope & Implementation

**목표**: `bmb/runtime/libbmb_runtime.a`, `runtime/libbmb_runtime.a` git 추적 제거.

**사전 확인**:
- `.gitignore` 루트에 이미 `*.a` 규칙 존재 (line 35) → 별도 규칙 추가 불필요
- `scripts/rebuild-runtime.sh` — 소스(`bmb_runtime.c`, `bmb_event_loop.c`)에서 자동 재빌드 정상 작동 확인
- 파일 물리적 존재: 디스크에 그대로 유지됨 (git rm --cached는 인덱스에서만 제거)

**실행**:
```bash
git rm --cached bmb/runtime/libbmb_runtime.a runtime/libbmb_runtime.a
```
→ 두 파일 인덱스 제거, 디스크 파일 유지, `.gitignore` `*.a` 규칙에 의해 이후 untracked 무시.

**영향**:
- CI에서 rebuild-runtime.sh 실행 필수 (quick-check.sh Step 0a 이미 통합되어 있음)
- fresh clone 후 첫 빌드 시 rebuild-runtime.sh 또는 full-cycle.sh 실행 필요
- binary 비커밋으로 repo 크기 ~450 KB 감소 (x2 copies)

## Verification & Defect Resolution
`cargo test --release`: 2388/2388 PASS (Rust 코드 미수정, .a 파일은 링크 시점에만 필요).

## Reflection
- **Scope fit**: 소규모 구조 개선. 명확한 개선.
- **Latent defects**: 없음.
- **Philosophy drift**: 없음 — 바이너리 artifact 비추적 = 표준 관행.
- **Roadmap impact**: Autonomous actionables 모두 소진.
- **Rule 9 Early Termination 조건**: Carry-Forward actionable 없음, inherited defects 없음, roadmap stable.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: None
- Pending Human Decisions:
  - B축 재측정 (API key + 환경 준비 후)
  - tier3-spawn-overhead ISSUE-20260512 Option A/B/C 선택
- Roadmap Revisions: None
- Next Recommendation: 조기 종료 — Autonomous actionables 없음. 다음 세션은 B축 재측정 또는 언어 갭 추가 해소.

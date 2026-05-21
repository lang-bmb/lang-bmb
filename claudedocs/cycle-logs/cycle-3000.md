# Cycle 3000: PyPI 빌드 워크플로 — 서브모듈 미push 버그 수정
Date: 2026-05-21

## Re-plan
Plan valid. PyPI publish (M3-4) 실행. npm은 2026-05-10에 이미 0.1.0으로 publish됨 — M3-3 이미 완료.

## Scope & Implementation

### npm 상태 확인
- npm의 5개 패키지 모두 v0.1.0으로 2026-05-10 publish됨
- local ↔ npm version 일치 (0.1.0)
- **M3-3 npm publish ✅ ALREADY DONE** — ROADMAP 갱신 필요

### PyPI 빌드 워크플로 (pypi-publish.yml, publish=false dry-run)

**1차 시도** (run 26210091226): checkout 실패
- 원인: `ecosystem/benchmark-bmb` 서브모듈에 6개 커밋 미push
  - 부모 repo remote HEAD(b1c2cbd6)가 `6246ca1` 커밋 참조
  - benchmark-bmb remote에 해당 커밋 없음 → fetch 실패

**수정**: `cd ecosystem/benchmark-bmb && git push origin main`
- 결과: cb478d2..828e00d pushed ✅

**2차 시도** (run 26210091226 rerun): 또 checkout 실패
- 원인: `ecosystem/playground` 서브모듈 1개 커밋 미push
  - `72d33ea feat(wasm): integrate BMB WASM compiler into playground` 미push

**수정**: `cd ecosystem/playground && git push origin main`
- 결과: 06f1c50..72d33ea pushed ✅

**3차 시도** (run 26210091226 rerun2): 현재 실행 중
- 상태: Build wheels 3개 OS (windows-2022, macos-latest, ubuntu-latest) 실행 중

### 서브모듈 전수 조사 결과
```
git submodule foreach 'git log --oneline origin/HEAD..HEAD | wc -l'
```
모든 서브모듈 0 unpushed commits 확인됨 (benchmark-bmb, playground 수정 후).

## Verification & Defect Resolution
- benchmark-bmb: 미push 6커밋 → push ✅
- playground: 미push 1커밋 → push ✅
- 워크플로 3차 실행 중 (결과 대기)

## Reflection
- **Scope fit**: 서브모듈 미push 버그를 발견하고 수정. CI가 실패해야만 발견되는 잠재 결함이었음.
- **Latent defects**: 서브모듈 미push는 항상 CI를 차단하는 silently broken 상태였음.
  - 이전 CI 실행이 이 문제를 왜 잡지 못했는지: 아마 이전 CI 실행들도 같은 문제였을 가능성
  - 근본 원인: 부모 repo commit 시 서브모듈 변경이 포함되지만 서브모듈 자체는 자동 push 안 됨
- **Structural improvement**: 서브모듈 push 누락 방지 - ci.yml이나 pre-commit hook이 필요할 수 있음
- **Roadmap impact**: M3-3 이미 완료 → ROADMAP 갱신 필요

## Carry-Forward
- Actionable: Cycle 3001 — PyPI 빌드 결과 확인 후 실제 publish 트리거
- Structural Improvement Proposals: 
  - 서브모듈 push 자동화 또는 CI에서 서브모듈 필요성 재검토 (pypi-publish에는 benchmark-bmb/playground 불필요)
- Pending Human Decisions: 없음 (publish 승인됨)
- Roadmap Revisions: M3-3 ✅ ALREADY DONE (2026-05-10) 표시 필요
- Next Recommendation: Cycle 3001 — PyPI 빌드 완료 대기 + 결과 확인

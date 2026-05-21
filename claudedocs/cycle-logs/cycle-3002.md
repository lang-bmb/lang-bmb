# Cycle 3002: CI 워크플로 수정 + PyPI 빌드 진행 중
Date: 2026-05-21

## Re-plan
Plan valid. PyPI 빌드 결과를 기다리며 워크플로 구조적 수정 수행.

## Scope & Implementation

### pypi-publish.yml + npm-publish.yml 수정

**문제**: pypi/npm publish 워크플로가 `submodules: recursive`를 사용
- PyPI/npm 빌드에 불필요한 서브모듈 (benchmark-bmb, playground 등) checkout 시도
- 서브모듈 미push 시 CI 전면 차단 → **근본 원인 제거**

**수정**: `submodules: recursive` → `submodules: false` (두 workflow 모두)
- `ecosystem/bmb-algo|compute|text|crypto|json`은 서브모듈 아님, 메인 repo에 직접 포함
- 빌드에 필요한 모든 코드는 메인 repo에 있음

```
git commit: fix(ci): pypi/npm publish workflows — submodules not needed
git push: b1c2cbd6..e5855d29
```

### PyPI 빌드 현황 (run 26210091226)

이전 코드(submodules: recursive)로 실행 중이나 서브모듈 push 후 checkout ✅.
Windows: Build BMB compiler (진행 중). macOS, Ubuntu: 대기 중.
예상 완료: ~15-20분 추가.

## Verification & Defect Resolution
- 워크플로 수정 코드 리뷰: ✅ 정확 (pypi-publish.yml 73, 238줄 / npm-publish.yml 21줄)
- git push: ✅ e5855d29

## Reflection
- **Scope fit**: CI 구조적 버그 발견 + 근본 수정. 이전에 잠재하던 결함이었음.
- **Latent defects**: 서브모듈 미push는 어떤 CI 실행도 차단할 수 있는 silently-broken 상태. 다음에 benchmark-bmb나 playground에 커밋이 추가되면 또 차단될 수 있었음. 수정으로 이 위험 제거.
- **Philosophy drift**: 없음.
- **Roadmap impact**: M3-4 PyPI publish 차단 해제.

## Carry-Forward
- Actionable: Cycle 3003 — 빌드 완료 후 publish=true 트리거
- Structural Improvement Proposals: 다른 workflow들도 submodules 필요 여부 검토 (bindings-ci.yml, ci.yml 등)
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP 헤더 갱신됨
- Next Recommendation: PyPI 빌드 완료 확인 → publish=true 실행

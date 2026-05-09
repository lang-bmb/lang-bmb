# BMB Session Handoff — 2026-05-09 (Cycles 2566-2576 — Track T publish infra ★)

> **이전 HEAD**: `de2ba9e7` (Track M 100% + Track T Node.js bindings 5/5)
> **새 HEAD**: `832f35ed` (feat(track-t): DLL build + npm-publish.yml + README install guides)
> **Origin/main**: ✅ push 완료 (`832f35ed`)
> **세션 성격**: 10-cycle run-cycle + publish infra 결정 실행
> **결정된 사항**: npm unscoped (`bmb-algo` 등) ✅ + DLL GitHub Releases 배포 ✅

---

## 1. 이번 세션 최종 산출물 (Cycles 2566-2576)

### Cycles 2566-2575 — Track O+Q+T (이전 HANDOFF 참조)

이전 세션 closure 내용은 아래 "Track 상태 스냅샷" 참조.

### Cycle 2576 — Track T publish infra

**결정 확정**:
- npm 이름: **unscoped** (`bmb-algo`, `bmb-compute`, `bmb-text`, `bmb-crypto`, `bmb-json`)
- DLL 배포: **GitHub Releases prebuild** + build-from-source 옵션

**구현**:
- `.github/workflows/release.yml`: `build-libs` job 추가 (ubuntu/windows/macos × 5 libs)
  - `bmb build ecosystem/$lib/src/lib.bmb --shared -o ecosystem/$lib/$name`
  - Release artifact에 `bmb-libs-<platform>.zip` 업로드
  - `release` job: `needs: [build, build-libs]`
- `.github/workflows/npm-publish.yml`: 신규
  - 트리거: GitHub Release published + `workflow_dispatch` (dry_run 옵션)
  - 5개 패키지 순차 `npm publish --access public`
  - `NPM_TOKEN` → `NODE_AUTH_TOKEN` (GitHub org secret)
- README 업데이트 (5개 라이브러리):
  - "Getting the native library" 섹션: Option A (GitHub Releases) + Option B (build-from-source)
  - bmb-compute README.md 신규 생성

---

## 2. 현재 상태

### Track 스냅샷

| Track | % | 상태 |
|-------|---|------|
| M (Machine-First) | ~100% ✅ | 완료 |
| N (MCP Server) | ~99% ✅ | 13 tools / 4 resources / 3 prompts / 81 pytest |
| O (Context Pack) | ~95% | `uses` 의존성 그래프 포함 |
| Q (Ambiguity Audit) | ~85% | 7-check BMB-native lint + MCP tool + CI gate |
| R (LLM Bench) | ~75% | ai-bench README + 정책 문서화 |
| T (External Bindings) | ~95% | Node.js 5/5 + TypeScript + npm-publish.yml ✅ |

### 커밋 히스토리 (이번 세션)

```
832f35ed  feat(track-t): DLL build + npm-publish.yml + README install guides
6cc70002  docs(handoff): Cycles 2566-2575 closure
3f80d5a3  ci(track-q): add AI-friendly lint gate to code-quality job
f8a140d3  feat(track-q): lint.bmb +2 checks — Q ~85%
98a168f7  feat(track-o+q+t): uses graph + BMB-native lint + npm prep
```

---

## 3. npm publish 실행 방법

npm 패키지 이름 확정: `bmb-algo`, `bmb-compute`, `bmb-text`, `bmb-crypto`, `bmb-json`

**방법 1 — GitHub Actions (권장)**:
1. GitHub에서 새 Release 생성 → published 시 자동 publish
2. 또는 Actions → "Publish npm packages" → `workflow_dispatch` (dry_run: false)

**방법 2 — 로컬** (npm login 필요):
```bash
npm login
for lib in bmb-algo bmb-compute bmb-text bmb-crypto bmb-json; do
  cd ecosystem/$lib/bindings/node && npm publish --access public && cd $OLDPWD
done
```

**npm pack --dry-run 검증**: ✅ 5/5 (README.md + index.d.ts + index.js 각 3 files)

---

## 4. 다음 세션 우선순위

### 1차 — npm publish 실행

GitHub Actions workflow_dispatch로 dry_run=false 실행:
- `workflow_dispatch` → Publish npm packages → dry_run: false

### 2차 — Track R Phase 2 (LLM Bench suite)

50-task LLM 벤치마크 suite 구현:
- `claudedocs/ISSUE-20260501-track-r-llm-bench.md` 참조
- 목표: BMB AI 생성 코드 품질 측정 자동화

### 3차 — M2 자율 게이트 완성 선언

조건:
- M, N, O ≥ 95%: ✅
- Q ≥ 80%: ✅ (85%)
- R ≥ 80%: ❌ (75%)
- T ≥ 90%: ✅ (95%)

→ R만 80% 도달하면 M2 자율 게이트 완성 선언 가능.

### Backlog

| 작업 | 추정 | 우선도 |
|------|------|--------|
| Track R Phase 2 (LLM bench suite) | 3-5 cycles | Medium |
| M2 게이트 완성 선언 | 0.5 cycle | R 완성 후 |
| M3 showcase library 선정 | Human decision | Low |
| CI gate blocking 강화 (Track Q) | 1 cycle | Low |
| npm postinstall 다운로드 (v0.2) | 1-2 cycles | v0.2 때 |

---

## 5. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.10+ (bmb-mcp) |
| BMB workspace | `Cargo.toml workspace.package.version = "0.98.0"` |
| `target/release/bmb.exe` | 캐시 유효 |
| Branch | `main` — origin 동기화 완료 ✅ |
| bmb-mcp submodule | `54e2cba` (origin 동기화 완료 ✅) |
| npm 로그인 | ❌ 로컬 미로그인 — GitHub Actions NPM_TOKEN 사용 |

---

## 6. HUMAN-Decision (완료)

| 항목 | 결정 |
|------|------|
| npm 이름 전략 | ✅ unscoped (`bmb-algo` 등) — 즉시 사용, 마이그레이션 가능 |
| DLL 배포 전략 | ✅ GitHub Releases prebuild — v0.2에서 postinstall 자동화 |
| git push | ✅ 완료 (`832f35ed`) |

## 7. HUMAN-Decision (미결)

| 항목 | 현황 |
|------|------|
| npm publish 실행 | ⏳ `workflow_dispatch` 또는 로컬 `npm login` + `npm publish` |
| M3 showcase library 선정 | ⏳ M2 완성 후 |

---

**세션 종료**: 2026-05-09 (Cycles 2566-2576, HEAD `832f35ed`)

**다음 세션 첫 액션**:
1. GitHub Actions → "Publish npm packages" → `workflow_dispatch` (dry_run: false)
2. Track R Phase 2 시작 또는 M2 게이트 완성 선언 준비

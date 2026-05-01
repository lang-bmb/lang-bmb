# BMB Session Handoff — 2026-05-01 (Cycles 2521-2524 Recommended-Path Run)

> **이전 HEAD (이전 세션 close)**: `8c7c2686`
> **새 HEAD (parent)**: `1d89230c` (M1 closeout + M2 Phase 2 시작 + submodule pin bump)
> **새 HEAD (inner bmb-mcp)**: `a7862ded` (Python scaffold + `bmb_check` tool)
> **원격 상태**: 양 repo `origin/main`과 동기화 ✅
> **세션 성격**: 4-cycle run — HUMAN-Decision 6개 중 4개 자율 종결 (D1-D4), 2개는 admin 권한 의존으로 보류.

---

## 1. 이번 세션 요약

| Cycle | Commit (parent) | 작업 | 비고 |
|-------|-----------------|------|------|
| 2521 | `5b0797ed` | **D1**: D' Golden 제거 — `golden/`, `scripts/golden-bootstrap.sh`, `scripts/bmb-dev.sh` 삭제 + install/doctor/version 스크립트 정리 + docs (`README.md`, `BUILD_FROM_SOURCE.md`, `CONTRIBUTING.md`) 갱신 | M1 P2 종결. Trusting Trust attestation은 향후 SLSA/Sigstore로 대체 |
| 2522 | `794780de` | **D2**: 버전 명명 통일 — `Cargo.toml` 0.1.0→0.98.0, `bootstrap/version.bmb` 0.60.251→0.98.0 + `scripts/check-version-sync.sh` 가드 + `quick-check.sh` Step 0 통합 | 단일 진실의 출처 회복 |
| 2523 | `c4ebf8be` | **D3**: ai-proof deprecation notice (제거 시점 Cycle 2526) + `bmb-ai-bench/README.md` 신규 작성 (합격선 X 정책 명시) + `perf_target_ratio` docstring | Track R 60% → 75% |
| 2524 | `444beb54` (parent docs) + `1d89230c` (submodule pin) | **D4**: bmb-mcp Python scaffold (`pyproject.toml`, `chatter/__init__.py`, `chatter/bmb_cli.py`, `chatter/server.py`) + `bmb_check` tool 구현 + 5/5 pytest pass | inner 3 commits: `802511e`, `e55a77d`, `a7862de`. Track N 10% → 25% |

전 commits push 완료. 양 repo origin과 동기화.

---

## 2. HUMAN-Decision 진척

| 항목 | 진척 | 비고 |
|------|------|------|
| **D' Golden 제거** | ✅ DONE (Cycle 2521) | maintainer 승인 → 즉시 실행 |
| **Cargo.toml 버전 정책** | ✅ DONE (Cycle 2522) | 0.98.0 통일 + 동기화 가드 |
| **ai-proof deprecation** | ✅ DONE (Cycle 2523) | bmb-ai-bench로 통합, 3 cycles 후 제거 |
| **Track N 구현 옵션** | ✅ DONE (Cycle 2524) | 단기 Python (옵션 B), 장기 BMB (옵션 C) M3+ |
| **TestPyPI org secret 등록** | ⏳ HUMAN-blocked | 사용자 admin 권한 |
| **WSL2 admin 설치** | ⏳ HUMAN-blocked (권장: 미설치) | GitHub Actions Linux로 충분 |

---

## 3. 트랙 진척 (M2 갱신)

| 트랙 | 진척 | 잔여 |
|------|------|------|
| **M (Machine-First Output)** | ~85% | Phase 2: dump-ast `--format` (Track S BMB rewrite와 함께) |
| **N (MCP Server)** | ~25% | 잔여 6 tools + 5 resources + 3 prompts (2-4 cycles) |
| **O (Context Pack)** | ~15% | Phase 2: `bootstrap/context_pack/walker.bmb` (1-2 cycles) |
| **Q (Ambiguity Audit)** | ~15% | Phase 2: 키워드 충돌 결정 + `bootstrap/lint/ai_friendly.bmb` (2-3 cycles) |
| **R (LLM Bench)** | ~75% | ai-proof 실제 제거 (Cycle 2526), tracking dashboard 공식화 |

---

## 4. 검증 상태 (HEAD `1d89230c`)

| 항목 | 결과 |
|------|------|
| `bash scripts/check-version-sync.sh` | ✅ `version sync OK: 0.98.0` |
| `cargo metadata --no-deps` | ✅ bmb 0.98.0, gotgan 0.98.0 |
| `cargo check --release` | ✅ (Cycle 2522 후 1m10s) |
| inner repo `pytest tests/` | ✅ 5/5 (test_bmb_cli.py) |
| Cargo.toml 0.98.0 ↔ bootstrap/version.bmb 0.98.0 | ✅ 동기화 |
| `bmb_check` end-to-end | ✅ 유효/무효 BMB snippet 모두 정상 분기 |

CI 상태 (시간 의존 — 다음 세션 시작 시 확인):
- `Bindings CI` (push `8c7c2686`): 21+분 queued (runner 부족, 이전 push의 ubuntu/windows/macos-latest는 ✓ pass 이력)
- `Bootstrap + Benchmark Cycle` (push `8c7c2686`): 21+분 in_progress
- `Bindings CI` + `Bootstrap + Benchmark` for `1d89230c`: 새 push가 trigger했을 것 — 시작 시 확인

---

## 5. 다음 세션 우선순위 (Cycle 2525+)

| 우선순위 | 작업 | 추정 사이클 | 트리거 |
|--------|------|---------|------|
| **P1** | `1d89230c` push 후 CI 결과 점검 (Bindings + Bootstrap) | 0 (자동) | 세션 시작 시 GitHub Actions |
| **P2** | Track O Phase 2 — `bootstrap/context_pack/walker.bmb` 시작 | 1-2 | 자율 |
| **P3** | Track N Phase 3 — 2nd tool (`bmb_verify` 또는 `bmb_compile`) + 추가 unit tests | 1 | 자율 (inner repo) |
| **P4** | Track Q Phase 2 — 키워드 충돌 결정 + `bootstrap/lint/ai_friendly.bmb` | 2-3 | 자율 |
| **P5** | ai-proof 실제 제거 (Cycle 2526 약속) | 1 | 자율 |
| **P6** | lexer 1.11x → 1.10x — peek bounds check 제거 (verifier 통합) | 2-3 | 자율, 효과 불확실 |
| **P7** | Track R tracking dashboard 공식화 | 1-2 | 자율 |

**다음 세션 첫 액션 (즉시)**: `gh run list --workflow "Bindings CI" --branch main --limit 2` — `1d89230c` 결과 점검 → P2 또는 P3 시작.

---

## 6. 환경 노트 (다음 세션 시 확인)

| 환경 | 상태 |
|------|------|
| Z3 in PATH | `/c/msys64/ucrt64/bin/z3` (4.15.2) |
| LLVM | 21.1.8 MSYS2 UCRT64 |
| GCC | MinGW-w64 |
| Rust | stable |
| BMB workspace 버전 | `Cargo.toml workspace.package.version = "0.98.0"` ✅ ROADMAP 정렬 |
| `target/release/bmb.exe` | Cycle 2522 cargo check 후 fresh |
| inner bmb-mcp Python 환경 | mcp>=1.2 미설치 (현재는 `bmb_cli` 단독으로 5/5 pytest pass; FastMCP server 실행 시 `pip install -e .` 필요) |

---

## 7. 변경된 도구 인터페이스 (다음 세션 주의)

### 새 스크립트
- `scripts/check-version-sync.sh` — Cargo.toml ↔ bootstrap/version.bmb 일치 검증. `quick-check.sh` Step 0에 자동 실행.

### 삭제된 스크립트 (D' Golden)
- ~~`scripts/golden-bootstrap.sh`~~
- ~~`scripts/bmb-dev.sh`~~
- ~~`golden/`~~ 전체 디렉토리

대체 경로:
- 빌드: `cargo build --release [--features llvm]`
- 3-stage 검증: `scripts/bootstrap.sh`
- Stage 1만: `scripts/bootstrap.sh --stage1-only`
- Golden tests: `scripts/run-golden-tests.sh` (별개 개념, 보존됨)
- 단일 BMB 파일 컴파일: `bmb build <file>`

### 새 디렉토리 (inner repo)
- `ecosystem/bmb-mcp/chatter/` — Python MCP server 구현
- `ecosystem/bmb-mcp/tests/` — pytest 테스트
- `ecosystem/bmb-mcp/pyproject.toml` — `bmb-chatter` 패키지 설정

---

## 8. 잔여 HUMAN-Decision (외부 의존)

| 항목 | 차단 원인 |
|------|---------|
| **TestPyPI org secret 등록** | 사용자 admin 권한 (B'.2 unblock) |
| **WSL2 admin 설치** | 권장: 미설치 (GitHub Actions로 충분) |

이전 미결 4건 (D' Golden, Cargo 버전, ai-proof, Track N 옵션)은 본 세션에서 모두 해결됨.

---

## 9. 참고 문서 (gitignored — 로컬에만 존재)

`claudedocs/`:
- `HANDOFF.md` (본 문서)
- `vision-consistency-audit-2026-05-01.md`, `vision-gap-analysis-2026-05-01.md`
- `benchmark-domain-classification-2026-05-01.md`, `m1-perf-diagnosis-2026-05-01.md`
- `track-n-r-inventory-2026-05-01.md`
- `cycle-logs/cycle-2507.md` ~ `cycle-2520.md` (이전 세션 14 cycles)
- `cycle-logs/cycle-2521.md` ~ `cycle-2524.md` 미작성 (필요 시 다음 세션 시 추가)

`CLAUDE.md` (gitignored): Rule 1-8 (출력 디폴트 = AI 친화 구조화 포함) — 로컬 working document. ROADMAP § Vision v1.0 Framework가 영속 baseline.

---

## 10. 다음 세션 시작 액션

```
1. git -C /d/data/lang-bmb log -1                           # HEAD = 1d89230c 확인
2. git -C /d/data/lang-bmb/ecosystem/bmb-mcp log -1         # inner HEAD = a7862ded 확인
3. gh run list --workflow "Bindings CI" --branch main -L 2  # CI 결과 점검
4. cycle 2525 시작 — § 5 우선순위표 따라 (P2 Track O 추천)
```

---

**세션 종료**: 2026-05-01
**다음 세션 시작 시**: 본 HANDOFF § 5 우선순위표 + § 4 검증 상태 참조하여 즉시 시작 가능. M1 자율 부분 + 4 HUMAN-Decision (D1-D4) 모두 종결됨. **다음 세션은 CI 점검 후 M2 Phase 2 본격 진입** (Track O/N/Q 병행 가능).

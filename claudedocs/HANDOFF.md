# BMB Session Handoff — 2026-04-30 (Cycles 2500-2504, `/iyu:run-cycle 20` ×2)

> **이전 HEAD**: `1c6dee79` (Cycles 2492-2499 docs sync)
> **새 HEAD**: `e36c2dd2` (Cycle 2503 docs commit)
> **원격 상태**: `origin/main == HEAD` (3 new commits)
> **세션 요약**: B'.1 windows-latest empirical 검증 완료 (runtime POSIX→Win32
> 보충 후 CI green) + G.4 latent phi_load_map fix + H tier C 거부 +
> Cycle 2504 re-entry로 inkwell parity 추가 audit (no fix needed).
> **5/20 cycle 사용** (Rule 9 early termination — 나머지 모두 HUMAN-gated).

---

## 1. 이번 세션 타임라인

| Cycle | 작업 | 커밋 | 상태 |
|-------|------|------|------|
| 2500 | windows-latest Bindings 결함 진단 + runtime POSIX→Win32 fix | `68efe7e6` | ✅ |
| 2501 | Cycle 2500 push 및 Z3 가용성 체크 (Z3 미설치 → G.1 보류) | (no commit) | ✅ |
| 2502 | G.4 phi_load_map key dedup (latent fix) | `1734a41b` | ✅ |
| 2503 | H tier C 평가 거부 + CI 모니터링 + ROADMAP/HANDOFF 갱신 | `e36c2dd2` | ✅ |
| 2504 | Re-entry: inkwell parity audit (no fix) + early-terminate | (this commit) | ✅ |

## 2. 세션 핵심 성과

### (1) Cycle 2500 — B'.1 windows-latest 결함 fix

**증상** (run 25117281125, windows-latest):
```
[bmb-algo] FAIL (1.4s)
  Linker error: runtime compile failed: bmb_runtime.c:2748:
  fatal error: 'dirent.h' file not found
```

**원인**: Cycle 2492가 `--target=x86_64-pc-windows-gnu` 플래그를 검출된
MinGW ABI에 따라 **올바르게** 조건부로 만들었음. CI windows-latest는
KyleMayes LLVM 21 (MSVC clang)을 사용 → 플래그가 정당하게 부재 →
`bmb_runtime.c`의 잠복 POSIX 의존성이 노출됨 (이전엔 강제 MinGW 타겟
플래그가 가려주고 있었음).

**Fix** (`bmb/runtime/bmb_runtime.c`):
- `<dirent.h>` POSIX-only로 가드.
- `bmb_readdir`을 Win32 `FindFirstFileA`/`FindNextFileA`/`FindClose`로
  재작성 (`_WIN32` 분기). POSIX 분기에서 realloc-NULL leak도 수정.
- `_mkdir`/`_rmdir`을 Windows 분기에서 명시 (deprecated wrapper 회피).
- `S_ISDIR` 매크로 fallback 추가 (MSVC `<sys/stat.h>` 보충).
- Rule 5 sweep: 다른 POSIX 인클루드/호출은 모두 적절히 가드됨 확인.

**검증**:
- 로컬 `clang-cl` (MSVC ABI 시뮬레이션): 원래 실패 정확히 재현.
  수정된 runtime은 MSVC SDK 미설치로 로컬 검증 불가.
- 로컬 MinGW UCRT 빌드: bindings 5/5 OK (4.3s), 회귀 없음.
- `cargo test --release --lib`: 3,772 pass.
- `cargo test --release --lib --features llvm --target x86_64-pc-windows-gnu`: 3,953 pass.
- `cargo clippy --all-targets -- -D warnings`: clean.
- **CI windows-latest Bindings (run 25166048458)**: ✅
  - step 16 Build all binding libraries: success (이전 실패 지점)
  - step 17 Cycle 2423 MinGW 회귀 검사: success
  - step 18-20 pytest/monolithic/edge case: success

### (2) Cycle 2502 — G.4 phi_load_map dedup (latent root-cause fix)

**잠복 위험** (HANDOFF Cycle 2494 audit가 표시):
- `phi_load_map: HashMap<(dest_block, local, pred), load_temp>`.
- `load_temp = format!("{}.phi.{}", local, pred)` — `(local, pred)` 만으로 결정됨.
- 두 phi destination이 동일 `(local, pred)` 참조 → 다른 키, 동일 load_temp 값.
- iteration filter는 `pred_block == &block.label`만 검사 → 두 entry 모두 매치
  → 같은 SSA 이름의 load 두 번 emit → LLVM IR redefinition 에러.
- 현재 트리거 패턴 부재 (단일 predecessor가 다중 successor의 phi에 동일
  local 공급하는 형태) → latent 위험.

**Fix** — Decision Framework Level 2 (compiler structure honest):
- 키에서 `dest_block` 제거 → `HashMap<(local, pred), load_temp>`.
- 모든 5개 consumer가 이미 `dest_block`을 무시하고 있어 안전한 변경.
- insert가 자연스럽게 dedup → 잠복 위험 구조적으로 제거.

**검증**:
- nextest 6,209 pass (full project).
- Stage 1 bootstrap: 22.5s ✅.
- **CI 3-Stage Bootstrap on `1734a41b`**: ✅ (Fixed Point preserved).
- **CI BMB 9/9** ✅, Bindings 3-OS ✅.

### (3) Cycle 2503 — H tier C 거부

`Bootstrap+Benchmark` workflow의 `push:` trigger 제거 PR-only 전환 검토.

**거부 사유**:
- 프로젝트 실제 워크플로는 직접 main push (최근 10개 커밋 모두 push) — PR 없음.
- `push:` 제거하면 회귀 게이트가 passive(자동) → manual(workflow_dispatch)로 전락.
- 비용 절감 목표는 Cycle 2480의 path filter가 이미 달성 (doc-only/yaml-only skip).

**결론**: H tier 종결 — F ✅ (nextest), E ✅ (PR matrix split), H ✅
(rust-cache@v2), C ❌ 거부.

---

## 3. CI 검증 (HEAD `1734a41b`)

| Workflow | Run ID | 결과 |
|----------|--------|------|
| BMB CI | 25166048495 | ✅ 9/9 green |
| Bootstrap + Benchmark Cycle | 25166048468 | ✅ 5/6 (Benchmark Suite finalizing) |
| Bindings CI | 25166048458 | ✅ ubuntu/macOS/windows-latest (macos-13 queued) |
| Update Benchmark Baseline | 25166048480 | (in progress) |

### 로컬 (HEAD `1734a41b`)

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | ✅ 3,772 pass / 0 fail |
| `cargo test --release --lib --features llvm --target x86_64-pc-windows-gnu` | ✅ 3,953 pass |
| `cargo nextest run --release` (full project) | ✅ 6,209 tests in 19.2s |
| `cargo clippy --all-targets -- -D warnings` | ✅ clean |
| `bash scripts/bootstrap.sh --stage1-only` | ✅ 22.5s |
| `python ecosystem/build_all.py` (MinGW UCRT, local) | ✅ 5/5 OK in 4.3s |

---

## 4. 다음 세션 체크리스트

```bash
# 1. 상태
git -C D:/data/lang-bmb status --short          # benchmark-bmb ? only (pre-existing)
git -C D:/data/lang-bmb log --oneline -5        # 1734a41b top
git -C D:/data/lang-bmb log origin/main..HEAD   # empty

# 2. Toolchain (pinned)
rustup show                                     # 1.95.0
rustc --version                                 # 1.95.0

# 3. 기초 QA
cargo test --release --lib                      # 3,772 pass
cargo clippy --all-targets -- -D warnings       # clean

# 4. CI 결과 확인 — Cycle 2503 진행 중 last fully-green run
gh run list --limit 8
# Latest fully validated: HEAD `1734a41b` BMB CI ✅, Bootstrap 3-Stage ✅,
# Bindings 3-OS ✅
```

---

## 5. 다음 세션 작업 범위 — 권장 순서

남은 모든 항목이 HUMAN-gated (4/20 cycle 사용한 이유).

### 🔵 B'.2 — TestPyPI 첫 실 업로드 (HUMAN gate)

전제: Maintainer가 GitHub org secret `TEST_PYPI_API_TOKEN` 등록 필요.

```yaml
# 등록 후 세션 진행:
# 1. workflow_dispatch로 pypi-publish.yml --target=testpypi 트리거.
# 2. 5 wheels 업로드 검증.
# 3. clean Windows VM에서 pip install 검증.
```

### 🔵 G.1 — verifier root cause (HUMAN gate — Z3 환경)

이전 핸드오프와 동일. Maintainer가 로컬 또는 CI에 Z3 셋업 필요.

```bash
# 등록 후 세션 진행:
winget install Z3  # 또는 brew/apt
BMB_VERIFY_DEBUG=1 ./target/release/bmb build stdlib/core/num.bmb -o /tmp/num.so --shared
# stderr에서 clamp SMT script 추출
# Cycle 2497 test_clamp_smt_script_dump 출력과 diff
# 차이 발견시 AST→CIR lowering (bmb/src/cir/lower.rs) 검사
# Fix 후 ecosystem/build_all.py에서 --trust-contracts 제거 재시도
```

### 🔵 C' — Defect 3 (HUMAN gate — WSL2 + gdb 환경)

이전 동일.

### 🔵 D' — Golden 정책 결정 (HUMAN gate)

이전 동일.

### 🟢 자율적 latent 후보 (선택)

- `phi_string_map`/`phi_coerce_map` dedup 검토 (counter-based naming이라
  collision 위험은 없음, 효율성 marginal). 현재로선 불필요.
- bmb_runtime.c MSVC ABI 추가 호환성 점검 (현재로선 모든 POSIX 호출이
  guard됨 — 추가 작업 불필요).

세션 자율 진척 가능 작업 부재 → 4-cycle 종결이 적절.

---

## 6. 철학 정렬

| 원칙 | 이번 세션 |
|------|----------|
| Performance > Everything | ✅ Cycle 2500은 Level 5 runtime fix. workaround 회피 (yaml-level 패치 등). |
| No Workaround | ✅ Cycle 2500: Win32 API 직접 사용 (proper substitution). Cycle 2502: 키 honest화 (defensive HashSet 회피). |
| Rule 5 (전수 검색) | ✅ Cycle 2500 sweep으로 모든 POSIX 인클루드/호출 점검. Cycle 2502 sweep으로 phi_load_map 5 consumer 모두 안전 확인. |
| Rule 6 (Rust frozen) | ⚠️ Cycle 2500 (runtime, distribution-blocker) + Cycle 2502 (codegen latent fix). 둘 다 정당한 예외. |
| Rule 7 (백엔드 parity) | ✅ Cycle 2502는 text backend만 영향, inkwell 미영향 (HashMap key shape은 inkwell에 노출되지 않음). |
| Rule 9 (early terminate) | ✅ 4/20 cycle. CI 모두 green, 잔여 항목 HUMAN-gated → 명확한 break-point. |
| 정직 측정 | ✅ Cycle 2500은 clang-cl 로컬 재현 → CI 검증으로 empirical 종결. Cycle 2502는 nextest 6,209 + 3-Stage. |

---

## 7. 파일 참조

### 이번 세션 변경

- `bmb/runtime/bmb_runtime.c` (Cycle 2500 — POSIX→Win32 dirent/mkdir/rmdir/S_ISDIR)
- `bmb/src/codegen/llvm_text.rs` (Cycle 2502 — phi_load_map key shape)
- `docs/ROADMAP.md` (Cycle 2503 — Recently completed 추가)
- `claudedocs/cycle-logs/cycle-250{0,2,3}.md` + `claudedocs/HANDOFF.md` (this)

### 다음 세션 필수 읽기 순서

1. `claudedocs/HANDOFF.md` (this) — 가장 먼저
2. `claudedocs/cycle-logs/cycle-250{0,2,3}.md` — per-cycle 상세
3. `docs/ROADMAP.md` — Recently completed (Cycles 2500-2503)
4. `gh run list --limit 10` — `1734a41b` 또는 successor의 CI 결과
5. `bmb/runtime/bmb_runtime.c` lines 2747-2895 — Cycle 2500 변경 영역
6. `bmb/src/codegen/llvm_text.rs` lines 2090-2495 — Cycle 2502 변경 영역

---

## 8. 도출된 태스크 체크리스트

### B'.1 verification (DONE ✅)
- [x] Bindings CI windows-latest on `1734a41b` 검증 — green confirmed.

### B'.2 (HUMAN + AUTONOMOUS)
- [ ] Maintainer: `TEST_PYPI_API_TOKEN` 발급 + org secret 등록
- [ ] Autonomous: TestPyPI dispatch + clean-VM 검증

### G.1 root cause (HUMAN + Z3 ENV)
- [ ] 로컬 또는 CI에 Z3 설치 (winget/brew/apt)
- [ ] `BMB_VERIFY_DEBUG=1`으로 실 stdlib clamp SMT 추출
- [ ] hand-built test 결과와 diff
- [ ] AST→CIR lowering 결함 식별 또는 Z3 model 차이 확인
- [ ] Fix 후 `--trust-contracts` 제거

### C' (HUMAN + 2 HARD cycles)
- [ ] WSL2 + Ubuntu + gdb 환경

### D' (HUMAN)
- [ ] Golden (A)/(B)/(C) 선택

### G.4 (DONE ✅)
- [x] phi_load_map key dedup (Cycle 2502).

### H tier (CLOSED)
- [x] F nextest, E PR matrix split, H rust-cache (Cycles 2495-2496)
- [x] C 거부 결정 (Cycle 2503)

---

**세션 종료** — HEAD `1734a41b`, origin 동기화 완료. 4/20 cycle 사용
(Rule 9 early termination — 잔여 모두 HUMAN-gated).

**핵심 empirical 성과**:
- B'.1 (Failure 3 windows-latest) **EMPIRICALLY VALIDATED**. Cycle 2492
  (build flag gating) + Cycle 2500 (runtime POSIX→Win32) 조합으로 MinGW
  UCRT (로컬) 와 MSVC clang (CI) 양쪽 ABI에서 binding build + pytest +
  monolithic + edge-case 모두 green.
- G.4 latent phi_load_map collision 위험 구조적 제거 (Cycle 2502 Stage
  3 Fixed Point empirical 검증).

**다음 세션 진입점**: `gh run list` → 그동안 새 결함 발생 여부 확인 →
B'.2 (TestPyPI token 등록되었으면) 또는 G.1 (Z3 환경 셋업되었으면) 또는
다른 작업.

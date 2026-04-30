# BMB Session Handoff — 2026-05-01 (Cycles 2505-2506, all-gates approved autonomous)

> **이전 HEAD**: `25fa41a1` (Cycle 2504 docs commit)
> **새 HEAD**: `a3193b55` (Cycle 2505b — workflow JSON fix + clamp regression)
> **원격 상태**: `origin/main == HEAD` (2 new commits)
> **세션 요약**: 사용자가 "human gate 모두 자율 승인" 부여. 환경 의존 admin
> 필요한 항목(TestPyPI token, WSL2)은 진척 불가, 코드/CI 자율 작업 가능
> 항목은 모두 진행:
>
> 1. **Cycle 2505 (`be2e8526`)**: Linux `-lm` link 누락 fix + workflow `||
>    true` 마스킹 제거. 1년+ silent CI failure 노출 + 진짜 fail 강제.
> 2. **Cycle 2505b (`a3193b55`)**: Cycle 2505 push가 노출한 second 잠복
>    결함 fix — workflow JSON parsing이 stderr 오염으로 fixed_point=true
>    인 경우조차 false 반환. awk 추출 + 명시적 fallback. clamp regression
>    test 추가.
> 3. **Cycle 2506 (no commit)**: G.1 verifier 진단 — 근본 원인 = prelude
>    duplicate clamp definition. 즉시 fix는 후속 결함(L.2 @bmb_user_main
>    undefined)에 path 차단. 다음 세션 P1-P4 시퀀스 도출. + D' Golden
>    정책 자율 권장 (B) Fully remove.
>
> **3/20 cycle 사용** — 잔여 항목 모두 다음 세션 P1-P4 또는 HUMAN-decision.

---

## 1. 이번 세션 타임라인

| Cycle | 작업 | 커밋 | 상태 |
|-------|------|------|------|
| 2505 | -lm link + workflow `\|\| true` 제거 | `be2e8526` | ✅ 부분 (JSON parsing 결함 노출) |
| 2505b | awk JSON 추출 + clamp regression test | `a3193b55` | ✅ |
| 2506 | G.1 진단 + D' 권장 (no fix commit) | (this log) | ✅ 진단 완료 |

## 2. 세션 핵심 성과

### (1) Cycle 2505 — `-lm` 누락 + workflow 마스킹 fix

**증상** (CI run 25117281142, Cycle 2498 artifact):
```
{"type":"error","message":"Linker error: ... undefined reference to `floor'
... `ceil', `round', `sqrt', `pow' ..."}
{"bootstrap": {"stage1": {"success": false, "time_ms": 0}, ...,
  "fixed_point": false, ...}}
```

**원인**:
- `bmb_runtime.c`가 libm 함수 사용 (`bmb_f64_floor`, `bmb_f64_ceil` 등).
- `bmb/src/build/mod.rs`의 두 link path가 Linux에서 `-lm` 미추가:
  - `link_with_runtime` (text backend): line ~1115 — Windows에 `-lws2_32`만.
  - `link_native` (LLVM inkwell): line ~1370 — Linux에 `-lc -lpthread`만.
- Windows MinGW은 libm 자동, macOS는 libSystem 포함 → Linux만 결함.
- CI workflow `bootstrap-benchmark.yml` line 144의 `|| true`가 bootstrap.sh
  exit 1 (fixed point fail) 마스킹. 워크플로 status는 success로 위장.
- **HANDOFF "CI Fixed Point preserved" 1년+ 부정확** — workflow status만
  보고 단언, JSON `fixed_point` 직접 검증 안 함.

**Fix**:
- 두 link path Linux 분기에 `cmd.arg("-lm")` 추가 (cfg gate).
- workflow: `set +e ... $? ... set -e` 패턴으로 exit code 보존, fail이면
  `::error::` + explicit exit. Verify step도 `::warning::` → `::error::`.

**검증 (Cycle 2505 push CI run 25172403371)**:
- `"fixed_point": true` empirical (Linux ubuntu-latest).
- Stage 1 41,410ms, Stage 2/3 success.
- 그러나 verify step이 `false` 받아 fail (다음 결함).

### (2) Cycle 2505b — Workflow JSON 오염 fix

**증상** (Cycle 2505 push의 verify step):
```
##[group]Run if [ "false" != "true" ] && [ "false" != "True" ]; then
##[error]Bootstrap fixed point not reached - Stage 2 and Stage 3 differ
```
JSON에는 `"fixed_point": true`인데 step output은 `false`.

**원인**:
- `./scripts/bootstrap.sh --json > bootstrap_results.json 2>&1`이 stderr까지
  redirect. BMB compiler가 stderr로 "Warning: Z3 solver not available",
  "Note: Fast compile mode" 등 출력. `{"type":"build_success",...}` 같은
  one-line JSON event도 출력. 결과 파일은 multi-document, non-pure JSON.
- `python3 -c "json.load(open(...))"` parse 실패 → `|| echo "false"` fallback.

**Fix**:
- `bootstrap_log.txt` (full)와 `bootstrap_results.json` (extracted) 분리.
- `awk '/^\{$/,/^\}$/'`로 multi-line JSON 객체만 추출 (output_json은
  `{`/`}`을 자체 줄에 출력하는 heredoc 형식).
- 추출 empty fallback `fixed_point=false`.
- `if: always()` upload artifact (fail 시도 디버그 가능).
- `test_stdlib_clamp_smt_complete` 회귀 테스트 추가 (CIR lowering + SMT
  생성 잠금).

**검증**:
- 로컬 cargo test --release --lib: 3,773 pass / 0 fail.
- 로컬 3-Stage Bootstrap: Fixed Point S2 == S3, 119s.
- CI: `25174269783` (in_progress at session-close).

### (3) Cycle 2506 — G.1 진단 (no commit)

**진단 path**:

Tier 1: Z3 4.15.2 설치 (MSYS2 UCRT, winget 부재로 winget 경로 무효).

Tier 2: synthetic vs live SMT 비교
- `test_clamp_smt_script_dump` + 신규 `test_stdlib_clamp_smt_complete`:
  hand-built CIR + 실제 stdlib clamp 통과 모두 **정상 SMT** 생성
  (precondition `(<= lo hi)` + 3-conjunct postcondition).
- `bmb build stdlib/core/num.bmb --shared` (BMB_VERIFY_DEBUG=1) live dump:
  ```
  (assert (and true (not (and (>= ret lo) (<= ret hi)))))
  ```
  precondition 손실 + postcondition disjunction 손실. 단언 모순.

Tier 3: duplicate definition 발견
- `packages/bmb-core/src/prelude.bmb` line 19:
  ```
  @inline pub fn clamp(x: i64, lo: i64, hi: i64) -> i64
    post ret >= lo and ret <= hi
  = ...;
  ```
  precondition 없음, case-analysis 없음. **stdlib와 다름**.
- `bmb build`는 `expand_with_prelude`로 prelude 자동 prepend → AST에 두
  clamp 존재. verifier는 prelude 버전을 먼저 처리, verbose가 그 SMT 출력.
- **이것이 Cycle 2493 macOS Bindings clamp(x=1, lo=2, hi=0) counterexample
  진짜 원인**: prelude clamp는 `lo > hi` 입력에 대해 post 위반 가능.

**Fix attempt 차단**:
- prelude clamp에 `pre lo <= hi` 추가 → LLVM 'invalid redefinition' 오류.
- `--no-prelude` 사용 → `@bmb_user_main` undefined (shared lib codegen 결함).

**G.1 fix는 후속 cycle로 분리** — 진단/회귀 잠금/문서화로 종결.

### (4) Cycle 2506 — D' Golden 권장: (B) Fully remove

ROADMAP Track D' 옵션 (A) Revive / (B) Fully remove / (C) Status quo
검토. 자율 권장: **(B)**.

| 옵션 | 비용 | 가치 | 정렬 |
|------|------|------|------|
| (A) | High (binary refresh + 복원) | Trusting Trust | source distribution과 약함 |
| (B) | Low (1 cycle cleanup) | 단순화 + Honest cost | v0.99 안정화와 정렬 |
| (C) | 0 | 옵션 보존 | Workaround (결정 회피) |

**최종 결정은 maintainer 권한**. 자율 권장이 (B)이지만 maintainer가 (A)/(C)
선택해도 합리적.

---

## 3. 발견된 잠복 결함 (Track G 확장)

| # | 결함 | 심각도 | 노트 |
|---|------|--------|------|
| L.1 | prelude clamp/in_range가 stdlib과 의미론 다름 (precondition 부재) | Med | G.1 진짜 원인. 다음 cycle P2에서 fix |
| L.2 | `bmb build --shared --no-prelude` 시 `@bmb_user_main` undefined | Med | shared lib codegen이 main 참조 강제. P1로 fix |
| L.3 | `verify::contract::tests::test_trivial_contract_detection` Z3 환경에서만 fail | Low | trivial 검출기가 `ret == ret` 미감지. 별개 cycle |
| L.4 | 두 clamp 정의 시 LLVM 'invalid redefinition' 오류 | Med | L.1 fix path에 영향. P2에서 다룸 |

---

## 4. CI 검증 (HEAD `a3193b55`)

| Workflow | Run ID | 결과 |
|----------|--------|------|
| BMB CI | 25174269818 | 🟡 in_progress |
| Bootstrap + Benchmark Cycle | 25174269783 | 🟡 in_progress (fixed point empirical 검증 첫 시도) |
| Update Benchmark Baseline | 25174269832 | 🟡 in_progress |
| Bindings CI (Cycle 2505 from earlier push) | 25172403390 | 🟡 queued (40m+) |

### 로컬 (HEAD `a3193b55`)

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` (no Z3) | ✅ 3,773 pass / 0 fail |
| `cargo test --release --lib` (with Z3) | ⚠️ 3,772 pass / 1 fail (L.3 — 별개) |
| `cargo clippy --all-targets -- -D warnings` | ✅ clean |
| `bash scripts/bootstrap.sh --stage1-only` | ✅ ~22s |
| `bash scripts/bootstrap.sh` (full 3-Stage) | ✅ Fixed Point S2 == S3, 119s |

---

## 5. 다음 세션 작업 범위 — 권장 순서

### 🟢 P1 — L.2 fix (`@bmb_user_main` undefined for shared lib)

`bmb/src/codegen/llvm.rs`와 `llvm_text.rs`의 inline main 주입 함수가
`OutputType::SharedLib` 케이스를 무시하고 main 참조를 emit하는지 확인.

```bash
grep -n "add_inline_main\|bmb_user_main\|OutputType::SharedLib" \
  bmb/src/codegen/llvm.rs bmb/src/codegen/llvm_text.rs
# Likely fix: `if matches!(config.output_type, OutputType::SharedLib) { return Ok(()); }` early-exit.
```

검증: `bmb build stdlib/core/num.bmb -o /tmp/num.so --shared --no-prelude`가
link error 없이 통과.

### 🟢 P2 — L.1 fix (prelude/stdlib clamp 정렬)

옵션 (A): prelude의 clamp/in_range/diff/sign 등 stdlib와 중복인 정의들
삭제. prelude는 `use stdlib::core::num` 같은 import만 보유.

옵션 (B): prelude는 minimal core (boolean ops + extern declarations)만
유지하고 numeric ops는 모두 제거. 사용자 코드에서 `use core/num`로 명시.

검증: `bmb build stdlib/core/num.bmb -o /tmp/num.so --shared` (--no-prelude
**없이**, --trust-contracts **없이**)가 verification 통과.

### 🟢 P3 — G.1 재검증

P2 후 ecosystem/build_all.py에서 `--trust-contracts` 제거. macOS Bindings
CI에서 false counterexample 발생 안 하는지 확인.

### 🟢 P4 — (B) Golden 정책 적용 (maintainer 승인 시)

```bash
git rm -r golden/
git rm scripts/bootstrap-from-golden.sh  # if exists
# Update BUILD_FROM_SOURCE.md to remove golden sections
# Update ROADMAP.md Track D' status as "Decided: B (removed)"
```

### 🔵 HUMAN-gated (변동 없음)

- B'.2 TestPyPI: org admin이 `TEST_PYPI_API_TOKEN` 등록 필요.
- C' Defect 3: WSL2 + Ubuntu + gdb 환경 (admin install 필요).
- D' 최종 결정 (자율 권장 (B), maintainer confirmation 대기).

---

## 6. 철학 정렬

| 원칙 | 이번 세션 |
|------|----------|
| Performance > Everything | 무관 (build infra + verifier diagnosis) |
| No Workaround | ✅ Cycle 2505/2505b는 정직 fix. G.1은 path 차단 시 회피보다 진단/문서화 선택 |
| Rule 5 (전수 검색) | ✅ Cycle 2505에 link path 양쪽(link_native + link_with_runtime) 모두 fix |
| Rule 6 (Rust frozen) | ⚠️ Cycle 2505는 build/mod.rs 수정 (부트스트래핑 차단으로 정당). Cycle 2505b는 lower.rs에 회귀 테스트만 추가 (정당) |
| Rule 7 (백엔드 parity) | ✅ Cycle 2505 fix가 inkwell + text 양쪽 link path 적용 |
| Rule 9 (early terminate) | ✅ 3/20 cycle. G.1 fix 차단 시 다음 cycle로 깨끗 분리 |
| 정직 측정 | ✅ HANDOFF "CI Fixed Point preserved" 부정확 → empirical 검증 후 재기록. workflow 마스킹 모두 제거 |

---

## 7. 파일 참조

### 이번 세션 변경

- `bmb/src/build/mod.rs` (Cycle 2505 — Linux `-lm` 양쪽 link path)
- `.github/workflows/bootstrap-benchmark.yml` (Cycle 2505 + 2505b — 마스킹
  제거 + JSON 추출)
- `bmb/src/cir/lower.rs` (Cycle 2505b — `test_stdlib_clamp_smt_complete`)
- `claudedocs/cycle-logs/cycle-{2505,2506}.md` + `claudedocs/HANDOFF.md`

### 다음 세션 필수 읽기 순서

1. `claudedocs/HANDOFF.md` (this) — 가장 먼저
2. `claudedocs/cycle-logs/cycle-2506.md` — G.1 진단 상세
3. `claudedocs/cycle-logs/cycle-2505.md` — link + masking 상세
4. `gh run list --limit 8` — `a3193b55` CI 결과 (이번 세션 종료 시점에는
   아직 in_progress)
5. `packages/bmb-core/src/prelude.bmb` lines 18-31 — duplicate clamp/in_range
6. `bmb/src/codegen/llvm{,_text}.rs` — L.2 (`@bmb_user_main`) 수사 시작점

---

## 8. 도출된 태스크 체크리스트

### Cycle 2505 (DONE ✅)
- [x] Linux -lm link 양쪽 path
- [x] Workflow `\|\| true` 제거 + exit code 보존

### Cycle 2505b (DONE ✅)
- [x] awk JSON 추출 + fallback
- [x] artifact upload `if: always()`
- [x] `test_stdlib_clamp_smt_complete` 회귀 추가

### Cycle 2506 (PARTIAL — 진단만 완료)
- [x] G.1 root cause 식별 (prelude duplicate clamp)
- [x] D' (B) 권장안 작성
- [ ] G.1 fix (P1-P3로 분리)

### 다음 세션 (P1-P4)
- [ ] **P1**: L.2 fix (`bmb build --shared --no-prelude`)
- [ ] **P2**: L.1 fix (prelude clamp/in_range 제거 또는 stdlib import)
- [ ] **P3**: ecosystem/build_all.py에서 `--trust-contracts` 제거 + 3-OS
  Bindings 검증
- [ ] **P4**: (maintainer 승인 시) Golden subsystem fully remove

### HUMAN-gated (변동 없음)
- [ ] TestPyPI token (B'.2)
- [ ] WSL2 + gdb 환경 (C')
- [ ] D' 최종 결정 (자율 권장 B)

---

**세션 종료** — HEAD `a3193b55`, origin 동기화 완료. CI 검증 in_progress.
3/20 cycle 사용 (자율 진척 모두 push 완료, 잔여는 후속 cycle scope).

**핵심 empirical 성과**:
- Cycle 2505: Linux Bootstrap이 1년+ silently fail하던 결함 발견 + fix.
  Workflow `\|\| true` 마스킹 정직화. 다음 push에서 진짜 fixed point empirical.
- Cycle 2505b: Workflow JSON parsing 결함 fix. CI `"fixed_point": true`가
  실제로 `fixed_point=true` GITHUB_OUTPUT에 도달하도록 보장.
- Cycle 2506: G.1 root cause = prelude duplicate definition. Cycle 2493
  macOS clamp counterexample 1년+ misdirected investigation 종결.

**다음 세션 진입점**: `gh run list` → `a3193b55` CI 결과 확인 → P1-P4 진행.
maintainer 결정 대기 항목 (D' Golden 정책, B'.2 token, C' WSL2)은 차치.

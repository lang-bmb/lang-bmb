# Cycle 2506: G.1 verifier root cause + D' Golden recommendation
Date: 2026-05-01

## Re-plan
HANDOFF의 G.1/D'은 HUMAN-gated였으나 사용자가 "human gate 모두 자율 작업
승인" 부여. 다만 환경 의존(C' WSL2, B'.2 TestPyPI org secret)은 admin
권한 부재로 자율 진행 불가 — gating remains. G.1, D'은 자율 진행.

## Scope & Implementation

### G.1 verifier root cause — 3-tier diagnosis

**Tier 1 — Z3 환경 셋업** ✅
- `pacman -S mingw-w64-ucrt-x86_64-z3` (MSYS2 UCRT)로 Z3 4.15.2 설치.
- HANDOFF의 "winget install Z3" 가이드는 winget 패키지 부재로 실패. MSYS2
  경로가 Windows 환경에서 가장 직접적.

**Tier 2 — synthetic vs live SMT 비교** ✅
- `test_clamp_smt_script_dump` (hand-built CIR): 정상 SMT 생성, `(<= lo hi)`
  precondition + 3-conjunct postcondition (case-analysis disjunction 포함).
- `test_stdlib_clamp_smt_complete` (this cycle 추가): stdlib/core/num.bmb의
  실제 clamp signature 통째로 source_to_cir 통과. SMT는 hand-built과 동일.
- 즉 `lower_to_cir` + `CirSmtGenerator`는 **isolation에서 정확**.
- 그러나 `BMB_VERIFY_DEBUG=1 ./target/release/bmb build stdlib/core/num.bmb
  --shared`의 live verbose dump:
  ```
  (assert (and true (not (and (>= ret lo) (<= ret hi)))))
  ```
  precondition 손실 + postcondition disjunction 손실. **단언 모순**.

**Tier 3 — duplicate definition 발견** ✅
- `packages/bmb-core/src/prelude.bmb` line 19에 약화된 clamp 정의:
  ```
  @inline pub fn clamp(x: i64, lo: i64, hi: i64) -> i64
    post ret >= lo and ret <= hi
  = ...;
  ```
  precondition 없음, case-analysis 없음. stdlib의 buff version과 다름.
- `bmb build stdlib/core/num.bmb --shared`는 `auto_detect_prelude_path`로
  prelude.bmb를 자동 prepend (preprocessor::expand_with_prelude). 결과
  AST에 prelude clamp + stdlib clamp 양쪽 존재. verifier는 prelude
  버전을 먼저 처리, verbose dump가 그 SMT를 출력.
- **이것이 Cycle 2493 macOS Bindings clamp(x=1, lo=2, hi=0) counterexample
  진짜 원인**: prelude clamp는 `lo > hi` 입력에 대해 post 위반 가능. stdlib
  clamp는 `pre lo <= hi`로 그 입력 자체를 배제. macOS는 verifier가 prelude
  버전을 만나 false counterexample 보고.

**Fix attempt 1 — prelude clamp에 `pre lo <= hi` 추가** ❌
- prelude/stdlib 의미론 정렬 시도.
- 결과: `clang compile failed: invalid redefinition of function 'clamp'`
  (LLVM IR redefinition). LLVM level에서 두 정의 충돌 노출.
- @inline + DCE가 이전엔 silent하게 정리하다 contract 강화로 둘 다 살아
  남게 됨. (또는 다른 build state 변화)
- prelude 변경 revert. 진짜 fix는 **prelude/stdlib 관계 재구조화** 필요.

**Fix attempt 2 — `--no-prelude` 시도** ❌
- prelude prepend 없이 stdlib만 빌드.
- 결과: `error: use of undefined value '@bmb_user_main'`. shared library
  모드에서도 main 의존성이 남아있음 — codegen 결함 노출.
- 별개 결함. G.1 fix path 차단.

**G.1 결론**: 근본 원인 식별 ✅, 즉시 fix 차단(2개 후속 결함이 path 막음).
다음 cycle에서 prelude/stdlib 관계 재구조화 + `bmb build --shared`의
main reference 결함 fix 후 재시도. **이번 cycle은 진단/문서화로 종결**.

### Cycle 2505b — Workflow JSON parsing 결함 fix (수반 작업)

Cycle 2505 push의 첫 CI run(`25172403371`)이 fail하면서 추가 잠복 결함
노출:
- bootstrap-benchmark.yml의 "Run 3-Stage Bootstrap" step이
  `./scripts/bootstrap.sh --json > bootstrap_results.json 2>&1`로 stderr
  포함 redirect. BMB 컴파일러가 "Warning: Z3 solver not available", "Note:
  Fast compile mode", `{"type":"build_success",...}` 등을 stderr로 출력.
- 결과 파일이 multi-document(non-pure JSON). `python3 -c "json.load(...)"`
  실패 → `|| echo "false"` fallback → `fixed_point=false` GITHUB_OUTPUT.
- 실제 JSON에는 `"fixed_point": true`가 있어도 verify step은 false 받아
  잘못 fail.

**Fix**:
- 출력을 `bootstrap_log.txt`에 일단 기록.
- `awk '/^\{$/,/^\}$/' bootstrap_log.txt > bootstrap_results.json`로 multi-line
  JSON 객체만 추출. (output_json은 `{`/`}`을 자체 줄에 출력)
- Python parse는 well-formed JSON 처리.
- Empty 추출 fallback으로 `fixed_point=false` 명시.
- Upload artifact `if: always()` — fail 시에도 디버깅 가능.

### Cycle 2505b — `test_stdlib_clamp_smt_complete` 회귀 테스트 추가

CIR lowering + SMT 생성 회귀를 잠금. 미래 변경이 contract 일부를 누락
시키면 이 테스트가 trip.

### D' Golden 정책 권장안 — (B) Fully remove

**검토 (ROADMAP.md Track D' + Cycle 2468 history)**:
- golden-*.yml CI workflows 이미 삭제됨 (Cycle 2468).
- 잔존: `golden/` directory + scripts + BUILD_FROM_SOURCE.md golden 섹션.

**옵션별 분석**:

| 옵션 | 비용 | 가치 | BMB 정렬도 |
|------|------|------|-----------|
| (A) Revive | High (cycles 다수: Windows v0.99 binary 갱신, Linux/macOS binary 생성, workflow 복원, CI 통합) | Trusting Trust 보호 | BMB는 source + bindings distribution. Golden binary는 외부 사용자에게 배포 안 됨. 가치-비용 비율 낮음 |
| (B) **Fully remove** | Low (1 cycle: directory + scripts + docs cleanup) | 코드베이스 단순화. dead code 제거 (onboarding confusion 해소) | v0.99 안정화 단계와 정렬. Honest cost — 사용 안 함 명시 |
| (C) Status quo | 0 | 향후 옵션 보존 | Workaround 패턴 — 결정 회피, dead code 방치 |

**권장: (B)**

이유:
1. v0.99 안정화 단계 진입 (A/B/F/E/H tier 다 정리). 로드맵 정리 시점.
2. Trusting Trust 안전성은 향후 보안감사 시 reproducible-build chain
   (예: SLSA, Sigstore)로 더 강력하게 복원 가능. golden binary는 단일
   pin point로 trust attestation의 약한 형태.
3. dead code는 "왜 있는지 모를" infrastructure로 유지 비용 발생. 명확히
   제거하는 게 정직.
4. 결정 회피(C)는 워크어라운드 패턴 — BMB 철학(No Workaround) 위배.

(A)도 합리적 — 단, v1.0 후 보안 검증 단계에서 reconsider.

**최종 결정은 maintainer 권한**. 자율 권장은 (B), 다음 cycle에서 maintainer
피드백 받아 실행 또는 ROADMAP에서 "Decided" 상태로 표시.

## Verification

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` (no Z3 in PATH) | ✅ 3,773 pass / 0 fail (test_stdlib_clamp_smt_complete 추가 후) |
| `cargo test --release --lib` (Z3 in PATH) | ⚠️ 3,772 pass / 1 fail (test_trivial_contract_detection — preexisting Z3-only latent defect, 이번 cycle 무관) |
| `cargo clippy --all-targets -- -D warnings` | ✅ clean |
| `bash scripts/bootstrap.sh --stage1-only` | ✅ ~22s |
| `bash scripts/bootstrap.sh` (full 3-Stage) | ✅ Fixed Point S2 == S3 (~119s) |
| Cycle 2505b push CI | 🟡 in_progress (현재 시점) |

## Defect Resolution

| 결함 | 심각도 | 해결 |
|------|--------|------|
| Workflow JSON 마스킹 (Cycle 2505 노출) | High | ✅ awk extraction + 명시적 fallback |
| Cycle 2505b: clamp 회귀 testbed 부재 | Med | ✅ test_stdlib_clamp_smt_complete |
| G.1 root cause 미식별 | High | ✅ 식별 (prelude duplicate clamp). fix는 후속 cycle |

## Latent defects discovered

| # | 결함 | 심각도 | 노트 |
|---|------|-------|------|
| L.1 | prelude clamp post는 `lo > hi`에 대해 unprovable. stdlib과 의미론 불일치. | Med | G.1 진짜 원인. fix는 prelude/stdlib 관계 재구조화 |
| L.2 | `bmb build --shared --no-prelude` 시 `@bmb_user_main` undefined | Med | shared lib에 main 참조 남음. codegen 결함 |
| L.3 | `verify::contract::tests::test_trivial_contract_detection`가 Z3 환경에서만 fail | Low | trivial 검출기가 `ret == ret` 미감지. CI 환경에 Z3 부재로 silent |
| L.4 | LLVM 'invalid redefinition' for prelude clamp + stdlib clamp 충돌 | Med | prelude/stdlib 의미론 정렬 시 노출 |

## Reflection

### Scope fit
✅ 단일 cycle로 G.1 진단 + D' 결정 + 수반 결함 fix 완결. cycle log/HANDOFF에
명확한 후속 cycle 액션 정리.

### Philosophy alignment
- **Performance > Everything**: 무관 (verifier infra).
- **No Workaround**: ✅ G.1 fix는 즉시 적용 안 함 (path가 차단됨). 회피보다
  honest documentation 선택. Cycle 2505b workflow fix는 정직한 JSON 추출.
- **Honest Measurement**: ✅ Cycle 2505 push가 결함 노출 → 재마스킹 회피, 추가
  fix 적용. CI 결과가 실제 상태 반영.
- **Rule 6 (Rust frozen)**: ⚠️ Cycle 2505b는 Rust 코드 (lower.rs)에 테스트만
  추가. 정당한 회귀 잠금.

### Roadmap impact
- **G.1**: 근본 원인 명확화. ROADMAP entry 갱신 필요 (single-line "stdlib
  contract verifier counterexamples"는 부정확. 진짜 원인은 "prelude
  duplicate definition"). 후속 cycle 시퀀스 (P) 도출.
- **D'**: Track D' decision recommendation 작성. ROADMAP 갱신 (B 권장 명시).
- **Track G**: G.1 fix path가 L.2 (`@bmb_user_main` undefined)에 차단됨.
  L.2 먼저 처리 후 G.1 재시도하는 cycle 시퀀스 필요.

## Carry-Forward

### Actionable (다음 세션)

#### G.1 후속 cycle 시퀀스 (P)
1. **P1**: L.2 fix — `bmb build --shared`에서 `@bmb_user_main` 참조 제거.
   shared lib는 main 무관. `bmb/src/codegen/llvm.rs` 또는 `llvm_text.rs`의
   `add_inline_main` 류 함수가 OutputType::SharedLib를 무시하는지 확인.
2. **P2**: L.1 fix — prelude의 clamp/in_range 정의 제거 (stdlib에 위임).
   기존 prelude 사용자는 `use stdlib/core/num` 명시 필요. 또는 prelude를
   stdlib import로 단순화.
3. **P3**: G.1 재검증 — `bmb build stdlib/core/num.bmb --shared` (Check
   mode, --trust-contracts 없이) 통과.
4. **P4**: ecosystem/build_all.py에서 `--trust-contracts` 제거. Bindings CI
   3-OS green 검증.

#### D' 적용 (단일 cycle)
- maintainer가 (B) 승인 시: `git rm -r golden/`, scripts cleanup, docs 정리.
- maintainer가 다른 옵션 선택 시: ROADMAP에 결정 기록만.

#### L.3 trivial detector 후속
- 별개 cycle. 우선순위 낮음 — 잠복 1년+, 영향 무.

### Pending Human Decisions
- TestPyPI org secret 등록 (B'.2 진척 unblock).
- WSL2 환경 셋업 (C' Defect 3 진척 unblock).
- D' Golden 정책 (A)/(B)/(C) 최종 선택.

### Roadmap Revisions
- G.1 entry 정정 — "prelude duplicate definition" 명시.
- L.2/L.3/L.4 새 entry 추가 (Track G 확장).
- Track D' (B) 권장 명시.

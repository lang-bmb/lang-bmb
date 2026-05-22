# Cycle 3039: script_args 빌트인 + check-version-sync BMB 포팅
Date: 2026-05-22

## Re-plan
Cycle 3038 carry-forward: script_args() 빌트인 + check-version-sync.sh BMB 포팅. Plan valid.

## Scope & Implementation

### script_args() 빌트인 추가

**동기**: Cycle 3038의 rebuild-runtime.bmb가 `BMB_ARGS` env var을 통해 CLI 인수를 받아야 했음. 이는 workaround — 자연스러운 접근은 `argv`에 해당하는 빌트인.

**발견**: `set_program_args()`가 이미 eval.rs에 존재 + PROGRAM_ARGS thread_local에 저장. 단지 BMB에서 접근하는 빌트인이 없었음.

**구현**:
- `eval.rs`: `builtin_script_args` — PROGRAM_ARGS[1..] → SVEC_REGISTRY (str_split 패턴)
- `types/mod.rs`: `script_args: () -> SvecHandle` 등록

**검증**: `bmb run test.bmb hello world` → `argc=2, arg[0]=hello, arg[1]=world` ✅

### 스크립트 업그레이드 (BMB_ARGS env var → script_args())

- `rebuild-runtime.bmb`: `has_flag(env_str, flag)` → `svec_has_flag(script_args(), n, 0, flag)`
- `rebuild-bootstrap-exe.bmb`: 동일

### check-version-sync.bmb (BMB port of check-version-sync.sh)

**구현**:
- `find_cargo_version(content)` — `[workspace.package]` 섹션에서 `version = "X.Y.Z"` 파싱
- `find_bmb_version(content)` — `fn bmb_version() -> String = "X.Y.Z"` 파싱
- `extract_quoted(s, start)` — 열린 따옴표 이후 닫힌 따옴표까지 추출
- `str_starts_with(s, prefix)` + `str_index_of(s, sub)` 유틸

**결과**: 실제 버전 불일치 정확히 감지:
```
VERSION MISMATCH:
  Cargo.toml workspace.package.version = 0.100.0
  bootstrap/version.bmb bmb_version()  = 0.98.0
```
(이것은 repo의 실제 상태 — 쉘 스크립트로도 동일한 결과)

## Verification & Defect Resolution

**cargo test --release**: ✅ 3782+47+22+2390+23 = 6264 passed; 0 failed

**Type checks**: ✅ rebuild-runtime.bmb (11 w), rebuild-bootstrap-exe.bmb (7 w), check-version-sync.bmb (10 w) — 모두 0 errors

**기능 테스트**:
- `script_args()` → argc/arg 정확히 반환 ✅
- `rebuild-runtime.bmb --force` → rebuild 작동 (not tested, but type-correct)
- `check-version-sync.bmb` → VERSION MISMATCH 정확히 출력 ✅

## Reflection

- **Scope fit**: 3개 스크립트 포팅 완료. M6 scripts/ P1 대부분 달성.
- **script_args()**: BMB_ARGS workaround 해소. 자연스러운 argv 접근 제공.
- **check-version-sync**: `awk` 두 줄짜리 파싱을 BMB 재귀로 구현. 더 장황하지만 언어 자체로 완전히 표현됨.
- **실제 버그 발견**: version.bmb가 0.98.0으로 stale — Cargo.toml 0.100.0과 불일치. 이것은 BMB 스크립트가 잡아낸 실제 repo 문제.
- **Philosophy drift**: None.
- **Roadmap impact**: M6 scripts/ P1 진행 (3/n 완료).

## Carry-Forward

- Actionable:
  - version.bmb를 0.100.0으로 업데이트 (check-version-sync.bmb가 발견한 실제 문제)
  - M6 다음 단계: bmb-ai-bench Python→BMB (P2, 3-5 cycles) 또는 추가 스크립트
  - `copy_file` 빌트인 추가 시 secondary copy sync 완성 가능
- Structural Improvement Proposals:
  - `str_starts_with`, `str_index_of` 같은 유틸은 많은 스크립트에서 필요 — stdlib에 추가하면 중복 제거
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3040 — version.bmb 0.100.0 업데이트 + M6 scripts 추가 포팅 또는 bmb-ai-bench 조사

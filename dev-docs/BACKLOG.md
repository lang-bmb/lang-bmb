# 백로그

> 갱신: 2026-10-24 · 단계와 통과 기준은 `docs/ROADMAP.md`.
> 태스크는 완료 기준이 측정·확인 가능한 크기로 쪼갠다. 완료로 바꿀 때는 근거(커밋·테스트·측정)를 적는다.
> 선택·도출·결정 권한 규칙은 `dev-docs/AUTONOMY.md`.

**상태:** ✅ 완료 · 🔄 진행 · ⏳ 대기 · 🚫 막힘

**ID 규칙:** `단계-번호`(예: `1B-03`). 번호는 재사용하지 않는다. 버린 태스크는 지우지 않고 ~~취소선~~과 사유를 남긴다.

## 지금 할 일

측정 결과와 무관하게 코드로 진행할 수 있는 것(2026-10-24 추가, 이 순서로):

1. `INF-07` 장기 퍼징: 첫 야간 실행 확인
2. 나머지(`1B-17`~`1B-20`, `1E-12`, `1E-13`, `2D-01`, `2D-02`)는 완료

측정 결과에 묶인 것:

1. `1E-10` 실제 모델 측정 — 사용자 머신에서 실행(`bench/m1/README.md`)
2. 결과가 커밋되면 `1E-06` 보고서, `1E-11` 마찰 기록, `1E-07` 2단계 우선순위 ADR

## 0 착수

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 0-01 | v0 보존과 main 리셋 | `legacy/v0` 브랜치, main 리셋 커밋 | — | ✅ `9bb88ea` |
| 0-02 | 설계·착수 문서 | `docs/DESIGN.md`, `dev-docs/v2-genesis.md` | — | ✅ `c7f8d5a` |
| 0-03 | ADR 체계와 착수 결정 이관 | ADR-0001~0016 | 0-02 | ✅ `9a3adec` |
| 0-04 | `v0.100-legacy` 태그 | GitHub에 태그 존재. 이 환경에서는 태그 푸시가 막혀 GitHub 웹에서 생성 필요 | 0-01 | 🚫 |
| 0-05 | 작업 브랜치 정리 | 원격 브랜치 삭제가 막혀 GitHub 웹에서 삭제 필요 | — | 🚫 |

## 인프라

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| INF-01 | CI | GitHub Actions가 push·PR마다 `scripts/check.sh` 실행 | INF-05 | ✅ `.github/workflows/check.yml`, 실행 #2 성공 |
| INF-02 | Z3 설치와 버전 고정 | 정해진 Z3 버전이 CI와 개발 환경에 설치됨. 버전이 다르면 빌드가 경고가 아니라 실패 | — | ✅ `scripts/toolchain.env`, `bmb doctor`, `tests/toolchain.rs` |
| INF-03 | LLVM 버전 고정 | LLVM 18 고정, CI 설치. 버전이 다르면 실패. IR을 고정 버전으로 검증하는 일은 1C-01 | — | ✅ `tests/toolchain.rs` |
| INF-04 | 세션 시작 훅 | 리포의 `.claude/settings.json` SessionStart 훅으로 새 세션에서 Z3·LLVM이 준비되어 `scripts/check.sh`가 바로 통과 | INF-02, INF-03, INF-05 | ✅ `.claude/hooks/session-start.sh`(Z3 제거 후 재설치로 확인) |
| INF-05 | 단일 검증 게이트 | `scripts/check.sh` 하나로 fmt·clippy(`-D warnings`)·test·백로그 검사를 실행. CI와 로컬이 같은 스크립트를 씀 | — | ✅ `scripts/check.sh` |
| INF-06 | 백로그 검사 | ID 중복, 존재하지 않는 의존 ID, 근거 없는 ✅, 의존이 미완료인 🔄를 찾아 실패하는 스크립트. 게이트에 포함 | INF-05 | ✅ `scripts/check_backlog.py` |
| INF-07 | 장기 퍼징 정기 실행 | 매일 1회 CI가 프론트엔드 퍼징(`BMB_FUZZ_ITERS`)과 건전성 퍼징(`BMB_SOUND_ITERS`)을 게이트보다 큰 횟수로 실행. 실패 시 시드가 로그에 남음 | 1A-09, 1B-17 | 🔄 `.github/workflows/nightly.yml`(매일, 수동 실행 가능). 첫 실행 확인 대기 |

## 1A 프론트엔드

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 1A-01 | 워크스페이스와 `bmb` 크레이트 | 외부 의존 0 | — | ✅ `d394946` |
| 1A-02 | 렉서·파서, 항목·문장 단위 오류 복구 | 한 번 실행에서 여러 오류 보고 | 1A-01 | ✅ `d394946` |
| 1A-03 | 검사기: 이름·타입·2차 참조·별칭·명세 배치·이름 붙은 술어·공허 계약·`decreases`·효과·`main` | 회귀 코퍼스 통과 | 1A-02 | ✅ `d394946` |
| 1A-04 | 안정 진단 코드와 결정론적 JSONL | 51개 코드 전부 코퍼스에서 발생, 결정론 테스트 | 1A-03 | ✅ `d394946` |
| 1A-05 | Rust 습관 진단(E02xx)과 수정 제안 | 타입 생략·매크로·`return`·`for`·`as`·`?`·`unwrap` 등 | 1A-03 | ✅ `d394946` |
| 1A-06 | 기계적 수정에 `replacement` 채우기 | E02xx 중 기계적으로 고칠 수 있는 것은 모두 `replacement` 보유. 적용 후 재검사에서 해당 오류가 사라짐을 테스트 | 1A-05 | ✅ 9종(E0104·E0201 타입 추론·E0202 `vec!`·E0206·E0208·E0209·E0402·E0601) `replacement`, `tests/fixes.rs` |
| 1A-07 | 별칭 검사 정밀화 | `&mut` 인자 별칭 검사에서 `if`·블록 인자를 보수적으로 거부하지 않고 실제로 추적 | 1A-03 | ✅ 블록 문장까지 정확히 추적, `ok/search.bmb`의 `alias_ok`, `err/types.bmb`의 `alias_err` |
| 1A-08 | 진단 코드 문서 자동 생성 | `bmb codes`로 `docs/diagnostics.md`를 생성하고, 어긋나면 CI 실패 | INF-01 | ✅ `bmb codes --md` → `docs/diagnostics.md`, 게이트의 `docs` 단계 |
| 1A-09 | 파서 퍼징 | 임의 입력에서 패닉 없음(퍼저 일정 시간 실행) | 1A-02 | ✅ `tests/fuzz.rs`(기본 1,500회, 게이트), 장기 실행 200,000회 중 타입 통과 2,428개까지 증명·IR 생성, 패닉 0 |

## 1B 증명 게이트

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 1B-01 | ADR: 정수 의미론과 SMT 인코딩 | 실행 산술(64비트, 무오버플로 의무)과 명세 산술의 의미, bitvector 대 정수 인코딩 결정. v0 `verify-vc` 설계 검토 포함 | — | ✅ ADR-0017 |
| 1B-02 | 검사된 AST → 검증용 중간 표현 | 타입이 확정된 IR. 지역 변수 SSA화, 루프는 불변식으로 절단 | 1A-03, 1B-01 | ✅ `verify.rs` 기호 실행(SSA 상수) |
| 1B-03 | VC 생성(최약 전제) | 대입·`if`·`while`(불변식 진입·유지·종료)·호출(전제 증명, 결과 가정)·`assert` | 1B-02 | ✅ `verify.rs`, `tests/verify` |
| 1B-04 | 암묵적 증명 의무 | 오버플로, 0 나눗셈, 인덱스 범위, `i64::MIN / -1` | 1B-03 | ✅ `tests/verify/fail/arith.bmb`, `bounds.bmb` |
| 1B-05 | 종료성 | 루프와 재귀(상호 재귀 포함)의 `decreases`가 감소하고 하한이 있음 | 1B-03 | ✅ `tests/verify/pass/recursion.bmb`, `fail/loops.bmb`, `fail/contracts.bmb` |
| 1B-06 | `spec fn` 인코딩 | 기본 불투명. `#[uses(..)]`로 선언한 정의만 펼침. 재귀 `spec fn`은 `decreases`로 정의 | 1B-03 | ✅ ADR-0017 §4, `fail/contracts.bmb`(불투명·비종료) |
| 1B-07 | 수량자와 트리거 | `forall`은 명시 트리거로만. 트리거 없는 `forall`은 진단 | 1B-06 | ✅ E0511·E0512, `:pattern` 출력 |
| 1B-08 | `&mut`과 `old` | 배열 갱신 모델, `ensures`의 `old(a)` | 1B-03 | ✅ `ok/search.bmb`(clear), `fail/contracts.bmb`(forgets_write) |
| 1B-09 | SMT-LIB 출력과 Z3 프로세스 연결 | 결정론적 질의 텍스트, 고정 `rlimit`. unknown·timeout·솔버 부재 = 실패 | 1B-03, INF-02 | ✅ `smt.rs`, ADR-0018 의무 격리 |
| 1B-10 | 반례 보고 | 실패한 의무마다 구체 입력값을 JSONL 진단에 첨부 | 1B-09 | ✅ `fail/*.jsonl`의 counterexample |
| 1B-11 | `bmb verify` 명령 | 정의별 증명 상태를 JSONL로 출력 | 1B-09 | ✅ `bmb verify [--emit-smt DIR]` |
| 1B-12 | 판정 회귀 코퍼스 | `tests/verify/pass`(전부 증명), `tests/verify/fail`(전부 거부 + 기대 진단). 틀린 코드 유형별 1개 이상 | 1B-09 | ✅ `compiler/tests/verify.rs` |
| 1B-13 | 게이트 강건성 테스트 | Z3 부재·`rlimit` 최소·질의 결정론을 자동 테스트로 확인 | 1B-12 | ✅ `compiler/tests/gate.rs` |
| 1B-14 | 양화 공리로 인한 `unknown`에서 반례 얻기 | 거짓 계약이 술어 정의 공리 때문에 `unknown`이 되는 경우(`fail/contracts.bmb`의 wrong_post)에도 반례를 보고. 방법은 실험으로 정함 | 1B-12 | ✅ ADR-0023 기저 인스턴스 재시도. `wrong_post`·`forgets_write`가 확정 반례(E0802)로 |
| 1B-15 | 증명 비용 보고 | `bmb verify`가 의무별 `rlimit` 사용량을 출력. 안정성 추이를 1E에서 측정 가능 | 1B-12 | ✅ 상태 줄 `rlimit`·`rlimit_max`, L1·`status`의 `rlimit_max`(결정론 확인) |
| 1B-16 | 비선형 의무 진단 | `unknown`의 원인이 비선형 산술일 때 그렇다고 진단하고 우회 방법(보조 술어·계약 분할)을 제안 | 1B-12 | ✅ `unknown`이고 변수끼리 곱하면 `fix`에 원인·우회 안내, `tests/verify/fail/nonlinear.bmb`, `nonlinear_detection` |
| 1B-17 | 건전성 퍼징 | 임의 정수 함수·계약을 증명하고, 증명된 함수가 전제를 만족하는 입력에서 결함(오버플로·0 나눗셈)이나 사후 조건 위반을 내지 않음을 독립 해석기로 확인. 검사 제거 빌드와 전체 검사 빌드의 출력이 해석기와 일치 | 1B-12, 1C-05 | ✅ `compiler/tests/soundness.rs`(게이트 6배치). 60배치: 함수 480개 중 증명 109개를 전제 만족 입력 383개로 실행, 위반 0. 두 빌드 3,664회 실행이 해석기와 일치. 생성기 버그 1건(음수 리터럴 메서드 우선순위) 발견·수정, 컴파일러 버그 0 |
| 1B-18 | 반례 재현 | 확정 반례(E08xx `counterexample:`)마다 입력이 전제를 만족하고 실제로 함수를 깨뜨림을 해석기로 확인 | 1B-12 | ✅ 같은 테스트. 60배치에서 확정 반례 620개 재현, 거짓 반례 0 |
| 1B-19 | 증명 안정성 측정 | 코퍼스 전체를 Z3 시드 여러 개로 증명해 판정·`rlimit`이 변하는 의무를 보고하는 명령과 테스트 | 1B-15 | ✅ `bmb stability [--seeds N]`, `lib::stability_sources`, `tests/stability.rs`(코퍼스 전체 시드 3개). 시드 5개: 판정이 바뀐 정의 0, `rlimit` 변동 최대 2.21배(`fail/arith.bmb`) |
| 1B-20 | 코퍼스 확장 | `tests/verify/pass`에 루프·재귀·배열·`&mut`·`old`·수량자 정의 8개 이상, `tests/run`에 그것을 실행하는 예제 1개 이상 | 1B-12, 1C-05 | ✅ `tests/verify/pass/loops.bmb`(5개)·`search.bmb`(3개), `tests/run/search.bmb`(+`.out`). 처음 쓴 `gcd` 명세가 틀린 것을 반례가 짚음. `check/ok`는 증명 코퍼스가 겸하므로 추가하지 않음 |

## 1C 코드 생성

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 1C-01 | LLVM 텍스트 IR 생성 | 정수·배열·루프·호출·`print_i64` | 1B-02, INF-03 | ✅ `codegen.rs`, `bmb emit-llvm` |
| 1C-02 | 런타임과 링크 | `print_i64` 구현, `bmb build`로 실행 파일 생성 | 1C-01 | ✅ IR 내 `@bmb_print_i64`(printf), `bmb build`(clang) |
| 1C-03 | 증명 기반 검사 제거 | 증명된 범위 사실이 있는 인덱스만 검사 생략. 증명 경로를 IR 주석으로 남김 | 1C-01, 1B-12 | ✅ 의무별 증명 여부로 검사 생략·잔존, `tests/codegen.rs` |
| 1C-04 | `noalias`와 검증된 `llvm.assume` | `&mut` 인자에 `noalias`. assume은 증명된 사실만 | 1C-03 | ✅ `noalias`/`readonly`, `nsw`/`nuw`/`inbounds`에 `; proved Exxxx` 주석. assume은 아직 방출하지 않음 |
| 1C-05 | 실행 코퍼스 | `tests/run`: 예제 실행 결과가 기대 출력과 일치 | 1C-02 | ✅ `tests/run/basics.bmb`, `run_corpus_prints_expected_output` |
| 1C-06 | IR 검사 테스트 | 증명이 있으면 범위 검사가 없고, 없으면 남는 것을 IR에서 확인 | 1C-03 | ✅ `proved_index_has_no_check_and_unproved_keeps_one` |
| ~~1C-07~~ | ~~증명된 사실을 `llvm.assume`으로 전달하는 실험~~ | 루프 불변식·사후 조건 중 증명된 것을 assume으로 내보냈을 때 성능 변화를 측정하고, 이득이 있을 때만 채택(ADR) | 1E-05 | 취소: 7개 벤치에서 BMB가 안전 Rust와 대등하고 unchecked Rust도 빠르지 않아 assume으로 얻을 여지가 측정되지 않음(`bench/baseline/results/2026-10-24.md`). 여지가 측정되면 새 태스크로 |
| 1C-08 | 메모리 접근 정렬 표기 | 원소 load/store와 alloca에 `align 8` 등 정확한 정렬을 붙이고 IR 검사 테스트 추가 | 1C-01 | ✅ 원소 load/store에 `align 8`(i64)·`align 1`(bool), `word_elements_are_eight_byte_aligned` |

## 1D 맵과 도구

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 1D-01 | ADR: 정규형과 정의 해시 | 포매터 규칙, 해시 입력의 정의(정규형 텍스트 + 의존 계약 해시 여부) | 1A-03 | ✅ ADR-0019 |
| 1D-02 | 정규형 포매터 `bmb fmt` | 왕복 불변(포맷 두 번 = 한 번) 테스트 | 1D-01 | ✅ `fmt.rs`, `bmb fmt [--check]`, `tests/fmt.rs`(멱등·증명 결과 보존) |
| 1D-03 | 정의 단위 content hash | 파일 배치·공백을 바꿔도 해시 불변 | 1D-02 | ✅ `sha256.rs`(FIPS 벡터), `bmb hash`, `tests/fmt.rs`(배치·주석·순서 불변, 의미 변화 감지) |
| 1D-04 | ADR: 맵 JSON 스키마 L0~L2 | 하위 호환 규칙 포함 | 1D-01 | ✅ ADR-0020 |
| 1D-05 | 맵 도출 | L0 목차, L1 표면(계약 이름·효과·증명 상태), L2 해석된 호출 엣지 | 1D-03, 1D-04 | ✅ `map.rs`, `bmb map` |
| 1D-06 | 읽기 질의 | `units`, `surface`, `edges`, `slice`, `find`, `contracts`, `status` | 1D-05 | ✅ `bmb query`, `tests/map.rs` |
| 1D-07 | 슬라이스 크기 계산·보고 | 정의마다 "본문 + 의존 계약" 크기. M1은 보고만 | 1D-06 | ✅ `slice`의 `bytes`·`approx_tokens` |
| 1D-08 | 쓰기 트랜잭션 | `tx.begin/put/remove/commit/abort`. 구조·타입 오류 시 전체 거부 | 1D-03 | ✅ `workspace.rs`, `bmb apply`, `tests/workspace.rs`(전체 거부·무변경, 증명 실패는 커밋) |
| 1D-09 | 결정론적 파일 배치 | 커밋된 정의를 도구가 정한 위치에 정규형으로 저장 | 1D-08 | ✅ `bmb import`, 단위당 파일 하나(ADR-0019 §3), 분할 무관 동일 배치 테스트 |
| 1D-10 | CLI 노출 | 모든 질의·트랜잭션을 `bmb` 하위 명령으로 | 1D-06, 1D-08 | ✅ `check verify build emit-llvm fmt hash map query import apply`, 디렉터리 인자 |
| 1D-11 | MCP 서버 노출 | CLI와 같은 JSON. 에이전트가 파일 접근 없이 과제 수행 가능 | 1D-10 | ✅ `mcp.rs`, `bmb mcp DIR`, `tests/mcp.rs`(전체 에이전트 세션) |
| 1D-12 | 맵·질의 결정론 테스트 | 두 번 실행 바이트 동일, 파일 배치 변경에도 동일 | 1D-06 | ✅ `tests/map.rs`(결정론·배치 독립) |
| 1D-13 | 증명 문맥 제안 | 실패한 의무에 대해 펼치면 증명될 수 있는 `#[uses]` 후보를 결정론적으로 제안(ADR-0017의 비전이 규칙 때문에 생기는 부담 완화) | 1D-06, 1B-12 | ✅ 실패 진단의 `fix`에 펼치지 않은 술어 목록, `hidden_predicate_hint_leads_to_a_proof` |

## 1E 측정

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 1E-01 | ADR: 측정 절차 | 과제·프롬프트 동결 방법, 1회 성공률 정의, 재시도 금지, 보고 형식 | — | ✅ ADR-0021 |
| 1E-02 | 과제 집합 | 수십 개 정의 규모의 정수·배열 과제와 정답 명세. 동결 해시 커밋 | 1E-01 | ✅ `bench/m1/tasks`(15개), `seed`(정의 23개), `MANIFEST` |
| 1E-03 | 에이전트 하네스 | MCP 도구만으로 과제를 수행하고 로그를 남김. 파일 직접 접근 차단 | 1D-11, 1E-02 | ✅ `bench/m1/harness.py`, 대본 에이전트(oracle 15/15, weak 0/1), `anthropic_agent.py`(스텁 테스트) |
| 1E-04 | 지표 수집 | 탐색 정확도, 질의당 토큰, 1회 성공률, Rust 습관 오류율(E02xx 빈도), 미증명 면적 | 1E-03 | ✅ 하네스가 5개 지표를 과제별로 기록(실측은 1E-10) |
| 1E-05 | Rust 성능 기준선 | v0 공정 하네스 기반. Rust 기본·`get_unchecked` 변형과 비교. 하네스와 함께 커밋 | 1C-05 | ✅ `bench/baseline/run.py`, `results/2026-10-24.md` |
| 1E-06 | 측정 보고서 | 결과·편향·재현 방법. `dev-docs/`에 커밋 | 1E-04, 1E-05, 1E-10 | ⏳ |
| 1E-07 | ADR: 2단계 우선순위 | 보고서 근거로 M2 범위 확정, ROADMAP 갱신 | 1E-06 | ⏳ |
| 1E-08 | 검사 제거가 드러나는 벤치 | 간접 인덱스(`a[b[i]]`) 등 LLVM이 스스로 범위 검사를 못 지우는 벤치를 추가하고 Rust safe/unchecked와 비교. 2026-10-24 기준선에서는 safe ≈ unchecked라 증명 기반 검사 제거의 효과가 보이지 않았다 | 1E-05 | ✅ `hist`·`gather`·`vadd` 추가. 결과: 검사 제거 이득 관찰 안 됨(safe ≈ unchecked) |
| 1E-09 | sieve·kadane 성능 차이 원인 분석 | BMB가 Rust보다 8~16% 느린 원인을 IR·어셈블리 비교로 특정(코드 생성 대 LLVM 버전 차이)하고, 코드 생성 원인이면 태스크로 등록 | 1E-05 | ✅ 원인: `bool` 원소를 메모리에 `i1`로 저장 → 바이트로 변경, sieve 1.20→1.03, `bool_arrays_are_bytes_in_memory` |
| 1E-10 | 실제 모델 측정 실행 | `harness.py run --agent anthropic:MODEL`로 과제 15개를 1회씩 실행하고 결과 커밋. 유료 API라 **사용자 머신에서 실행**(2026-10-24 결정, 절차는 `bench/m1/README.md`) | 1E-03 | 🚫 사용자 실행 대기 |
| 1E-11 | 측정 중 발견한 언어 마찰 기록 | 과제·벤치 작성 중 드러난 마찰(명세 정수 혼합 → ADR-0022, 불변식의 `old` 허용, `#[uses]` 누락 반복)을 측정 보고서에 포함 | 1E-06 | ⏳ |
| 1E-12 | 결과 요약기 | `harness.py summarize RESULTS.jsonl`이 ADR-0021 §4 지표 표를 마크다운으로 출력. 대본 에이전트 결과로 테스트 | 1E-04 | ✅ `harness.py summarize`(파일마다 한 열), 게이트에서 oracle 결과로 확인, `bench/m1/README.md` |
| 1E-13 | 보고서 초안 | 모델 실행과 무관한 부분(절차, 과제 편향, Rust 기준선, 마찰 기록)을 `dev-docs/measurements/`에 먼저 작성하고 모델 수치 자리를 비워 둠 | 1E-05 | ✅ `dev-docs/measurements/m1.md`(초안, §3 결과 자리 비움) |

## 통과 기준 추적

ROADMAP의 통과 기준마다 그것을 확인하는 태스크다. 빈칸이 있으면 태스크를 만든다(AUTONOMY.md §3.2). 해당 태스크가 모두 ✅이고 단계 종료 감사를 마쳐야 단계를 닫는다.

| 단계 | 통과 기준 | 확인 태스크 |
| --- | --- | --- |
| 1A | 진단 코드 전부 코퍼스에서 발생, 결정론, E02xx 수정 제안 | 1A-04, 1A-05 ✅ |
| 1B | 1. `tests/check/ok` 전부 증명 | 1B-12 |
| 1B | 2. 틀린 코퍼스 전부 거부 + 반례·원인 | 1B-10, 1B-12 |
| 1B | 3. Z3 부재·`rlimit` 최소에서 실패 | 1B-13 |
| 1B | 4. SMT 질의 결정론 | 1B-13 |
| 1C | 1. 예제 실행·기대 출력 | 1C-05 |
| 1C | 2. 증명 유무에 따른 검사 제거·잔존 | 1C-06 |
| 1C | 3. assume은 증명된 사실만, IR 주석으로 추적 | 1C-04, 1C-06 |
| 1D | 1. 맵·질의 결정론 | 1D-12 |
| 1D | 2. 파일 배치 변경에도 맵·해시 불변 | 1D-03, 1D-12 |
| 1D | 3. 트랜잭션 거부·증명 미완료 표시 | 1D-08 |
| 1D | 4. 정규형 왕복 | 1D-02 |
| 1E | 1. 하네스·과제·프롬프트·결과 커밋, 재현 가능 | 1E-02, 1E-03, 1E-06 |
| 1E | 2. 과제 편향 명시 | 1E-06 |
| 1E | 3. 2단계 우선순위 ADR | 1E-07 |

## 2단계 이후 후보

`docs/ROADMAP.md`의 2~4단계 표가 후보 목록이다. 1E-07에서 우선순위가 정해지면 태스크로 쪼개 여기에 올린다.

1단계 태스크가 모두 막혀 당겨 온 것(AUTONOMY.md §3.1). 언어 범위를 바꾸지 않는 도구 작업만이다.

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2D-01 | 해시 키 증명 캐시 | 정의의 본문 해시와 의존 계약 해시를 키로 증명 결과를 저장해 바뀌지 않은 정의는 Z3를 부르지 않음. 캐시 유무로 판정이 같음을 테스트 | 1D-03, 1B-11 | ✅ `cache.rs`, `bmb verify --cache DIR`, `tests/cache.rs`(코퍼스 출력 동일, 재실행 시 증명된 의무는 해법기 생략, 펼친 정의가 바뀌면 무효). `run/search.bmb` 1.0초→0.02초. MCP·`build`에는 아직 연결 안 함 |
| 2D-02 | 사전 영향 분석 질의 | 정의의 계약을 바꾸면 다시 증명해야 하는 정의 집합을 반환하는 `impact` 질의. CLI·MCP 노출, 결정론 테스트 | 1D-06 | ✅ `query impact DEF [body or iface]`, `tests/impact.rs`(본문 변경마다 SMT 스크립트가 바뀐 정의 ⊆ 보고 집합). MCP 설명문은 1E-10까지 유지 |

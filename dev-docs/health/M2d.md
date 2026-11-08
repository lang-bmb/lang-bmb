# M2-d 종료 감사 (표현 확장)

> HEALTH.md §6 국면 종료. 2026-11-08. 국면 정의는 `docs/ROADMAP.md` "트랙과 국면"의 M2-d: 증명 함수 `proof fn`(T1-18, ADR-0062), 2U 문자열과 JSON 파서(T4-05), `f64`(T2-02 → T2-03, ADR-0061)와 n-body·spectral-norm(T3-04), 2M 메서드 구문(ADR-0051). 백로그 2P-04.

## 판정: M2-d 통과

통과 기준 넷을 모두 충족한다. 기준 밖에 남는 것은 T3-04의 공식 Rust 제출과의 시간·크기 비교 하나다(네트워크 정책으로 막힘, 아래).

## 통과 기준

| 기준 | 상태 | 근거 |
| --- | --- | --- |
| 1. 각 페이즈 통과 기준 | 충족 | 아래 표 |
| 2. JSON 파서가 JSONTestSuite `y_`·`n_`를 통과한다 | 충족 | `std::json`(4B-02 (h), T4-05 합침): 고정 커밋 `1ef36fa`의 `y_` 95개 모두 받고 토큰이 Python `json`과 같으며, `n_` 188개 모두 거부(`scripts/json_suite.py`, `dev-docs/measurements/json-suite/2026-11-08.jsonl`). 전부 증명(`dev-docs/health/4B.md`) |
| 3. Benchmarks Game 둘(n-body·spectral-norm)이 더 들어온다 | 충족 | `bench/game/nbody.bmb`·`spectralnorm.bmb`가 전부 증명되고(`compiler/tests/verify.rs` `game_programs_are_proved`) 출력이 벤치의 기대 출력과 손으로 쓴 Rust 기준(`bench/game/ref/`)과 같다(`compiler/tests/codegen.rs` `game_programs_match_their_rust_references`) |
| 4. `proof fn`으로 판정기 미결정 사례(`vc-vv0039`)의 같은 뜻 명세가 증명된다 | 충족 | `compiler/tests/verify.rs` `the_vv0039_equivalent_contract_is_proved_with_a_lemma`: 판정기의 텍스트 그대로는 미결정(재현), 보조 정리를 부르면 증명(T1-18) |

### 기준 1: 페이즈별

| 페이즈 | 상태 | 근거 |
| --- | --- | --- |
| T1-18 `proof fn`(ADR-0062) | 충족 | 백로그 T1-18 ✅(P1~P5; P6은 T1-24): 표면·검사, 인코딩, 지움(`ghost_code_leaves_no_trace_in_the_ir`), 퍼저 `soundness_specs`, 변이체 다섯을 대상 시험이 잡음 |
| 2U 문자열(ADR-0050) | 충족 | `dev-docs/health/2U.md`(종료 감사 2026-11-05). 그 뒤 2U-06(UTF-8 사용 지점 사실), 2U-07, 1B-55·1B-56(텍스트 건전성 결함 둘을 찾아 고침) |
| `f64`(T2-02 → T2-03, ADR-0061) | 충족 | T2-03 F2~F7 완료(백로그). 이번 감사에서 남았던 F6 변이체 셋을 게이트 전체로 돌렸다: `c-fadd-as-fsub`(`test soundness_float`·`test codegen`·`health`가 잡음), `c-olt-as-ole`(`test codegen`·`test soundness_float`), `v-f64-lt-interpreted`(`test verify`), 셋 다 잡힘(`dev-docs/measurements/gate-power/results.jsonl`) |
| 2M 메서드 구문(ADR-0051) | 충족 | `dev-docs/health/2M.md`(종료 감사 2026-11-07) |
| T4-05 JSON 파서 | 충족 | 기준 2 |

## 남는 것

- **T3-04의 시간·크기 비교.** 공식 Rust 제출(benchmarksgame-team.pages.debian.net, salsa.debian.org)이 이 환경의 네트워크 정책에 막힌다(2026-11-08에도 `connect_rejected`). BMB 쪽(증명·출력)은 끝났고 비교 행은 조용한 머신에서 공식 출처로 채운다. M2-d 기준 3("들어온다")의 밖이다.
- **게이트 검출력 표의 빈칸.** 변이체 99개 중 전체 게이트 기록이 없는 것이 이 감사 전 46개였다. F6 셋을 돌려(셋 다 잡힘) 43개가 남았다. 한 변이체에 약 14~21분이라 나머지는 후속 `1B-57`이다. 각 변이체의 대상 시험 기록은 그것을 더한 태스크에 있다.

## 다섯 축(2026-11-08 기준선)

| 지표 | 값 |
| --- | --- |
| V3_unproved_defs | 0 |
| V4_rlimit_total | 13,721,465 |
| V5_rlimit_headroom | 0.1244 |
| V7_unstable_queries | 0.0002 |
| V10_library_rlimit_total | 24,512,277 |
| V12_sat_check_undecided | 208 |
| P1_runnable_programs | 73 |
| P3_errors_with_help | 0.9118 |
| S4_stage0_rust_lines | 49,071 |

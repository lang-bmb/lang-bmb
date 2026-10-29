# 백로그

> 갱신: 2026-10-25 · 단계와 통과 기준은 `docs/ROADMAP.md`.
> 태스크는 완료 기준이 측정·확인 가능한 크기로 쪼갠다. 완료로 바꿀 때는 근거(커밋·테스트·측정)를 적는다.
> 선택·도출·결정 권한 규칙은 `dev-docs/AUTONOMY.md`.

**상태:** ✅ 완료 · 🔄 진행 · ⏳ 대기 · 🚫 막힘

**ID 규칙:** `단계-번호`(예: `1B-03`). 번호는 재사용하지 않는다. 버린 태스크는 지우지 않고 ~~취소선~~과 사유를 남긴다.

## 지금 할 일

**승인(2026-10-28, 방향성 논의):** 실행 순서를 트랙 T0~T5로 바꾼다(ADR-0058, 아래 "트랙" 절과 ROADMAP "트랙과 국면", 논의 기록 `dev-docs/reviews/direction-2026-10-28.md`·`collaboration-2026-10-28.md`). 측정은 CLI 파일럿을 먼저 한다(정식 자동 경로 `T0-03`은 승인 대기). 언어 후보 셋은 초안 ADR-0059~0061(제안)이고 채택은 각각 승인이다. 새 언어 페이즈는 T3·T4의 수요나 T1의 판정이 열 때 연다. 아래 웨이브 표(W0~W6)는 2026-10-28까지의 기록이다.

### 실행 계획(2026-10-28, 사람 지시: 태스크 도출·국면 구성·우선순위 재정렬)

2단계의 남은 일을 국면 M2-a~M2-e로 묶는다(ROADMAP "트랙과 국면"). M2-a와 M2-b는 함께 진행한다. 국면 안에서는 아래 우선순위, 같은 순위 안에서는 AUTONOMY §3.1 규칙을 따른다.

| 순위 | 국면 | 태스크(순서) | 왜 이 순서인가 |
| --- | --- | --- | --- |
| 1 | M2-a | `T3-03` | BMB 팔 파일럿(BVB 43/45)을 해석할 유일한 비교. Verus 0.2026.10.04 설치 확인(2026-10-28) |
| 2 | M2-a | `T1-14`, `T1-15` → `2D-19` → `T1-12` | 값싼 결정론 근거. 파일럿 실패 셋이 모두 명세 강도라 원인부터. `2D-19`의 함의 질의가 교환 관계의 정제 판정에 쓰인다 |
| 3 | M2-b | `2N-02` → `2N-03` → `2N-04` → `2N-05`(= `T4-01`) → `T4-04`, `T4-03`. 병행 `T4-02`, `T3-01` | 비트 연산이 라이브러리 셋을 연다. UTF-8과 Benchmarks Game 둘은 지금 언어로 된다 |
| 4 | M2-a | `T1-05`, `T1-04`, `T1-13` → `T0-04` | 새 가설의 측정 도구와 2차 파일럿 |
| 5 | M2-a | `2A-14e`(배경) | 측정 머신이 빌 때. CPU 팔이 무거운 일과 겹치면 안 된다 |
| 6 | 국면 끝 | `2P-01`(M2-a), `2P-02`(M2-b) 종료 감사 | 통과 기준 추적 표 |
| 7 | M2-c·M2-d | `T1-18`(ADR-0062), `T1-10`(ADR-0059) 병행, `T2-02`(✅) → `2N-03` → `T1-11`(ADR-0060), `T2-03`(ADR-0061) | ADR-0059~0062 채택(2026-10-28). 순서는 ROADMAP 우선순위 6 |
| 대기 | — | 승인: `T0-03`, `T1-16` 채택, 협업 논의 Q2, 정식 측정의 쓰기 표면, `sha2` 비교(T4-01). 승인 없이 할 수 있는 준비: `T1-16` 초안 | 결정이 오는 대로 국면에 넣는다 |
| 미룸 | — | `2R-03`·`2R-05`(모델 측정), 2E·2F(1E-07 대기), 3단계 이후 | M2-e(2H) 또는 승인 뒤 |

**승인(2026-10-26, "Ok, 승인"):** 판단 기준(ADR-0045, `dev-docs/JUDGMENT.md`)으로 기존 결정을 소급 점검하는 태스크(2V) 등록과, 그 결과로 W5a를 다시 짜는 것을 승인. 점검 결과는 `dev-docs/reviews/2V-01-judgment-retro.md`. 컴파일러 층 수정(2V-02~2V-05)은 자율, 언어 층(2V-06~2V-12)은 ADR 초안까지 자율이고 채택은 승인이다. 직전 W5a 제안(단형 맵, 아레나 전제의 파일럿)은 철회한다(점검 R12).

**승인(2026-10-25, "모두 차례로 진행"):** 2B·2C 착수를 1E-07(실제 모델 측정) 전에 승인. 근거: 잠정 순서와 셀프호스팅 권고안 모두 2B→2C로 시작한다. 2E·2F·2G는 여전히 1E-07 뒤. 진행 순서는 아래 실행 계획(2026-10-25 재정렬, 사람 지시)이 대체한다.

**승인(2026-10-25, "권장옵션 채택, 자율 실행/검증 승인"):** 2G(+2S) 착수를 1E-07 전에 승인(W4의 앞부분). 2G 착수 때 제안 ADR-0031을, 2S 착수 때 2S-01 초안을 갱신·채택한다(2B·2C의 0026·0028과 같은 방식). 2E·2F와 그 ADR(0029·0030)은 여전히 1E-07 뒤. 웨이브 W0~W4의 자율 실행과 검증(게이트·측정으로 완료 판정)을 승인.

### 실행 계획(2026-10-25 재정렬, 사람 지시)

정렬 원칙: (1) 가장 많이 막는 것은 실제 모델 데이터다. 사용자 실행 한 번이 판정을 최대로 채우게 실행 전에 열을 모두 준비한다. (2) 언어를 넓히기 전에 증명 게이트의 검출력과 안정성 기준선을 잰다(2B가 `verify.rs`·`codegen.rs`를 바꾼다). (3) 셀프호스팅 임계 경로(2B→2C→2G→2S→4B·4C, 4A-01)를 앞당긴다. (4) 첫 데이터가 가치를 정할 새 하네스는 데이터 뒤로 미룬다. 웨이브 안에서는 AUTONOMY §3.1 규칙을 따르고, 큰 태스크는 착수할 때 쪼갠다.

| 웨이브 | 목적 | 태스크(순서) | 끝나는 조건 | 권한 |
| --- | --- | --- | --- | --- |
| W0 | BCB 판정(배경 실행) | `2A-14e` → `2A-14f` | 반증 조건 1~4 판정 커밋 | 자율 |
| W1 | 한 번의 사용자 실행으로 최대 판정 | `1D-24`, `2H-04`, `2A-12`, `2D-21` → `2H-06`, `INF-14` | `INF-14 --dry-run`이 m1·m1b·BVB·BRB를 대본 에이전트로 끝까지 돌고 G2·G3·G5·G8·H-ORACLE·H-SPEC·H-FAMILIAR 열을 모두 냄(게이트) | 자율 |
| W2 | 신뢰 기반 기준선 | `1B-38` → `1B-40` → `INF-11` → `1B-41`, `2D-18`, `INF-12`, `INF-13` | 검출률·불안정 비율(V7)·벤치 부하 지표가 래칫에 들어감, 놓친 결함마다 시험 | 자율(`1B-39`는 승인) |
| W3 | 언어 1: 데이터 타입·변환 | `2R-04d` → `2B-11` → `2B-03`~`2B-09` → `2B-12` → `2R-04c` → `2C-05` → `2C-02`~`2C-04` → `2C-06`. 병행 `2R-04a`·`b`·`e`~`h`(가벼운 일), 초안 `2S-01`, `4B-03` | 2B·2C 통과 기준과 HEALTH §5(실행 프로그램 각 2개 이상) | 승인됨. 2C 완료(2C-06), 2B는 기준 4(2B-10, 사용자 트랙)만 남음 |
| W4 | 언어 2: 메모리·문자열·효과·탈출구 | `2G-02`~`2G-06` → `2S-02` → `2F-02`·`2F-03` → `2E-02`~`2E-04`(4A-01 권고 순서, 1E-07이 다르게 정하면 따름). 병행 `4C-03`, `4A-03`, `4A-04`, `5C-03` | 각 페이즈 통과 기준과 HEALTH §5 | 2G·2S 승인됨(2026-10-25), 2F·2E는 1E-07 뒤. 2G·2S 완료(`dev-docs/health/2G.md`·`2S.md`) |
| W4b | 판단 기준 소급(ADR-0045)과 언어 완성 | `2V-01`~`2V-05`·`2V-13`~`2V-16` 완료 → `2V-18`(초안 0052·0053 채택, 승인) → 페이즈 `2T` → `2O` → `2K` → `2U` → `2N` → `2M`(셀프호스팅 임계 경로 순, 각 페이즈는 ADR → 검사 → 인코딩 → 코드 생성 → 코퍼스 → 퍼저 → 실용·감사). 병행 `1B-45`, `1B-43`, `2A-14e`(배경) | 각 페이즈 통과 기준과 HEALTH §5 | 2T~2M 승인됨(2026-10-26). 2V-14·2V-16 채택은 승인. 2T 완료(2026-10-27, `dev-docs/health/2T.md`) |
| W5 | 자립 | `4B-01`·`4B-02`, `4C-01`·`4C-02`, `4D-01` → `4D-02`~`4D-06` → `4E-01` → `4F-01`, `4G-01` | 4단계 통과 기준(고정점, 자기 증명, 씨앗 재현) | ADR·동결·은퇴는 승인 |
| W6 | 성숙·생태계 | `5A-01` → `5A-02`, `5B-01`, `5C-01`·`5C-02`·`5C-04`, `5D-01`~`5D-03`, `5E-01` | 5단계 통과 기준 | 정책·릴리스·생태계는 승인 |

**사용자 트랙(병행, 웨이브와 무관하게 언제든):** `INF-14`가 준비되면 `1E-10`·`2A-06`(같은 실행) → 결과가 커밋되는 즉시 `1E-06`·`1E-11` → `1E-07`·`2A-16`(승인) → `2R-03` → `2R-06`(승인), `2D-12` → `2A-07`, `2D-13`, `2B-10`·`2H-01` → `2H-02`(3단계 범위). 사람 참여·네트워크: `2H-05`, `2R-05`, `2A-15`, `0-04`, `0-05`.

**데이터 뒤로 미룸:** `2D-19`·`2A-13`(H-GAME), `3A-06`·`3A-07`(H-SCALE·H-REUSE), `3A-08`, `2D-17`. 첫 실제 모델 결과(사용자 트랙)가 나오면 웨이브에 다시 넣는다. 3단계의 나머지(3B~3E)는 `2H-02`가 정한다.

진행 중: `2A-14` BCB. `2A-14a`~`2A-14d` 완료, `2A-14e` N=1,000 측정 실행 중. `body-only-1k` Rust 팔 완료(BMB 컴파일러를 쓰지 않아 그대로 씀), B 팔은 `2D-22`(ADR-0043, 승인) 컴파일러로 다시 고정해 잰다. 그다음 `2A-14f`(N=10,000·판정). 발견: `1D-24`. `2D-20`(MCP 위협 모델, 링크 탈출 수정) 완료.

**측정 중 운영:** 측정과 같은 머신의 컴파일·게이트는 CPU를 재는 팔(`rust`·`b1`·`b0`)을 오염시킨다. `defects` 팔은 검출 여부만 판정에 쓰고 해법기 한도가 결정론적(`rlimit`)이라 무거운 일과 겹쳐도 된다. 문서·Python 일은 측정과 함께 하고, 무거운 태스크는 `suite`를 팔·이력 경계에서 멈추고 한 뒤 이어 돌린다(재개 지점이 결과 머리 줄에 남는다). `2A-14f`에서 조건 1이 반증되면 원인 조사 태스크(차가운 증명 비용 분해)를 도출 규칙 "측정 결과"로 만든다.

트레이드오프 원장(ADR-0041, 사람 지시 2026-10-25): 테스트는 실행 최적화가 아니라 구조적 제거로 주장한다. 새 목표 G8(테스트 코드, `2D-21`→`2H-06`), G9(바이너리 크기, `health.py` B1 래칫). 수용 비용 관측 `INF-13`.

전제 수정(ADR-0040, 승인 2026-10-25): 모델 없이 자율로 할 것은 `2A-14` BCB(누적 검증 비용, G7), `1B-40` 솔버 안정성, `1B-41` 편집 국소화, `2D-19` 명세 약화 감지, `2A-13` BMB-Impossible, `2H-04` 오라클 지표, `3A-07` 중복 지표, `2D-20` 위협 모델. `2A-15`·`2H-05`는 사용자.

페인포인트 조사(2026-10-25): 네 가설 방향은 지지되나 근거가 "테스트 비용"이 아니라 "테스트 통과≠정답, 리뷰 확장 불가"에 있고, 다음 병목은 명세 타당성. 반영은 `2A-11`(전제 수정은 승인).

지속 관리(ADR-0039, 사람 지시 2026-10-25): 지표·래칫 `INF-09`, 푸시 장치 `INF-10` 완료. 다음 자율 후보는 `INF-11`(벤치 부하 증명 비용), `1B-38`(증명기 검출력), `2D-18`(고칠 길 없는 오류), `INF-12`(성능 회귀). `1B-39`는 승인 대기. 운영 규칙은 `dev-docs/HEALTH.md`.

페이즈 구성은 `docs/ROADMAP.md`(2026-10-24 재구성). 1단계 1F(신뢰성 보강)는 완료: `1A-10`~`1A-13`, `1B-17`~`1B-35`, `1D-14`~`1D-21`, `INF-07`, `1E-12`, `1E-13`.

자율 태스크(2026-10-24): `2A-01`~`2A-05`(과제 집합 `m1b`), ADR 초안 `2B-01`·`2B-02`·`2C-01`·`2E-01`·`2F-01`·`2G-01`(제안 0026~0031), 도구 `2D-09`~`2D-11` 완료.

완료(2026-10-24): `1B-36`, `1D-22`, `1D-23`, `2D-14`~`2D-16`, `INF-08`, `3A-01`~`3A-04`.

미션 목표(ADR-0034, 승인)로 만든 벤치마크 BVB(`2A-08`·`2A-09`), BSB(`2A-10`), BRB(`2H-03`), BNB(`3A-05`)는 기반 완료. 실제 모델 실행은 사용자 머신.

부트스트랩·성숙(ADR-0037, 사람 지시 2026-10-25): 지금 자율로 할 수 있는 것은 `4A-01` 격차 분석, `4A-02` 포팅 지도, `5A-01` 레퍼런스 초안, `5C-03` 디버그 정보.

문법 재평가(2R, 사람 지시 2026-10-25): `2R-01` 평가 저울 ADR → `2R-02` 현행 문법 1차 평가 → `2R-04` 대안 오프라인 비교는 자율. `2R-03`은 1E-10 뒤, `2R-05`는 사용자 실행, `2R-06` 결정은 승인. 2B 표면(ADR-0026) 채택 전에 `2R-02`까지 마친다. `2D-12`·`2D-13`은 측정 뒤.

승인이 필요한 것: 제안 ADR 0026~0031의 채택과 2B~2G 구현 태스크("승인" 표시). 1E-07로 순서가 정해진 뒤 페이즈 단위로 승인을 받는다.

측정 결과에 묶인 것:

1. `1E-10` 실제 모델 측정 — 사용자 머신에서 실행(`bench/m1/README.md`). 같은 방식으로 `m1b`도 실행할 수 있다(`2A-06`, `bench/m1b/README.md`)
2. 결과가 커밋되면 `1E-06` 보고서, `1E-11` 마찰 기록, `1E-07` 2단계 우선순위 ADR

## 트랙 (ADR-0058, 2026-10-28)

### T0 측정 경로

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| T0-01 | CLI 파일럿 실행기 | `bench/pilot/`: Claude Code CLI를 피험 에이전트로, 동결 과제·MANIFEST·`prompt.md`(시스템 프롬프트 전체)·`bmb mcp` 도구만(내장 도구 없음, 저장소 밖 빈 디렉터리)·40회 한도·과제당 한 세션·하네스 판정(`judge_write`·`spec_kill`·`judge_navigate`). 행마다 `"pilot": true` | — | ✅ `bench/pilot/cli_pilot.py`·`mcp_proxy.py`, 스모크 `w01-clamp` 성공(도구 7회, MCP만) |
| T0-02 | 파일럿 실행: m1·m1b·BVB | 세 집합을 한 번씩 돌려 `bench/pilot/results/`에 결과와 요약을 커밋. 결과는 가설 등록부의 "파일럿" 상태에만 쓴다 | T0-01 | ✅ `bench/pilot/results/2026-10-28-*`. 1차는 큰 도구 결과를 보지 못하는 파일럿 결함(`T1-15`) 아래라 기록으로만 둔다. 2차(`*-r2.jsonl`): m1 15/15, m1b 37/37(BSB 186/186), BVB 42/45. 1차의 "실패는 모두 명세가 약해서"라는 해석은 틀렸다(`T1-14`, `bench/pilot/README.md` 정정) |
| T0-03 | 정식 자동 측정 경로 | API 키를 시크릿으로 두고 `bench/run_models.py --budget`로 m1·m1b·BVB·BRB를 실행해 결과 커밋(`1E-10`·`2A-06`의 대체 경로) | T0-01 | ⏳ 승인(유료 API, 시크릿 설정은 사람) |
| T0-04 | 2차 파일럿: 새 과제 집합 | H-NAME 난독화 쌍둥이(`T1-04`)와 H-SPLIT 분업판(`T1-13`)을 `T0-02`와 같은 조건으로 실행해 `bench/pilot/results/`에 커밋 | T1-04, T1-13 | ⏳ |
| T0-07 | BRB 파일럿(H-ORACLE, G5, G8) | `bench/pilot/brb_pilot.py`·`rust_mcp.py`: 동결 하네스를 라이브러리로 두 비교군 24과제, `bench/brb/harness.py summarize`로 표 | T0-01 | ✅ 큰 모델(`claude-opus-5-5`): 두 쪽 24/24, H-ORACLE 판정 불가(Rust 통과 & 숨은 실패 0), G8 0줄 대 834줄, 비용 $3.56 대 $1.45(총 토큰 3.6배). 작은 모델(`claude-haiku-5-5`): BMB 21/24 대 Rust 24/24, H-ORACLE 판정 불가(0/21 대 0/22), G8 0줄 대 1,277줄, 총 토큰 11.9배. BMB 실패 셋(t10·t16·t24)은 모두 비선형 산술이고, 40회 한도에 걸렸다. 같은 모델의 `write` 판(`brb_pilot.py --variant bench/brb-write`)은 24/24였으므로, 원인은 트랜잭션 절차의 호출 예산이다(처음의 ADR-0062 귀속은 정정, `bench/pilot/README.md`). 원자료 `bench/pilot/results/2026-10-28-brb-*`, 요약 `bench/pilot/README.md` BRB 절. H-ORACLE을 가르려면 더 어려운 과제가 필요하다(T0-08) |
| T0-08 | H-ORACLE 변별 과제(BRB 어려운 판) | 두 크기 모델 모두 Rust 쪽 "테스트 통과 & 숨은 실패"가 0이라 H-ORACLE을 가르지 못했다(T0-07). 경계·오버플로·드문 입력에서 테스트가 놓치기 쉬운 과제를 새 동결 판으로 만들고, 숨은 판정·기준 해법·약한 해법을 함께 커밋한다. 과제와 프롬프트는 먼저 동결하고 한 번씩 돌린다 | T0-07 | ⏳ |
| T0-09 | H-ENCAP BRB 확장(ADR-0060 결과 6) | 자료 구조 수정 과제(정렬 집합, 고리 버퍼, 구간 목록, 최소 힙)를 BRB 새 판으로 만들어 동결: 과제마다 숨은 불변식 판정, BMB 팔(타입 불변식)과 Rust 팔(private + 테스트)의 기준·약한 해법을 CI로 확인. 실행은 파일럿 조건(T0-01)으로 한 번, "통과 & 숨은 불변식 위반" 비율을 비교. 반증 조건: BMB 팔에서 위반이 게이트를 통과 | T1-11, T0-07 | ⏳ |
| T0-05 | 판정 거부 이유 기록 | 명세 강도 검사가 거부할 때 `refuted`(반례)와 `undecided`(해법기 미결정)를 행에 남김. 판정은 그대로 | T1-14 | ✅ `bench/m1/harness.py` `spec_rejection`·`spec_rejections`, `test_harness.py` `SpecRejection`·`AlsoCheck` |
| T0-06 | 판정기 미결정의 분리 보고 | 요약(`summarize`, `compare`)에 명세 강도의 "반박"과 "미결정"을 따로 세는 열. 미결정이 같은 뜻 명세일 때 측정이 언어가 아니라 판정기를 재는 문제를 드러낸다 | T0-05 | ✅ `bench/m1/harness.py` `summarize`: "명세 강도 거부: 반박(반례)"·"미결정(해법기)" 두 행, 과제 표의 실패 칸에 이유. 시험 `SummarizeRejections` |

### T1 가설

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| T1-01 | H-ID 현행 비용: 이름 바꾸기 파급 | 정의 하나의 이름만 바꿨을 때 해시가 바뀌는 참조자 수 | — | ✅ `dev-docs/experiments/rename-ripple/`: 412건 중 356건(86%), 계약 해시 377 |
| ~~T1-02~~ | ~~H-PAR 동시 편집 재생~~ | 정의 단위 충돌률은 `T1-12`(H-COMMUTE) 실험의 한 열 | 2A-14d | 취소: `T1-12`에 합침(협업 논의 2026-10-28) |
| T1-03 | H-SLICE 슬라이스 크기 축 | BNB 1k·3k·10k에서 정의별 슬라이스 크기 대 파일 읽기 크기의 분포와 기울기 | 3A-05 | ⏳ |
| T1-04 | H-NAME 난독화 쌍둥이 과제 | m1·BVB 작성 과제의 알파 개명판·오도하는 이름판을 생성기로 만들어 동결, 기준 해법·약한 대조군 CI. 실행은 T0 | — | 🔄 도구 완료: `bench/hname/gen.py`가 `bench/m1b-alpha`(불투명 이름·중립 요약)·`bench/m1b-mislead`(같은 종류끼리 이름 뒤섞음)를 만들고 동결, 게이트 `bench hname`(생성 일치, 기준 해법 37/37, 약한 대조군 25/25 실패). 파일럿: 알파 37/37(BSB 186/186, 비용 $7.69 대 m1b $6.97). 남은 것: 오도판 파일럿, Rust 쌍둥이 쪽(H-NAME의 비교 절) |
| T1-05 | H-VIEW 투영 크기 | 같은 작업 공간의 투영(정규형, 공백 최소화, 계약만, 시그니처만)별 바이트·토큰 표. 성공률은 T0 | — | ✅ `dev-docs/experiments/view-size/`: 네 작업 공간, Claude 토큰. 공백 최소화 −13~15%(같은 해시), L1은 코드의 2.0~3.0배, 해시·위치·빈 필드를 빼면 0.68~0.93배. 후속 `2D-23` |
| T1-06 | H-EDIT·H-GRAIN 편집 표면 | 같은 과제를 세 도구 표면(정의 단위 트랜잭션, 파일 diff, JSON AST)으로 푸는 하네스 변형. 실행은 T0 | T0-01 | 🔄 `bench/edit/gen.py`의 `bench/m1b-write`·`bench/bvb-write`·`bench/brb-write`(BRB는 BMB 쪽 프롬프트와 `set.json`만, `brb_pilot.py --variant`), 게이트 `bench edit`(m1b·BVB 판의 기준·약한 해법). 파일럿: 정의 단위 한 번 쓰기가 트랜잭션 여러 호출보다 m1b는 같은 37/37에서 비용 −22%, BVB는 42/45 → 44/45에서 비용 −24%·차례 452 → 143(Verus 44/45·156차례), BRB 큰 모델은 24/24에서 비용 −31%·Rust 대비 총 토큰 3.6 → 1.56배, 작은 모델은 21/24 → 24/24(`bench/pilot/README.md`). 남은 것: 파일 diff·JSON AST 표면 |
| T1-07 | H-UNIT 도출 목차 | 호출 그래프 군집으로 L0를 도출하는 실험 맵, BNB 탐색 과제를 두 판(선언 단위, 도출 군집)으로 | 3A-05 | ⏳ |
| T1-08 | ADR-0059 초안: 해시 참조 | 정체성 해시, 이름은 색인, 키와 증명 캐시를 이름 없이, `rename`·`expect` 트랜잭션 | T1-01 | ✅ `dev-docs/adr/0059-hash-references-names-as-metadata.md`(제안) |
| T1-09 | ADR-0060 초안: 타입 불변식 | `#[invariant(p)]`, 새 값은 가정·잊는 값은 의무(E0823), 가시성 없이 | — | ✅ `dev-docs/adr/0060-type-invariants.md`(제안) |
| T1-10 | ADR-0059 채택과 구현 | ADR 결과 표의 다섯 태스크(정체성 해시, 키·캐시, 맵, 트랜잭션, 측정). rename-ripple 참조자 수 0 | T1-08 | 🔄 하위 태스크 `T1-10a`~`T1-10f`. rename-ripple 참조자 0/424, 증분 감사 불일치 0, V7 그대로. 남은 것: `T1-10e`(질의 기호, 결정 필요), `T1-10f`의 재인코딩 0 |
| T1-10a | T1-ID-01 정체성 해시 | 이름 없는 출력기(`fmt::Printer::identity`), 참조 그래프의 강연결 성분과 색 정제 순서(`src/identity.rs`), 명목 타입·trait(`impl`을 타입·trait 텍스트에), `bmb hash`의 `id`. 퍼저: 이름 하나를 바꿔도 모든 `id` 불변, 리터럴 편집은 그 정의의 `id`를 바꿈 | T1-08 | ✅ `tests/fmt_fuzz.rs` `renames_keep_identities_and_edits_change_them`(코퍼스의 모든 이름으로 한 번 통과, `BMB_FMT_FUZZ_RENAMES`), `tests/fmt.rs` `identity_follows_what_names_resolve_to_not_the_names`. 구현 시 확인은 ADR-0059 |
| T1-10b | T1-ID-03 맵과 질의 | L1 `id`(타입·trait 포함), `duplicates`를 정체성으로, 알파 동치 중복 코퍼스. `id`를 빼면 맵 출력이 이전과 같음 | T1-10a | ✅ `tests/map.rs` `duplicates_groups_alpha_equivalent_definitions`, 코퍼스 `tests/duplicates/alpha.bmb`(9묶음: 매개변수·지역·레이블·필드 이름, 재귀·상호 재귀, 복제를 부르는 정의). 가독성: 코퍼스 85파일 4,783행이 `id`를 빼고 해시 값을 가리면 이전 컴파일러와 바이트 단위로 같음(`dev-docs/experiments/rename-ripple/readability.py`). BNB의 Rust 쪽 분석기(`bench/bnb/analyze.py`)도 호출을 대상의 정체성으로 적게 바꿨다: 동결 과제 `n26`·`n27`의 정답은 N=100·1,000·10,000에서 그대로이고, 전체 중복 묶음 수만 늘었다(5→10, 18→52, 109→472). 근거 묶음(2E)은 2E와 함께 |
| T1-10c | T1-ID-04 트랜잭션 | `rename`(경로 → 새 이름, 선언과 모든 참조, `id` 불변 확인·가로채기 거부)과 `expect`(경로 → `id` 전제조건) | T1-10a | ✅ `workspace::rename`, `tests/workspace.rs` `rename_rewrites_references_and_keeps_every_identity`·`rename_that_would_collide_or_capture_is_rejected`·`expect_rejects_a_changed_definition_only`·`rename_and_expect_parse_from_json`. `bmb apply`만, MCP 노출은 `T1-20` |
| T1-10d | T1-ID-02a 키·캐시: 노드 위치 | 저장하는 의무 위치를 구문 노드 위치로(`incremental::node_spans`), 증명 키에서 원문 행을 뺌, 파일 형식 v2. `with_audit` 불일치 0(배치·주석·이름 바꾸기 편집 포함) | T1-10a | ✅ `tests/incremental.rs` `layout_inside_a_definition_reencodes_nothing`(재인코딩 0), `edits_that_shift_spans_or_change_context_keep_outputs_exact`(배치·정의·매개변수·지역 이름 바꾸기 편집 추가, 감사 불일치 0). `verify.rs` 변경은 호출 두 줄 |
| T1-10e | T1-ID-02b 이름 없는 질의 기호와 키 | 질의 기호를 이름 없이(질의마다 처음 나온 순서의 정준 이름, 범례는 `--emit-smt` 밖 파일), 인코더의 이름순 순회(`Enc::branch` 병합 등)를 결합 순서로, 증분 키의 이름 행(범례)을 이름 없는 행으로(참조는 키 안 위치 번호), 증명 캐시 형식 버전, V7 재측정. 완료: rename-ripple `--stats`의 재인코딩·캐시 놓침 0, 감사 불일치 0 | T1-10d | ⏳ 결정 필요(ADR-0063 초안, 제안): ADR-0059 §4의 "`id` 접두사 기호"는 기호를 본문에 묶어 계약 방화벽을 깬다(ADR-0059 구현 시 확인). `verify.rs` 전반을 고치므로 ADR-0062 구현(`T1-18`)과 겹치지 않게 순서를 정한다 |
| T1-10f | T1-ID-05 측정 | rename-ripple 재실행(참조자, `--stats` 재인코딩·캐시 놓침), V7, BCB 이름 바꾸기·이동 재생(`T1-22`), H-PAR 입력 | T1-10c | 🔄 `dev-docs/experiments/rename-ripple/results.md`: 이름 바꾸기 424건, 참조자 `id`·본문·계약 해시 변화 0(이전 컴파일러로 같은 집합 367건·87%, 본문 626·계약 384). 재인코딩 847·캐시 놓침 3,560은 `T1-10e` 전까지 남음. V7 그대로(질의 텍스트 불변) |
| T1-11 | ADR-0060 채택과 구현 | ADR 결과의 여섯 태스크(검사, 인코딩, 증분 키, 코퍼스, 퍼저, BRB 확장) | T1-09 | ✅ 1~5(6은 `T0-09`). 검사·표면: `#[invariant(p)]`, E0254·E0255(`tests/check/err/invariants`, 11줄 모두 안내), 정규형·정체성(`#[uses]`처럼 술어를 `id`로), 맵 L1 서명과 슬라이스의 술어(`tests/map.rs` `a_type_shows_its_invariant_and_slices_carry_the_predicate`), `rename`(`tests/workspace.rs` `rename_follows_an_invariant_predicate`). 인코딩 `compiler/src/verify/invariants.rs`: 깊은 유효성(필드·페이로드·`Box`·원소 양화, 재귀 타입은 지상 펼침 두 단계, 매개변수는 비해석 술어 `'T.valid`), 가정(진입·호출 뒤·루프 머리), 의무 E0823(인자·출구·`&mut` 출구·루프 진입과 반복 끝, 반례에 경계의 값). `proof fn`·`proof` 블록은 명세처럼 가정·의무 없음. 코퍼스 `tests/verify/pass/invariants`(19정의), `tests/verify/fail/invariants`(경계마다 반례), 실행 `tests/run/ring`·`sorted_set`(rustc 쌍둥이와 출력 같음), `generic_stack` 이주(계약 절 7 → 6, `some`의 `len <= 8` 빠짐). 증분 `tests/incremental.rs` `invariant_edits_reencode_where_values_pass`(감사, 재인코딩 ⊆ `impact`, `values` 사유). 퍼저 `test soundness_data`(무작위 불변식, `q: P` 매개변수, E0823 반례 재실행). 검출력 변이체 셋(`v-inv-mut-exit-dropped`·`v-inv-loop-entry-dropped`·`v-inv-arg-dropped`) 모두 `test verify`가 잡음. 측정과 열린 질문 2·3은 ADR-0060 "구현 시 확인" |
| T1-12 | H-COMMUTE 동시 편집 재생 | BCB 이력의 같은 기준 커밋 위 편집 쌍·k개 묶음 표본마다 `git merge-file` 충돌, 깨끗한 텍스트 병합의 BMB 증명·Rust 쌍둥이 테스트(거짓 병합), 교환 판정(쓰기 집합 ∩ 상대 읽기 집합, 계약 변경은 정제 판정), 정의 단위 병합의 증명과 재증명 비용. 교환 가능 판정의 거짓 병합 0이 건전성 조건. 결과 `dev-docs/experiments/` | 2A-14d | ✅ `dev-docs/experiments/commute/run.py`, `results.md`: 네 설정 116쌍. 독립 판정 77쌍의 병합이 모두 증명(건전성 위반 0), 정의 단위 쓰기 충돌은 텍스트 충돌과 같은 수준, git 거짓 병합 0/95(합성 이력에 의미 충돌이 없음), 충돌 판정 37쌍 중 18쌍은 보수적. 후속 `T1-19` |
| T1-13 | H-SPLIT 계약 우선 분업 과제 | m1b·BRB 과제를 계약 고정 + 정의 분할 판으로 만들어 동결, 기준 해법의 분할 조각을 따로 증명한 뒤 합친 결과도 증명됨을 CI로 확인. 실행은 T0 | — | ⏳ |
| T1-14 | 파일럿 실패 원인: 명세 강도 | `w09-spread`·`vb-mbpp-62-smallest`·`vc-vv0039-max-of-list`의 에이전트 계약과 숨긴 계약의 차이를 기록하고, 지금 도구(`bmb mutate` W0904, `bmb lint`, 반례)가 커밋 전에 잡았을지 재현. 귀속 층(도구 노출·도구 결함·과제·모델)과 조치 태스크 | T0-02 | ✅ `bench/pilot/README.md` "실패 분석": 분석할 수 있는 넷 모두 판정기 미결정(`spec_rejections` = `undecided`), 에이전트 계약은 숨긴 계약과 같은 뜻. 원인 둘: 재귀 술어 → 한정 기호 함의의 귀납(ADR-0062 제안), 상수 나눗셈·나머지의 비선형 산술(`1B-47`). 판정기에 거부 이유 기록(`T0-05`) |
| T1-15 | 파일럿 실패 원인: 탐색 한도 | `m1b/n12-io`가 40회 한도에 닿은 질의 경로를 같은 조건으로 다시 기록(호출 인자까지)하고, 정답에 필요한 최소 질의 수와 비교. 귀속 층과 조치 태스크 | T0-02 | ✅ 원인은 파일럿 실행기: CLI가 큰 도구 결과(`map L1` 59,442바이트)를 파일로 옮기고 앞 2KB만 보여 주어 에이전트가 정답이 든 결과를 보지 못함(탐침으로 확인). `mcp_proxy.py`의 도구 `_meta`(`anthropic/maxResultSizeChars`)와 `MAX_MCP_OUTPUT_TOKENS`로 고침, 2차에서 `n12-io` 성공 |
| T1-16 | 교환 관계 ADR 초안 | 쓰기 집합·읽기 집합(증분 키)·정제 판정(`2D-19`의 함의 질의)으로 정의한 교환 관계를 언어 명세 항목으로(협업 논의 Q1). 초안은 자율, 채택은 승인 | T1-08, 2D-19 | ⏳ |
| T1-17 | ADR-0062 초안: 증명 함수와 귀납 | Verus 꼴 `proof fn`(requires·ensures·decreases, 재귀로 귀납, `proof { }`에서 호출, 코드 생성에서 지움). `T1-14`의 귀납 부류와 T4 라이브러리 명세의 수요 | T1-14 | ✅ `dev-docs/adr/0062-proof-functions-and-induction.md`(제안) |
| T1-18 | ADR-0062 채택과 구현 | ADR 결과의 P1~P6(표면, 인코딩, 지움·맵·증분, 퍼저, 실용: `vc-vv0039`의 같은 뜻 명세 증명·`numlist` 이주, `bmb fix` 보조 정리 제안) | T1-17 | ✅ P1~P5(P6은 `T1-24`). 표면·검사: `proof fn`, `proof { }`, `Mode::Ghost`, E0256~E0258, 오류 코퍼스 `tests/check/err/proofs`(15줄, 모두 안내), 정규형 왕복은 코퍼스와 `fmt_fuzz` 생성기의 보조 정리. 인코딩: `tests/verify/pass/proofs`(재귀 ⇔ 한정 기호 두 방향, 앞부분 확장 `sum_grows`, 이음 `count_join`, 구조적 귀납 `size_nonneg`, 제네릭·다른 단위), `tests/verify/fail/proofs`(거짓 결론의 반례, 감소 실패 E0809, 전제 E0801, 유령 산술의 인자 범위 새 진단 E0819, `assert`). 지움·맵·증분: `codegen`의 `ghost_code_leaves_no_trace_in_the_ir`(유령 코드를 뺀 프로그램과 IR이 같은 바이트), `map`의 `proof_fns_and_their_callers_are_mapped`(`def_kind: "proof fn"`, L2 `proof_calls`), `incremental`의 `lemma_edits_follow_the_contract_firewall`(감사 모드). 퍼저·검출력: `soundness_specs`에 배치당 보조 정리 함수 2개(40배치: 보조 정리 80개 중 55개 증명, 결론 318회 확인), 게이트 변이체 5개(`v-lemma-*` 넷, `k-proof-block-is-exec`)를 대상 시험이 모두 잡음. 실용: `verify`의 `the_vv0039_equivalent_contract_is_proved_with_a_lemma`(판정기 텍스트 그대로 미결정 재현, 보조 정리로 증명), 실행 프로그램 `tests/run/max_list`(새)·`numlist`(경우별 술어 이주), `lib/utf8` 이음 명세(`valid_up_to` rlimit 310,475 → 62,292). 결정·측정: ADR-0062 "구현 시 확인". 판정기의 보조 정리 사용은 `T1-25` |
| T1-24 | `bmb fix`의 귀납 보조 정리 제안(ADR-0062 §5, P6) | 미결정(E0811) 의무의 질의에서 재귀 `spec fn` 하나의 정의를 따라가는 기계적 꼴(재귀 ⇔ 한정 기호, 앞부분 확장)을 알아보면 `proof fn` 초안과 호출 자리를 `replacement`로 낸다. 확인: 결정론 시험, 제안된 보조 정리가 증명되고 원래 의무가 풀린다(`vc-vv0039` 꼴) | T1-18 | ⏳ |
| T1-25 | 판정기가 작업 공간의 보조 정리를 쓴다(ADR-0062 채택 시 확인, 열린 질문 1) | 숨은 명세 강도 검사의 검사 함수가 에이전트의 `proof fn`을 어떤 인자로 부를지 정하는 규칙(예: 대상의 매개변수와 결과로 채울 수 있는 보조 정리를 차례로 `proof` 블록에서)을 정하고, 판정 변경은 ADR-0021의 재동결 절차를 따른다. 확인: `vc-vv0039`의 재귀 계약이 받아지고 BVB·m1b의 약한 계약은 여전히 거부된다 | T1-18 | ⏳ |
| T1-26 | ADR-0060 열린 질문 3: 반복 사이에 깨 두는 불변식 | `tests/run/sorted_set`의 `insert`를 `self.items` 위에서 제자리로 옮기면(Rust의 `copy_within` 꼴) 반복 사이에 같은 값 둘이 생겨 루프 머리의 유효성(E0823)에 막힌다. 지금은 사본 위에서 옮긴다(우회). 선택지(머리 유효성 유지 + 사본, 루프 `invariant` 절이 havoc된 값의 유효성을 대신하게 하는 표기, 루프 출구에서만 유효성)를 JUDGMENT §10으로 비교한 ADR 개정안. 의미 변경이라 채택은 승인 | T1-11 | ⏳ |
| T1-19 | H-COMMUTE: 의미 충돌 생성기 | 따로 적용하면 각자 증명되지만 합치면 깨지는 편집 쌍(한쪽은 `requires` 강화·`ensures` 약화·잎 제거, 다른 쪽은 옛 계약에 기대는 호출 추가)을 고정 시드로 만들고, git 깨끗한 병합의 거짓 병합률과 교환 판정의 검출률·건전성을 잰다 | T1-12 | ⏳ |
| T1-20 | MCP의 `rename`·`expect` | 새 과제 집합 판과 함께 MCP 쓰기 도구에 두 연산을 노출(ADR-0021: 동결 판의 도구 목록은 그대로, 옵트인 플래그). `mcp_fuzz` 모형에 두 연산 | T1-10c | ⏳ |
| T1-21 | H-PAR: `expect`가 비교할 값 | `id`는 전이적이라 피호출자 본문 편집이 호출자의 `expect`를 깬다. BCB 동시 재생(`T1-12` 설정)에서 전이적 `id`와 자기 텍스트 해시의 거짓 충돌·놓친 충돌을 재고 하나를 정한다 | T1-10c, T1-12 | ⏳ |
| T1-22 | H-ID 이득 쪽: 이름 바꾸기·이동을 섞은 BCB 이력 | BCB 이력 생성기에 이름 바꾸기·이동 편집을 더한 판, 커밋당 재인코딩 수와 CPU를 ADR-0059 전후로 비교 | T1-10e | ⏳ |
| T1-23 | `rename`의 남은 대상 | trait 메서드(선언·모든 `impl`·호출), 단위 사이 이동(`move`), 단위 이름 바꾸기. `id` 불변 확인은 같은 방식 | T1-10c | ⏳ |

### T2 언어 (수요 기반)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| T2-01 | ADR-0061 초안: `f64` | IEEE 의미, 기본 불투명 인코딩과 `by (bit_vector)` 안의 비트 정밀 증명, `f64::from` 정확성 의무 | — | ✅ `dev-docs/adr/0061-floating-point-f64.md`(제안) |
| T2-02 | `f64` 인코딩 비용 실험 | 항상 비트 정밀 대 기본 불투명의 rlimit·판정을 n-body·spectral-norm 초안과 합성 질의에서(`dev-docs/experiments/`, 2B-02 형식). ADR-0061 §5 기본값의 근거 | T2-01 | ✅ `dev-docs/experiments/T2-02/`(`run.py`, `results.md`, `README.md`): 부동소수가 목표와 무관한 의무(spectral-norm 행, n-body 쌍)에서는 두 인코딩의 판정이 같고 비용 차이 ±30% 안. 부동소수 비교가 경로를 정하는 mandelbrot 꼴은 항상 비트 정밀이 한도 안에 결정하지 못하고(k=4 반례, k=16), 불투명은 k=16을 rlimit 약 20만으로 증명. 부동소수 값 자체의 사실은 비트 정밀로만, 곱 하나(한도의 절반)까지. ADR-0061 §5 기본값 유지. 발견: 불투명 질의의 반례는 IEEE와 맞지 않을 수 있어 보이기 전에 재계산이 필요(T2-03) |
| T2-03 | ADR-0061 채택과 구현 | ADR 결과의 F2~F7. F4에는 불투명 질의 반례의 IEEE 재계산(맞지 않으면 반례 없이 미결정, G1, T2-02 발견)을 넣는다(파서·정규형, 검사기, 인코더, 코드 생성과 `print_f64`, 퍼저, n-body·spectral-norm) | T2-02 | 🔄 구현(ADR-0061 "구현 시 확인"). 세션 핸드오프로 멈춘 시점의 체크포인트이고, `fmt_fuzz`가 줄 안의 바이트 리터럴(`b' '`)을 건드리던 시험 결함을 함께 고쳤다. **끝난 것:** F2 렉서·파서·정규형(`tests/check/err/float_literals`, 새 진단 E0109, `fmt.rs`의 `float_literals_have_one_normal_form`), F3 검사기(`tests/check/err/floats`, `Eq`·`Ord` 거부 E0428 안내), F4 인코더(불투명 `f64.*`, `by (bit_vector)` 안 `fp.*`, 반례의 IEEE 재계산과 IEEE 반례 탐색: `tests/verify/pass/floats`, `tests/verify/fail/floats`, `verify.rs`의 `f64_counterexamples_are_confirmed_by_ieee_arithmetic`), F5 코드 생성(`double` 슬롯, `print_f64`: `codegen.rs`의 `f64_operations_compute_rusts_results`·`print_f64_formats_as_rust`·`f64_code_has_no_fast_math_or_contraction`, `tests/run/floats`·`mandel`과 Rust 대응 `*_ref.rs`, V6에 두 프로그램), F6 퍼저(`soundness_float`, 60배치 18,000값·360단언 통과, 변이체 `c-fadd-as-fsub`·`c-olt-as-ole`·`v-f64-lt-interpreted` 각각 목표 시험이 잡음), F7 `bench/game/nbody.bmb`·`spectralnorm.bmb` 증명·출력이 손으로 쓴 Rust 기준(`bench/game/ref/`)과 같음(`game_programs_match_their_rust_references`). **남은 것:** `compiler/README.md`의 표면 규칙 행 정리(지금은 요약 한 행), 공식 Rust 제출과 시간·크기 비교(T3-04), F6 변이체의 전체 게이트 실행(`gate_power.py run`), M2-d 종료 감사 |

### T3 공인 벤치

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| T3-01 | Benchmarks Game: 지금 쓸 수 있는 둘 | binary-trees·fannkuch-redux를 BMB로 쓰고 증명, 공식 Rust 제출과 출력 일치·시간·strip 크기 비교 하네스와 결과 | — | 🚫 BMB 쪽 완료: `bench/game/binarytrees.bmb`(n = 21)·`fannkuchredux.bmb`(n = 12)가 `lib/`와 함께 전부 증명(`game_programs_are_proved`)되고, 출력은 알려진 결과와 같다(`bench/game/*.out`). 막힌 것: 공식 Rust 제출과 공식 기대 출력의 출처(salsa.debian.org, benchmarksgame-team.pages.debian.net)가 네트워크 정책에 막힘. 환경 설정의 허용 도메인 추가 필요(사람) |
| T3-02 | HumanEval·MBPP 이식(MultiPL-E 방식) | 현재 언어로 옮길 수 있는 과제를 선별 기록과 함께 동결, 숨은 시험이 판정, 기준 해법·약한 대조군 CI. 실행은 T0 | — | ⏳ |
| T3-03 | BVB Verus 팔 파일럿 | 같은 BVB 과제를 원본 Verus 툴체인으로 T0-01과 같은 조건에서 실행(H-FAMILIAR 방향 판단, 파일럿). Verus 0.2026.10.04(`bench/bvb/verus/install.sh`)가 이 환경에 설치됨을 확인(2026-10-28). 필요한 것: Verus 하네스(`bench/bvb/verus/harness.py`)의 에이전트 자리에 CLI 파일럿을 끼우는 실행기 | T0-02 | ✅ `bench/pilot/verus_pilot.py`·`verus_mcp.py`, `bench/pilot/results/2026-10-28-bvb-verus-claude-opus-5-5.jsonl`: Verus 44/45($2.07), BMB 2차 42/45($4.44), 출력 토큰 2.06배·도구 호출 3.7배. Verus 0.2026.10.04 설치 확인 |
| T3-04 | Benchmarks Game: `f64` 둘 | n-body·spectral-norm을 BMB로 쓰고 증명, 공식 Rust 제출과 출력 일치·시간·크기 | T2-03 | 🚫 BMB 쪽 완료: `bench/game/nbody.bmb`(n = 1000)·`spectralnorm.bmb`(n = 100)가 전부 증명(`game_programs_are_proved`)되고 출력이 벤치의 작은 인자 기대 출력(-0.169075164/-0.169087605, 1.274219991)·손으로 쓴 Rust 기준(`bench/game/ref/*.rs`)과 같다(`game_programs_match_their_rust_references`). 막힌 것: 공식 Rust 제출의 출처가 네트워크 정책에 막힘(T3-01과 같음), 시간·크기 행은 조용한 머신에서(공유 머신이라 비워 둠) |
| T2-04 | 불투명 `f64` 질의의 MBQI 비용 조사 | `#[uses]`로 드러나지 않은 spec fn이 `&[f64]`(`(Array Int Float64)`)를 받으면 그 정의 공리가 있는 질의가 rlimit 5,000,000까지 돈 뒤 기저 인스턴스 재시도에서야 답했다(spectral-norm 초안, 구현 중 발견). 같은 꼴의 정수 배열 질의와 비교해 원인(FP 정렬의 MBQI)을 재고, 인코딩으로 줄인다 | T2-03 | ✅ 원인: 펼친 비재귀 `spec fn`의 양화 정의 공리가 반례가 있는 질의에서 Z3의 배열 모델 구성을 막았다(`incomplete (theory array)`). 최소 재현(공리 하나, 단언 다섯)에서 원소가 `Int`면 rlimit 240에 `sat`, `Float64`면 382,936에 `unknown`. 같은 뜻의 매크로(ADR-0064, `Prog::axiom`, 잎 `spec fn`만)로 바꿔 1,397에 `sat`. 실패 코퍼스 rlimit 합 137,479,381 → 20,609,594(−85%), 코드·위치 불변, `fail/generics.bmb`의 `fill_short` 하나가 미결정(E0811) → 반례 있는 실패(E0802). 재귀 제네릭 인스턴스는 그 제네릭으로 재귀를 판단(`recursive.bmb`). 비재귀 전부를 매크로로 하면 중첩 펼침 때문에 `lib/sha256.bmb`의 공허 검사가 시간 한도를 넘어(게이트 `test verify` 시간 초과) 잎 규칙으로 좁혔다 |
| T3-05 | Benchmarks Game: mandelbrot·fasta | 비트 출력과 바이트 출력이 필요한 둘 | 2N-05, T2-03, 2U-05 | ⏳ |

### T4 라이브러리 (공인 시험 벡터)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| T4-01 | SHA-256 | 전부 증명, FIPS 180-4 예제·NIST CAVP 벡터 통과, Rust `sha2` 대비 속도·크기 표. 2N-05의 실행 프로그램을 겸한다 | 2N-02 | 🔄 `lib/sha256.bmb`(22정의 전부 증명, 의무 191, 최대 rlimit 256,389 `compress`, `stability --seeds 8` 불안정 0). 사후조건은 기능 정확성이다: `digest(msg)`의 32바이트가 FIPS 180-4를 그대로 옮긴 명세 `hashed`(채우기 §5.1.1 `pad_byte`, 일정 §6.2.2 `sched`, 라운드 `round`·`rounds`, 상수 `kk`·`h0`, 블록 사슬 `chain`)와 같다. 코드는 명세와 같은 모양의 작은 함수(`big_sigma0`·`temp1`·`step`·`next_word` 등, 각각 명세 함수와 같음을 증명)로 나누고 호출자는 그 계약만 본다. 전제는 메시지 길이 < 2^61바이트(비트 길이가 64비트에 든다, FIPS의 한도). 채운 메시지를 한 번 복사한다(`compiler/src/sha256.rs`와 같다). 시험 `tests/lib_run/sha256.bmb`(377줄 모두 일치, `library_programs_print_expected_output`): FIPS·NIST 예제 다섯("abc", 빈 문자열, 448비트 두 블록, 896비트, 100만 개의 a), NIST CAVP SHAVS 바이트 지향 `SHA256ShortMsg.rsp` 65개(0~64바이트)와 `SHA256Monte.rsp` 몬테카를로 100개(§6.4, 다이제스트 100,000번), 의사 난수 바이트열의 길이 0~200(블록 경계 55·56·63·64·65·119·120 등)과 1000·4095·4096·4097·65536·70000. CAVP 줄의 기대값은 `.rsp` 파일의 MD이고(`tests/lib_run/sha256_cavp/`: csrc.nist.gov가 이 환경에서 막혀 pyca/cryptography 저장소의 사본을 받았다, SHA-256 `75e1cb83…`·`29ea30c6…`), 나머지는 Python `hashlib.sha256`(`sha256_ref.py`, CAVP 값도 `hashlib`와 대조). `SHA256LongMsg.rsp`(64개, 최대 6만여 바이트)는 소스에 넣기엔 커서 넣지 않았다(긴 길이는 `hashlib` 차등이 맡는다). 속도·크기 표 `bench/lib/results/2026-10-28.md`(`bench/lib/run.py`): 16MiB를 8번 해시하는 데 BMB 839ms 대 같은 알고리즘의 안전한 Rust 824ms(1.02배, 범위가 겹친다). strip한 실행 파일은 14,512 대 345,952바이트. 남은 것: Rust `sha2` 크레이트와의 비교(외부 크레이트라 승인 필요, crates.io는 닿는다). 발견: `1B-49`, `2N-08` |
| T4-02 | UTF-8 검증 | 전부 증명, Rust `std::str::from_utf8`와 차등 시험(경계·과잉 길이·대리 코드 포함), 속도 표 | — | ✅ `lib/utf8.bmb`(전부 증명, `stability --seeds 8` 불안정 0), 차등 시험 `tests/lib_run/utf8.bmb`: 길이 1~3의 모든 바이트열과 길이 4의 경계 조합 256,000개에서 유효 개수·`valid_up_to` 합이 Rust(`tests/lib_run/utf8_ref.rs`)와 같음(`library_programs_print_expected_output`). 속도·크기 표 `bench/lib/results/2026-10-28.md`(`bench/lib/run.py`, 9회 교차 실행 중앙값, 출력 일치 확인 후): 16MiB 섞인 UTF-8을 20번 검사하는 데 BMB 1,326ms 대 Rust `std::str::from_utf8` 1,562ms(0.85배, 범위가 겹치지 않음). strip한 실행 파일은 14,512 대 343,768바이트. 시간에는 두 쪽 같은 알고리즘의 입력 생성이 들어 있다 |
| T4-03 | base64 | 인코드·디코드 왕복 증명, RFC 4648 §10 벡터 통과 | 2N-02 | ✅ `lib/base64.bmb`(17정의 전부 증명, 의무 188, 최대 rlimit 343,768, `stability --seeds 8` 불안정 0). `encode(s)`: 결과가 명세 `enc_at`(§4: 24비트 묶음의 6비트 넷, 알파벳 표 1, 한·두 바이트 묶음의 `==`·`=`)과 같고(`encoded`), 정규형이며(`well_formed`), `dec_byte`로 `s`가 되읽힌다(`decodes`). `decode(t)`: `None`이면 정규형이 아니고(길이가 4의 배수, 알파벳 밖 문자, 끝이 아닌 `=`, §3.5의 버려지는 비트가 0이 아님), `Some(v)`면 정규형이고 `v`가 `t`의 디코딩이다. 왕복 `decode(encode(s)) == Some(s)`는 두 계약만으로 증명된다(`base64::round`). 증명하지 않은 것: 반대 방향 `encode(decode(t)) == t`(정규형 `t`). `decode`의 계약에 `encoded(v, t)`를 더하면 되지만, 자라는 `v`에서 `enc_at(v, i)`가 `v`의 길이(채우기 위치)에 달려 있어 루프 불변식으로 옮기는 증명을 시도하지 않았다. 시험 `tests/lib_run/base64.bmb`(102줄 일치): §10 벡터 일곱("", "f" … "foobar")의 인코드·디코드, 거부 열일곱(길이, `=` 위치, 정규형 아님 `Zh==`·`Zm9=`, 알파벳 밖, 정규형 셋은 받음), 의사 난수 바이트열 길이 0~64의 인코딩, 3000까지의 CRC-32 합(`lib/crc32.bmb`)과 왕복 3001개, 16문자로 된 모든 4문자열·8문자열 65,536개씩의 디코드 수와 합. 기대 출력은 Python `base64`(`base64_ref.py`; 정규형 판정은 `b64decode(validate=True)` 뒤 다시 인코드해 같은지). 발견: `1B-50` |
| T4-04 | CRC-32 | 전부 증명, 표준 검사값(`"123456789"` → `0xCBF43926`) 통과 | 2N-02 | ✅ `lib/crc32.bmb`(5정의 전부 증명, 최대 rlimit 6,020, `stability --seeds 8` 불안정 0). `checksum(s)`·`update(crc, s)`(zlib `crc32(crc, s)`)의 결과가 비트 단위 명세 `run`(반사 다항식 0xedb88320, 초기값·최종 xor 0xffffffff, 바이트마다 8비트 단계)과 같다. 시험 `tests/lib_run/crc32.bmb`(72줄, Python `zlib.crc32`의 `crc32_ref.py`와 같음, `library_programs_print_expected_output`): 검사값 3421780262(= 0xCBF43926), 빈 입력, "The quick brown fox …", 의사 난수 바이트열 길이 0~64와 0~4096의 합, 1·2바이트 입력 65,792개의 합, `update`로 나눠 계산한 4,097가지가 모두 전체와 같음. 표 방식(256항목)은 보조정리 `byte(c, b) == byte((c ^ b) & 0xff, 0) ^ (c >> 8)`가 혼합 인코딩에서 rlimit 5,000,000에 닿아(같은 식을 순수 bitvector로 쓰면 Z3가 160,687에 증명) `2N-03`(`by (bit_vector)`)을 기다렸다. 2N-03 뒤(2026-10-28): 표 방식이 구현이다. `table()`(256항목, 불변식 `filled`, 항목 `k`는 `byte(k, 0)`), `table_step`(찾기 하나와 `c >> 8`), `update_with(t, crc, s)`(표 하나로 여러 번), `update`·`checksum`은 표를 만들어 `update_with`. 표 단계가 비트 단계와 같음은 `proof fn table_lemma`의 `assert(..) by (bit_vector);`(rlimit 104,291, 순수 bitvector 측정 160,687보다 작다) → `table_step`의 사후 조건 5,859(같은 모양 합동 없이는 5,000,000에서 실패). 9정의 전부 증명, 최대 rlimit 104,291, `stability --seeds 8` 불안정 0. 시험은 73줄(표 하나로 0~4096 접두사 4,097개의 합 `table4096` 더함, Python `zlib.crc32`와 같음). 발견: `1B-49` |
| T4-05 | JSON 파서 | 전부 증명, JSONTestSuite의 `y_`·`n_` 판정 통과 | 2U-03 | ⏳ |


### 국면 종료 감사 (M2-a ~ M2-d)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2P-01 | M2-a 종료 감사 | ROADMAP M2-a 통과 기준 1~5를 하나씩 확인, 가설 등록부 갱신, `health.py table` 스냅샷 `dev-docs/health/M2a.md` | T3-03, T1-12, T1-14, T1-15, 2A-14e | ⏳ |
| 2P-02 | M2-b 종료 감사 | M2-b 통과 기준 1~3, `dev-docs/health/M2b.md` | 2N-05, T4-01, T4-02, T4-03, T4-04, T3-01 | ⏳ |
| 2P-03 | M2-c 종료 감사 | M2-c 통과 기준(채택된 ADR 범위), `dev-docs/health/M2c.md` | T1-10 | ⏳ 승인(ADR-0059) |
| 2P-04 | M2-d 종료 감사 | M2-d 통과 기준(채택된 ADR 범위), `dev-docs/health/M2d.md` | 2U-05, T4-05 | ⏳ |

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
| INF-07 | 장기 퍼징 정기 실행 | 매일 1회 CI가 프론트엔드 퍼징(`BMB_FUZZ_ITERS`)과 건전성 퍼징(`BMB_SOUND_ITERS`)을 게이트보다 큰 횟수로 실행. 실패 시 시드가 로그에 남음 | 1A-09, 1B-17 | ✅ `.github/workflows/nightly.yml`(매일, 수동 실행 가능). 첫 실행(run 1, 수동) 성공: 프론트엔드 100,000회, 정수 건전성 400배치, 25분. 이후 루프 퍼징(1B-21) 추가 |
| INF-08 | 게이트 시간 단축 | 게이트가 5분 30초를 넘음(m1b 추가 후). 단계 병렬화·중복 빌드 제거·느린 테스트 정리로 시간 측정과 함께 단축, 검사 범위는 줄이지 않음 | INF-05 | ✅ `scripts/gate_jobs.py`(테스트 실행 파일·벤치를 병렬로, 실패 작업 로그와 종료 코드 보존, 시간 초과), 빌드 한 번. 게이트 약 7.2분 → 2분 28초(부하 상태 측정). 판정에 증명 캐시 사용(`query`·`map --cache`) |
| INF-09 | 지속 관리 지표와 래칫 | 다섯 축 중 기계로 셀 수 있는 V·S·P·O 지표를 계산하고 기준선과 비교해 고정·래칫·예산 위반 시 실패. 게이트에 포함, 악화는 이유와 함께만 기준선 갱신 | INF-05 | ✅ `scripts/health.py`, `scripts/test_health.py`, `dev-docs/health/baseline.json`, ADR-0039, `dev-docs/HEALTH.md` |
| INF-10 | 게이트 종료 코드로만 푸시 | 커밋된 트리에서 게이트를 돌려 종료 코드 0일 때만 푸시. 미커밋 변경·main·실행 중 트리 변경이면 거부 | INF-05 | ✅ `scripts/ship.sh`(게이트 실패를 출력으로 읽어 푸시한 사고 2회: `8dcdb33`, `6947378`) |
| INF-11 | 벤치 부하의 증명 비용 지표 | 하네스 결과에 정의별 `rlimit`을 기록하고, `health.py`가 m1b·BVB 기준 해법과 BSB 킬 검사의 비용 합·최대를 추적(V4·V5 확장). 1B-37 원안을 다시 넣으면 지표가 실패함을 확인 | INF-09 | ✅ `bench/m1/harness.py` 결과 행에 판정 작업 공간과 BSB 킬 검사의 `rlimit` 합·최대(`proof_cost`). `health.py`가 m1b·BVB 기준 해법을 돌려 V8 비용 합(예산 +10%)·V9 한도에 닿은 정의 수(래칫, 감소만). 기준선 V8 50,821,776, V9 2. 1B-37 원안(모든 `i64` 배열 원소에 범위 공리)을 다시 넣으면 V8 66,166,481(+30%)로 예산 위반 확인. V5형 고정 상한(≤ 0.2)은 지금 이미 한도에 닿은 의무 2개가 있어 래칫으로 바꿨다(1B-42) |
| INF-12 | 성능 회귀 야간 추적 | `bench/baseline`을 야간 CI에서 돌려 안전 Rust 대비 비율이 ±5%(G6) 밖이면 실패, 결과 보존 | INF-07 | 🔄 `nightly.yml`의 `perf` 작업: `bench/baseline/run.py --runs 15 --max-ratio 1.05`(BMB/안전 Rust 비가 1.05를 넘으면 실패, 빠른 것은 회귀가 아님), 결과는 아티팩트 `perf-N`. 로컬에서 한도 0.5는 종료 1, 5는 종료 0 확인. 야간 실행 확인 남음 |
| INF-13 | 빌드 시간 관측(수용 비용) | 첫 빌드·증분 빌드 시간을 정의당으로 야간 기록(ADR-0041 수용 비용의 관측). 게이트 판정에는 쓰지 않음(시간은 비결정) | INF-07 | 🔄 `scripts/build_time.py`: m3 작업 공간(N=1,000)에서 `emit-llvm --cache` 차가운·따뜻한·정의 하나 편집 CPU와 정의당 비용을 JSON 한 줄로(실패하지 않음), 야간 `perf` 작업이 아티팩트로 보존. N=100 로컬: 차가운 1.18초(정의당 11.8ms), 편집은 1개만 재인코딩. 야간 실행 확인 남음 |
| INF-14 | 사용자 실행 묶음 | 실제 모델 측정(m1 `1E-10`, m1b `2A-06`, BVB, BRB)을 명령 하나로 돌리는 진입점. `--dry-run`은 대본 에이전트로 끝까지 돌고 과제 수·예상 토큰·예상 비용을 보고, `--budget`을 넘으면 시작하지 않음, 결과를 각 벤치의 결과 경로에 쓰고 요약(`summarize`)을 낸다. 한 번의 유료 실행이 G2·G3·G5·G8·H-ORACLE·H-SPEC·H-FAMILIAR 열을 모두 채우는지 게이트에서 대본 에이전트로 확인 | 1E-03, 2A-09, 2H-03 | ✅ `bench/run_models.py`(설명 `bench/README.md`): 여섯 집합을 순서대로 기존 하네스에 돌리고 요약과 지표별 채움 여부를 한 보고서로. 가격·예산은 인자(가격은 바뀌므로 넣지 않음), 다음 집합의 예상 비용(세션당 평균 × 세션 수)이 예산을 넘으면 시작하지 않음(`over_budget`). 완료 기준의 "dry-run이 예상 토큰·비용 보고"는 바꿈: 대본 에이전트는 토큰을 쓰지 않고 세션 상한(도구 40회 × 요청당 16,000 출력)은 너무 커서 쓸모없는 상한이라, 실제 지출로 다음 집합을 예상한다. 미커밋 트리·키 없음·MANIFEST 불일치·Verus 없음이면 시작 전 거부. 게이트 `bench run_models (dry run)`(`--limit 2`, G2·G3·H-SPEC·G5·H-ORACLE·G8 채움, Verus 없어 H-FAMILIAR 건너뜀)과 `bench/test_run_models.py` |
| INF-15 | 게이트를 커밋 단독 체크아웃에서 | 2V-15 게이트가 작업 트리의 쓰다 만 ADR 색인을 읽어 실패했다(2V-04a의 `target/` 재빌드와 같은 종류). 판단 기준 §1(프로세스 층): 게이트는 푸시할 커밋만 보아야 한다 | INF-10 | ✅ `scripts/ship.sh`: `.git/ship-gate` 워크트리(자체 `target/`, 증분 유지)를 그 커밋으로 맞추고 무시 파일까지 지운 뒤 게이트, 통과하면 그 커밋을 푸시. 작업 트리 편집·빌드와 병행 가능 |

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
| 1A-10 | 배열 길이 상한 | `[T; N]`의 N이 `u128 → u64`로 잘리지 않고, ADR-0017 §3 길이 상한을 넘으면 E0406. 코드 생성의 `len * 8` 넘침 방지 | 1A-03 | ✅ 타입 표기의 길이도 `u32::MAX` 상한(E0406, 잘림 없음, 후속 타입 오류 없음). 1A-11 작업 중 발견된 동일 문제 포함. `tests/check/err/types.bmb`의 `array_limits` |
| 1A-11 | 중첩 깊이 제한 | 소스 파서와 JSON 파서에 깊이 상한(초과 시 진단·오류). 50k 괄호·200k `[`로 스택 넘침이 재현됨(MCP 서버 종료) | 1A-02 | ✅ JSON `MAX_DEPTH` 256, 파서 `MAX_NESTING` 256과 새 코드 E0107, CLI·MCP를 256MiB 스택에서 실행(`bmb::with_stack`). `tests/nesting.rs`, `check/err/nesting.bmb`, MCP 200k `[` 테스트 |
| 1A-12 | 리터럴 식의 타입 추론 | 리터럴만으로 된 식(`(66 + 14)`, `if c { 5 } else { 7 }`)과 `if`의 리터럴 가지가 다른 쪽 피연산자·가지에서 타입을 받음(Rust와 같음). 이전에는 `i64`로 정해져 `u64` 코드가 E0401 | 1A-03 | ✅ 1B-33 퍼저가 발견. `check.rs`의 `untyped_literal`, `tests/check/ok/literals.bmb` |
| 1A-13 | 검사기 완화 세 가지 | 읽기만 하는 스칼라 인자와 `&mut` 함께 넘기기, 명세의 부호 없는 부정, 명세의 큰 리터럴(감사에서 확인, 사람 승인) | 1A-03 | ✅ ADR-0024, `tests/run/relaxed.bmb`, 거부 유지 사례는 기존 `check/err` |
| 1A-14 | 리터럴 수정이 값을 바꾸던 버그 | `0x10`의 E0104 수정이 `0`으로 바꿈(2R-02 발견). 16·8·2진은 같은 값의 10진으로, 타입 접미사만 제거, 그 밖은 기계 수정 없음 | 1A-06 | ✅ `lexer.rs`, `tests/fix_cmd.rs`의 `literal_fixes_keep_the_value` |
| 1A-15 | 같은 E0237이 두 줄로 나온다 | 제네릭 `spec fn`을 계약에서 타입 인자 없이 부르면(`ensures same(x, x)`) 같은 진단 줄이 두 번 출력된다(T1-18 구현 중 발견, 실행 함수도 같음). `Diags::to_jsonl`의 중복 제거는 정렬 뒤 이웃한 같은 줄만 지우므로 두 항목의 정렬 키(파일·위치)가 다르다. 원인을 찾아 한 번만 내고, 회귀 시험을 더한다 | — | ⏳ |

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
| 1B-21 | 루프·배열 건전성 퍼징 | `&mut` 배열·`while`·불변식·`old`가 있는 임의 함수를 증명하고, 증명된 함수가 결함 없이 불변식(매 루프 머리)과 사후 조건을 지킴을 해석기로 확인. 진입 불변식 반례 재현, 두 빌드 출력(배열 내용 포함) 일치 | 1B-17 | ✅ `compiler/tests/soundness_loops.rs`(게이트 6배치, 야간 200배치). 정확한 템플릿 4종 + 변이 1개, 무작위 함수. 60배치: 함수 360개 중 증명 162개를 입력 964개로 실행(불변식·사후 조건·결함), 위반 0. 반례 131개 전제 만족, 진입 반례 9개 재현. 컴파일 출력 14,108개 일치. 발견: 1B-22(반례 표시), 합 루프의 길이 상한 필요(마찰, 판정은 옳음) |
| 1B-22 | 반례 표시 버그: 짧은 배열 뒤 매개변수 값 밀림 | 길이 < 4인 배열 다음 매개변수에 배열 원소 값이 표시되던 문제 수정과 회귀 코퍼스 | 1B-10 | ✅ 1B-21 설계 중 `counterexample()` 검토로 발견. `x = 8`(실제 원소 1) → `x = 7`. `fail/bounds.bmb`의 `cex_after_array` |
| 1B-23 | 건전성 버그: 배열 리터럴을 같은 배열에 대입 | `a = [a[1], a[0]]`이 원소를 하나씩 저장해 뒤 원소가 바뀐 값을 읽음. 증명은 `r == 1`, 실행은 2. 모든 원소를 먼저 평가한 뒤 저장 | 1C-01 | ✅ 커버리지 조사(2026-10-24)에서 발견. `codegen.rs` `init_array`, `tests/run/swap.bmb` |
| 1B-24 | 배열 반환 함수가 코드 생성에서 패닉 | `fn f() -> [i64; 2]`가 검사·증명을 통과한 뒤 `unreachable!`. M1에서는 E0402로 거부(`&mut [T]` 매개변수로 채우기) | 1A-03 | ✅ 같은 조사에서 발견. `check.rs`, `tests/check/err/types.bmb`의 `array_return` |
| 1B-25 | 배열 값 `if`가 증명·코드 생성에서 패닉 | `let a: [i64; 2] = if c { [1, 2] } else { [3, 4] };`가 검사를 통과한 뒤 `verify.rs` "index of non-array" 패닉(재현). 증명은 배열 병합, 코드 생성은 배열 슬롯+복사 | 1B-03 | ✅ 증명: 배열 병합(`merge`), 명세 `if`도. 코드 생성: 배열 버퍼에 가지별 복사. `tests/run/array_if.bmb`(bool 배열·자기 대입 포함), `verify/pass/conditions.bmb`의 `array_if` |
| 1B-26 | 분기 병합 항의 지수적 증가 | 병합마다 항을 문자열로 복제. 연속 `if` 13개가 12초에 `rlimit` 초과로 올바른 함수를 증명 못 함(재현). 병합값을 새 상수로 묶음 | 1B-03 | ✅ 200자 넘는 병합 항을 새 상수에 묶음(`Enc::merge`). 연속 `if` 13개: 11.8초·증명 실패 → 0.13초·증명, 40개 0.38초. 작은 항은 그대로라 코퍼스 기대 출력 불변 |
| 1B-27 | 지역 배열을 명세 술어에 넘기기 | `p(&a)`(지역 `[T; N]`)가 명세에서 미지원(E0814) 후 해법기 오류(E0812)로 이어짐. `Ref`를 명세에서 펼침 | 1B-06 | ✅ 명세의 `&a`는 배열 자체. `verify/pass/conditions.bmb`의 `after_and`(`first_is_one(&a)`) |
| 1B-28 | 루프 havoc의 지역 변수 타입 | 대입값·조건 안 블록에 선언된 지역 변수를 타입 없이 havoc해 E0812(sort mismatch). 검사기 타입표 사용 | 1B-03 | ✅ `find_let`이 모든 문장·식(대입값·호출 인자·조건·`while` 조건)을 탐색 |
| 1B-29 | `&&`·`or` 오른쪽의 사실 보존 | 오른쪽 피연산자의 피호출자 `ensures`·나눗셈 보조 사실이 버려짐(불완전, 건전). `(=> guard facts)`로 보존 | 1B-03 | ✅ 오른쪽에서 얻은 사실을 `(=> guard facts)`로 보존. 수정 전에는 `after_and`·`after_or`가 거짓 확정 반례(`c = true`)와 함께 실패했다. `verify/pass/conditions.bmb` |
| 1B-30 | 명세에서 부호 없는 원소의 범위 사실 | `&[u64]` 원소 읽기에 `0 ≤ a[i] ≤ MAX`가 명세·havoc에서 빠짐(불완전) | 1B-06 | ✅ 부호 없는 원소 배열을 새로 만들 때(매개변수, 루프·호출 havoc) 패턴 있는 범위 공리 추가(`Enc::fresh_elems`). `tests/verify/pass/unsigned_elems.bmb`(명세만 읽는 경우, 루프로 쓴 뒤) |
| 1B-31 | 부분 증명 IR의 검사 제거 | `emit-llvm`에서 실패한 전제·불변식을 가정해 증명된 의무의 검사가 지워짐(`build`는 거부하므로 안전). 증명이 완전하지 않은 정의에서는 검사를 남김 | 1C-03 | ✅ `lib::usable_proofs`: 호출 그래프에서 실패한 정의와 연결된 구성 요소는 검사를 유지. `tests/codegen.rs`의 `proofs_resting_on_failed_ones_keep_their_checks` |
| 1B-32 | 호출·재귀 건전성 퍼징 | 피호출자 계약 가정, `&mut` 인자 havoc, `old`, 재귀·상호 재귀 `decreases`를 생성기로 확인(1B-17 방식) | 1B-21 | ✅ 호출: `tests/soundness_calls.rs` 60배치, 호출자 300개 중 피호출자까지 증명된 51개를 306회 실행, 위반 0. 재귀: `tests/soundness_rec.rs`(`decreases n`, 비종료 c=0 포함) 60배치, 360개 중 증명 47개를 282회 실행, 위반 0, 반례 557개 전제 만족 |
| 1B-33 | 부호 없는 산술 건전성 퍼징 | u64/usize 값(2^63 이상 포함), `-`·`/`·`%`·`wrapping_*`를 두 빌드와 해석기로 확인 | 1B-17 | ✅ `tests/soundness_unsigned.rs`(2^63 이상 값, 언더플로, 부호 없는 `/`·`%`·`wrapping_*`, 두 빌드는 기대값 비교 출력). 60배치: 480개 중 증명 55개 330회, 반례 595개 재현, 컴파일 3,698회 일치, 위반 0. 발견: 1A-12 |
| 1B-34 | 배열 대입 퍼징 | 배열 리터럴·`[v; N]`·배열 복사 대입을 루프 퍼저에 추가(1B-23 재발 방지) | 1B-21 | ✅ 정수 퍼저에 지역 배열 `t: [i64; 3]`의 통째 대입(`t = [..]`, `t = if c { [..] } else { t }`) 추가. 1B-23 버그를 되돌리면 실패함을 확인 |
| 1B-35 | 중첩 배열 리터럴이 검증기에서 패닉 | `[[1]][0][0]`이 검사를 통과한 뒤 "array used as scalar" 패닉(1D-19 작업 중 발견). 배열 리터럴 원소도 스칼라만 허용(E0402) | 1A-03 | ✅ `check.rs` 배열 리터럴, `tests/check/err/types.bmb`의 `array_limits` |
| 1B-36 | 명세·수량자 건전성 퍼징 | `forall`·재귀 `spec fn`·`#[uses]` 펼침·기저 인스턴스 재시도(ADR-0023)가 있는 임의 계약을 증명하고, 증명된 것은 해석기로(유한 범위 수량자 직접 평가) 확인, 반례는 재현 | 1B-21 | ✅ `tests/soundness_specs.rs`(템플릿 14종 + 변이 55%, 25%는 `#[uses]` 누락, 삼치 해석기). 60배치: 360개 중 증명 170개(47%)를 996회 실행, 위반 0. 확정 반례 44개 재현. 300배치도 위반 0. 버그 없음 |
| 1B-37 | `i64` 원소 범위 사실 | `small(#[trigger] a[i])` 같은 `i64` 원소 위 명세 함수가 수량자 안에서 펼쳐지지 않음(BVB 작업 중 발견). 원소 범위 공리를 매개변수·선언 지역의 `i64`까지 확장(호출 뒤·루프 재설정 값은 제외: 전체 확장 시 m1b BSB 킬 검사 2개가 rlimit 초과) | 1B-30 | ✅ `fresh_elems(.., signed)`, `tests/verify/pass/elem_preds.bmb`. 코퍼스 판정 불변, m1b oracle BSB 186/186 |
| 1B-38 | 증명기 검출력 측정 | `verify.rs`·`codegen.rs`에 의도적 결함(부호·경계·의무 누락 등) 20개 이상을 하나씩 넣고 게이트(코퍼스·퍼저)가 잡는 비율을 측정·커밋. 놓친 결함마다 시험 추가 | INF-09 | ✅ `scripts/gate_power.py`(변이체 31), `dev-docs/measurements/gate-power/`. 첫 실행 26/31(서식·경고로만 실패한 3개는 경고 없는 변이체로 다시), 놓친 5개마다 시험(`loops.bmb` 두 곳, `codegen.rs` 하나) 뒤 31/31(852d1ae, 10a8b4f). 관찰: 8개는 한 작업만 잡고, 건전성 퍼저는 `%` 최솟값·부정 오버플로·측도 하한·루프 havoc 결함을 못 잡는다 → 1B-43 |
| 1B-43 | 건전성 퍼저가 못 잡는 결함 종류 보강 | 1B-38 관찰: `v-min-rem-dropped`·`v-neg-overflow-dropped`·`v-decreases-unbounded`·`v-havoc-skipped`·`v-mut-param-havoc-skipped`·`v-recursion-measure-dropped`는 `test verify` 코퍼스만 잡는다. 퍼저 생성기에 `i64::MIN`·`-1` 피연산자, 부정, 측도가 음수가 되는 루프, 루프 안 `&mut` 전달을 넣고, 각 변이체를 퍼저 단독으로 잡는지 `gate_power.py run --only`로 확인 | 1B-38 | ⏳ |
| 1B-39 | 두 번째 솔버 교차 확인 | 코퍼스·벤치 질의를 cvc5로도 풀어 판정 불일치 0 확인(단일 솔버 위험, HEALTH.md §3). 외부 도구 추가라 승인 필요 | 1B-38 | ⏳ 승인 |
| 1B-40 | 솔버 안정성 퍼저 (H-STABLE) | 코퍼스·시드의 질의에 의미 보존 변형(이름 바꾸기, 단언 순서 섞기, 시드)을 적용해 판정이 바뀌는 질의 비율을 측정. 비율을 `health.py` V7(기준 < 2.6%, Mariposa)로 추적. 불안정 질의는 원인 태스크로 | INF-09 | ✅ `scripts/solver_stability.py`(`--emit-smt`의 의무마다 단언 순서 섞기·기호 이름 바꾸기·시드·셋 다, 같은 `rlimit`이라 부하와 무관, 변형 시드는 파일·의무·종류로 고정). 측정(2026-10-25): 코퍼스·시드 16개 프로그램 의무 718개 × 변형 16개(종류당 4)에서 판정이 바뀐 의무 0(0.00%). 불안정 질의 없음 → 원인 태스크 없음. `health.py` V7(고정 < 2.6%, 게이트는 종류당 1개), `scripts/test_solver_stability.py`. 한계: M1 질의는 작다(정수·배열, 수량자 적음) |
| 1B-41 | 편집 재생 국소화 시험 (H-MAINT) | 코퍼스·m3 작업 공간에 무작위 편집(본문·계약, BCB 동결 이력과 `build --stats` 재증명 집합을 재사용할 수 있다)을 적용하고, 증분 빌드가 다시 증명한 정의 집합이 `impact` 집합 안에 있음을 시험. 재증명 비용(rlimit 합)의 편집당 분포 보고 | 2D-10 | ✅ `bench/bcb/localize.py`: BCB 이력을 재생하며 커밋마다 `verify --cache --stats`의 재인코딩 집합 ⊆ 편집한 정의의 `impact` 합집합(본문 편집 `body`, 계약·명세 편집 `iface`, 새 정의는 자신)인지 확인하고 재증명 `rlimit` 합을 기록. 동결 이력 측정(2026-10-25, ADR-0043 컴파일러): `body-heavy-1k` 커밋 1,000개 밖으로 샌 커밋 0, 재인코딩 평균 1.71 대 영향 1.78, 커밋당 `rlimit` 중앙값 14,434·90% 36,913·최대 135,523. `contract-heavy-1k` 0, 2.10 대 2.26, 13,158·47,988·145,012. 게이트 `test_bcb.py`의 `check_localization`(N=100 세 프로필, 빈 영향 집합이면 실패하는지도). H-MAINT 반증 안 됨(이 코퍼스에서) |
| 1B-44 | 공허 증명 검사 | 2G-05 발견: 길이를 잘못 둔 루프 처리로 루프 뒤 코드가 도달 불가가 되어 코퍼스 두 정의가 공허하게 증명되었는데 증명기가 알리지 못했다. 증명된 실행 정의마다 본문 끝(그리고 각 루프 출구)의 경로 조건이 만족 가능한지 확인하고, 아니면 경고(W 코드, `bmb lint`)나 증명 실패로 한다. 코퍼스·벤치에서 오탐 0, 2G-05 결함을 되돌리면 잡는 시험 | 1B-38 | ✅ `verify::vacuous_ends`(인코더가 루프 출구·본문 끝의 경로 조건을 기록, 해법기가 반박하면 보고, `rlimit` 100만, 결정 못 하면 무보고), `bmb lint` W0905, `bmb::vacuous_points`. 시험 `proofs_are_not_vacuous`: `tests/verify/pass`·`tests/run` 전부, 벤치 기준선 7개와 씨앗 3묶음(m1·m1b·bvb)에서 반박 0(23초). 2G-05 결함을 되돌리면 `run/primes`의 다섯 지점으로 실패하고, `bmb lint`는 재현 함수의 루프 출구와 본문 끝 둘을 보고. 벤치 과제 해법(JSON 편집)은 하네스가 적용하므로 이 시험 밖 |
| 1B-45 | 자기 매칭 트리거 경고 | 1B-42 발견: `forall`의 트리거 항(`out[k + 1]`)과 같은 배열을 다른 첨자로 읽는 본문 항(`out[k]`)이 있으면 인스턴스가 사슬로 늘어난다(매칭 루프, 벤치 킬 검사 하나가 한도에 닿음). `bmb lint` 경고(W09 계열 새 번호)와 안내(두 항 모두 `#[trigger]`로 다중 트리거)를 더하고, 코퍼스와 m1b `w22` 기준 명세로 시험 | 1B-42 | ✅ W0906(`src/lint.rs` `self_matching`): 트리거 항이 하나인 `forall`에서 트리거 밖의 같은 배열·함수 항이 다른 인자로 묶인 변수를 쓰면 경고하고, 그 항을 `#[trigger]`로 표시하는 교체를 붙인다. 시험 `lint::self_matching_triggers`(앞·뒤 첨자, 함수 호출, 중첩 `forall`에서 보고, 다중 패턴·바닥 항·다른 배열은 보고 없음, 교체 후 보고 없음), `lint::prefix_sum_specification_is_reported`(m1b `w22` 기준 명세의 `runs_below`·`prefix_ok`에서 보고, 다중 패턴으로 바꿔도 증명되고 rlimit 142811 → 142815로 같음) |
| 1B-47 | 상수 나눗셈·나머지의 인코딩 | `x / c`·`x % c`(상수 `c`)를 몫·나머지 새 기호와 선형 제약(`x = c*q + r`, 나머지 범위, 0 쪽 반올림)으로 인코딩해 비선형 산술을 피한다(JUDGMENT §8). 확인: `vc-va0093-max-balance`의 같은 뜻 명세 함의가 결정됨, V4·V7 유지 | T1-14 | ⏳ |
| 1B-48 | `utf8::width` 사후조건 의무의 증명 비용 | 의무 하나가 rlimit 1,843,904(한도 5,000,000의 37%). 원인(중첩 `if`의 경로 수, `seq_len` 펼침)을 분해하고 인코딩에서 줄인다. 사용자 코드를 쪼개 우회하지 않는다(JUDGMENT §8) | T4-02 | ⏳ |
| 1B-49 | 추상 질의의 비트 항 합동 | T4-04 발견: 코드의 `x ^ k`와 명세의 `x ^ k`가 추상 질의(ADR-0048 §4)에서 서로 모르는 두 상수였다(코드는 피연산자 bitvector 상수, 명세는 `int2bv`). 그래서 `r == bit(c)` 하나가 정확한 질의로만 갈 수 있었고 rlimit 5,000,000에서 끝나지 않았다. 같은 연산을 다른 표기로 쓴 항을 합동 사실로 잇는다(건전성: 어느 모형에서나 참) | 2N-02 | ✅ `verify::bit_congruences`(`abstract_bits`가 부름): 잎을 폭과 값(코드 상수는 주어진 값, 2의 보수 꼴은 `mod`)으로 읽어 같은 나무가 되는 두 항(중첩 항 포함)에 "다른 잎의 값이 같으면 같다". 시험 `bit_congruences_relate_notations_and_hold`(만든 사실 셋이 기대와 같고, 표본 값 30쌍마다 Z3가 부정을 반박), 코퍼스 `tests/verify/pass/bits`의 `mix`(중첩)·`mask_xor`(`i16`)가 실패 → 증명(rlimit 2,475·6,829). `crc32::bit_step` 10,003,176(실패) → 2,481, `byte_step` 5,008,708 → 3,228. 전체 `cargo test` 통과(실패 코퍼스 출력 불변) |
| 1B-50 | 실행 코드 열거형 값의 페이로드 범위 사실 | T4-03 발견: `decode`의 결과 `Option<Vec<u8>>`를 받아 `match` 없이 계약끼리 잇는 증명(`base64::round`)이 `unknown`이었다. 명세 `decodes(t, v)`의 정의 공리는 `v.len()`의 범위 가드 아래에서만 펼쳐지는데, 호출 결과의 페이로드 길이 `Some.0.len`에 범위 사실이 없었다(`match` 갈래만 준다, ADR-0027 §5). 실행 코드의 열거형 값(매개변수·호출 결과·루프 뒤 값)에 변형마다 페이로드 잎의 범위 사실을 그 변형일 때로 준다 | 2B-05 | ✅ `Enc::fresh_val`의 열거형 갈래(`enum_variants`, `Enc::payload`의 사실을 `(=> ((_ is V) x) ..)`로). `base64::round` 실패 → 증명. 실패 코퍼스 여덟 파일은 판정·진단 코드·위치가 같고 rlimit 숫자와 반례 값 하나(`mut_match`의 `mf::half`: 다른 반례)만 바뀌어 다시 기록 |
| 1B-46 | 실패 의무의 증명 비용 | 2T-01 관찰: 거짓인 `assert` 하나(`tests/verify/fail/generics` `wrong_value`, 같은 모양의 비제네릭 프로그램도)가 rlimit 상한 5,000,000까지 쓴다. 실패 코퍼스 다섯 파일에 같은 항목. 원인(첫 질의의 `unknown`, 반례 확인, 바닥 인스턴스 재시도 중 어디인지)을 `--emit-smt`로 찾아 고치고, 실패 의무의 rlimit을 건강 지표로 잰다 | 1B-38 | ⏳ |
| 1B-42 | 해법기 한도에 닿는 벤치 의무 원인 조사 | INF-11이 찾음: m1b `w22-prefix-sums`의 BSB 킬 검사 하나와 BVB `vc-va0053-aspect-pairs` 기준 해법의 의무 하나가 `rlimit` 5,000,000에 닿고 기저 인스턴스 재시도로만 증명된다. 원인(수량자 패턴, 펼침)을 특정하고, 인코딩으로 줄일 수 있으면 고쳐 V9를 0으로(증명 결과 불변) | INF-11 | ✅ 원인 둘을 특정. (1) BVB `vc-va0053` 기준 해법: `solve`의 사후조건이 `a / (x / g) >= 0`(피연산자 양수)을 비선형 산술로 찾다 한도에 닿았고, `gcd_exec`의 사후조건은 `i64` `%`(0 쪽 자름)와 명세 `given::gcd`의 `mod`(유클리드)가 같음을 곱 `b * (a / b)`로 찾다 닿았다(z3 통계: 수량자 인스턴스 24~56개, 비선형 단계가 비용). 인코딩 수정: `SIGNED_DIV_LEMMAS`에 몫의 부호 사실과 비음수 피연산자의 `div`·`mod` 일치 사실 추가(`tests/lemmas.rs`가 정리로 증명). `solve` 6,728,516→653,268, `gcd_exec` 5,059,400→249,577(재시도 없음). (2) m1b `w22` 킬 검사 `k0_6`: 동결된 기준 명세의 트리거 `#[trigger] out[k + 1]`이 본문의 `out[k]`와 다시 맞아 매칭 루프(인스턴스 15,562개, 세대 11). 사용자 트리거를 약하게 바꾸지 않으면 인코딩으로 없앨 수 없어 남긴다(다중 트리거 `out[k + 1]`·`out[k]`이면 21,907). 경고 진단은 1B-45. 측정: V9 2→1, V8 50,821,776→39,910,940, V4 +2.0%, 증명 결과 불변(실패 코퍼스는 rlimit 숫자만 바뀜) |

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
| 1D-14 | `edges` 깊이 조기 종료 | 경계가 비면 반복 중단. 깊이 10^11에서 멈추지 않음(재현, MCP로도 도달) | 1D-06 | ✅ `reach`가 경계가 비면 멈춤. `tests/map.rs`의 `edges_with_a_huge_depth_terminate` |
| 1D-15 | 작업 공간 단위 경로 검증 | `tx_unit path="../../x"`가 작업 공간 밖에 파일을 씀(재현). 경로 각 부분을 식별자 규칙으로 검사, 빈·공백 이름 거부 | 1D-08 | ✅ `parser::is_ident`로 모든 연산의 경로 각 부분 검사, `write_units`도 거부. `put`의 `mod` 주입도 막음. `tests/workspace.rs`의 `invalid_paths_are_rejected_and_write_nothing`(17가지) |
| 1D-16 | JSON 문자열 파싱 선형화 | 문자마다 남은 입력 전체를 UTF-8 검사(400KB에 3.7초) | 1D-11 | ✅ 다음 `"`·`\\`까지 한 번에 복사. 1.5MB 문자열 디버그 1초 미만(`json::tests::long_string_is_linear`) |
| 1D-17 | 질의마다 재증명하지 않기 | 증명 상태가 필요 없는 질의(`units`·`edges`·`find`·`slice`·`contracts`·`impact`)는 증명하지 않음 | 1D-06 | ✅ 증명 상태를 쓰는 요청(L1 맵, `surface`, `status`)만 증명. 나머지는 z3 없이 동일 출력(`tests/no_prove.rs`). `query units` 652→3ms |
| 1D-18 | 해법기 시작 비용 | 정의마다 z3 재시작과 `z3 --version` 재실행을 없애고, 의무마다 명령 왕복을 묶음. 프로세스 전체 `-T` 한도가 큰 함수 중간에 걸리는 문제 함께 | 1B-09 | ✅ 실행 전체에 해법기 프로세스 하나(의무마다 `(reset)`+기반 재전송), `z3 --version` 성공은 PATH별 기억, `-T`는 정의 경계에서 교체로 정의마다 300초 보장. 19개 코퍼스 모든 명령 출력 바이트 동일. `verify/pass` 4개 1185→674ms. `tests/gate.rs`에 해법기 중도 종료 사례 |
| 1D-19 | MCP·트랜잭션 견고성 | 비UTF-8 경로 `unwrap`, 중첩 `mod`를 조용히 버리는 `tx_put`, 비원자적 `write_units`, 오류 경로 테스트(`tx_remove`·`tx_abort`·없는 tx·알 수 없는 도구·`build` 거부) | 1D-11 | ✅ `build` 경로 `unwrap` 제거, `put`의 `mod`·빈 코드 거부, 임시 파일 후 이름 바꾸기. `tests/mcp.rs`의 `tool_errors_over_mcp`, `tests/workspace.rs` 2건 |
| 1D-20 | 진단 변형 커버리지 | 코퍼스에 없는 메시지 변형(E0101·E0204·E0207·E0402·E0408·E0503 등)과 `replacement` 적용 테스트(E0207·E0210·E0604·E0401), E0814 코퍼스 | 1A-04 | ✅ `check/err`의 `unsupported`·`places`·`spec_mode`·`attrs`, `verify/fail/unsupported.bmb`(E0814), `fixes.rs`에 E0207·E0210·E0604·E0401. 발견: 명세 함수의 `assert` 허용 → E0503으로 수정 |
| 1D-21 | 하네스 판정 단위 테스트 | `judge_write`·`judge_navigate` 경계(빈 답, 상위 집합, `bmb_accept`·`ref_pre` 이름 충돌, 도구 한도) | 1E-03 | ✅ `bench/m1/test_harness.py`(19개, 게이트). 발견 2건 수정: 빈 정답에 빈 답은 성공, 에이전트 단위 `bmb_accept`와 숨은 판정 단위 이름 충돌 회피 |
| 1D-22 | 정규형·해시 퍼징 | 퍼저 생성 프로그램에 대해 `fmt` 멱등, 배치·주석 변경에 해시 불변, `fmt` 전후 증명·IR 동일 | 1D-02 | ✅ `tests/fmt_fuzz.rs`(재배치·파일 분할·주석·공백 변형에 정규형·해시 동일, 정규형의 증명·IR 동일). 3,000개(증명 없이)와 14개(증명) 통과, 버그 없음 |
| 1D-23 | MCP 세션 퍼징 | 임의 도구 호출 순서(트랜잭션 중첩·중단·잘못된 인자)에서 서버가 죽지 않고 작업 공간이 항상 파싱 가능한 상태로 남음 | 1D-19 | ✅ `tests/mcp_fuzz.rs`(무작위 도구 호출·잘못된 인자·트랜잭션 모델·커밋 전후 스냅숏). 400세션×200호출 통과. 발견·수정: 요청 객체가 아닌 JSON 줄에 응답이 없어 클라이언트가 멈춤 → -32600 |
| 1D-24 | 닫힌 파이프에서 패닉 | `bmb hash DIR`을 `head`로 파이프할 때처럼 표준 출력이 닫히면 `failed printing to stdout` 패닉과 역추적(2A-14a 작업 중 발견). 조용히 종료하고 CLI 테스트 추가 | 1D-10 | ✅ `src/main.rs`의 `print!`·`println!`이 닫힌 표준 출력에서 종료 코드 141로 조용히 끝남(SIGPIPE 무시는 유지: 해법기가 죽으면 오류), `mcp`도 같음. `tests/cli.rs`의 `a_closed_stdout_ends_quietly`(수정 전 실패 확인) |

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
| 1F | 퍼저 위반 0, 감사 결함 회귀 테스트와 함께 수정 | 1B-17~1B-35, 1A-10~1A-13, 1D-14~1D-21 ✅ |
| 2A | 1. 1단계 종료 | 1E-06, 1E-07, 1E-10, 1E-11 |
| 2A | 2. `m1b` 동결, 기준 해법·대조군 CI | 2A-04 |
| 2A | 3. BVB·BSB 동결, 기준 해법 CI | 2A-08, 2A-09, 2A-10 |
| 2A | 4. 미션 목표값 확정 | 2A-16 |
| 2B | 1~4. 실행·망라성·퍼저·측정, 실용성(HEALTH §5) | 2B-08, 2B-04, 2B-09, 2B-10, 2B-12 |
| 2C | 1~2. 변환 의무, 퍼저, 실용성(HEALTH §5) | 2C-02, 2C-04, 2C-06 |
| 2D | 1~2. 결정론 테스트, 증명 보존 | 2D-01~2D-13 각 행 |
| 2E | 1~2. 근거 필수·면적 집계, 미확인 표시 | 2E-03, 2E-04 |
| 2F | 효과 검사·질의 | 2F-02, 2F-03 |
| 2G | 1~2. `Vec` 모델·이동 검사, 퍼저 | 2G-03, 2G-02, 2G-05 |
| 2S | 바이트·문자열 실행·퍼저 | 2S-02 |
| 2T | 1. 검사·인코딩·코드 생성·코퍼스 | 2T-01, 2T-02, 2T-03, 2T-04, 2T-07 |
| 2T | 2. 퍼저 | 2T-05 |
| 2T | 3. 실행 프로그램 2개 이상, 종료 감사 | 2T-06 |
| 2O | 1. 검사·인코딩·코드 생성·코퍼스 | 2O-01, 2O-02, 2O-03, 2O-03b, 2O-03c, 2O-03d, 2O-04, 2O-04b |
| 2O | 2. 퍼저 | 2O-05 |
| 2O | 3. 실행 프로그램 2개 이상, 종료 감사 | 2O-06 |
| 2K | 1. 검사·인코딩·코드 생성·코퍼스 | 2K-01, 2K-02, 2K-03, 2K-04 |
| 2K | 2. 퍼저 | 2K-05 |
| 2K | 3. 실행 프로그램 2개 이상, 종료 감사 | 2K-06 |
| 2U | 1. 검사·인코딩·코드 생성·코퍼스 | 2U-01, 2U-02, 2U-03 |
| 2U | 2. 퍼저 | 2U-04 |
| 2U | 3. 실행 프로그램 2개 이상, 종료 감사 | 2U-05 |
| 2N | 1. 검사·인코딩·코드 생성·코퍼스 | 2N-01, 2N-02, 2N-03 |
| 2N | 2. 퍼저 | 2N-04 |
| 2N | 3. 실행 프로그램 2개 이상, 종료 감사 | 2N-05 |
| 2M | 1~3. 구문·정규형, 퍼저 생성기·실행 프로그램의 이주, 종료 감사 | 2M-01, 2M-02, 2M-03 |
| 2R | 1. 평가 저울 ADR | 2R-01 |
| 2R | 2. 1차 평가·대안 오프라인 비교 | 2R-02, 2R-04 |
| 2R | 3. 문법 결정 ADR | 2R-06 |
| M2-a | 1. 모든 가설에 데이터 또는 동결 도구 | T1-04, T1-05, T1-13, T0-04, 2P-01 |
| M2-a | 2. BVB 두 팔 파일럿 | T0-02, T3-03 |
| M2-a | 3. H-COMMUTE 건전성·거짓 충돌률 | T1-12 |
| M2-a | 4. BCB N=1,000 판정 | 2A-14e |
| M2-a | 5. 파일럿 실패마다 귀속 층·조치 | T1-14, T1-15 |
| M2-b | 1. 2N 통과 기준 | 2N-02, 2N-03, 2N-04, 2N-05 |
| M2-b | 2. 라이브러리 넷: 증명·공인 벡터·Rust 비교 | T4-01, T4-02, T4-03, T4-04 |
| M2-b | 3. Benchmarks Game 둘: 출력·시간·크기 | T3-01 |
| M2-c | 1~3. 정체성 해시(파급 0), 교환 관계, 타입 불변식(채택 범위) | T1-10, T1-16, T1-11, 2P-03 |
| M2-d | 1~3. 2U·`f64`·2M 통과 기준, JSONTestSuite, `f64` 벤치 | 2U-05, T4-05, T2-03, T3-04, 2M-03, 2P-04 |
| M2-d | 4. `proof fn`으로 판정기 미결정 사례(`vc-vv0039`)의 같은 뜻 명세가 증명된다 | T1-18 |
| 2H | 단계 측정(G7 BCB, G8 테스트 코드, H-ORACLE 포함), 3단계 범위 ADR | 2H-01, 2A-14f, 2H-04, 2H-06, 2H-02 |
| 4 | 격차 분석·포팅 지도 | 4A-01, 4A-02 |
| 4 | 차등 시험으로 구성 요소별 출력 동일 | 4D-02~4D-06 |
| 4 | stage2 = stage3 고정점, stage2로 게이트 통과 | 4E-01 |
| 4 | 자기 증명(미증명 면적 상한) | 4F-01 |
| 4 | stage0 씨앗 재현 | 4G-01 |
| 5 | 명세·적합성, 안정성, 재현 빌드·다중 타깃·디버그·릴리스, 도구, 생태계 판단 | 5A-01~5E-01 |

## 2단계 (M2)

페이즈와 통과 기준은 `docs/ROADMAP.md` 2단계. **"승인"이 붙은 태스크는 언어 범위를 바꾸므로 사람 승인 후 착수한다**(AUTONOMY.md §4). ADR 초안(상태 "제안")은 자율로 쓴다. 언어 페이즈의 순서는 1E-07이 정한다.

### 2A 측정 마감과 과제 확장

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2A-01 | ADR: 두 번째 과제 집합 `m1b` | 범위(M1 언어 안), 과제 수 30~40, 유형(여러 단위 작성, 큰 시드 탐색, 영향 분석·`#[uses]` 활용), 동결·결과 비교 규칙(ADR-0021 확장). 자율 채택 가능한 구현 수준 결정 | 1E-01 | ✅ ADR-0025(범위·동결·집합 혼합 금지·모델 비교 규칙) |
| 2A-02 | `m1b` 시드 작업 공간 | 단위 8개 이상, 정의 80개 이상, 모두 증명. 호출 깊이 3 이상의 경로와 `&mut` 쓰기 경로 포함 | 2A-01 | ✅ `bench/m1b/seed`: 단위 9개, 정의 116개(함수 53, 술어 63), 전부 증명. 깊이 3 호출 경로·`&mut` 쓰기 경로 |
| 2A-03 | `m1b` 과제와 기준 해법 | 과제 30~40개, 작성 과제마다 기준 해법·숨은 기준 계약·약한 대조군. 탐색 정답은 맵에서 계산 | 2A-02 | ✅ 과제 37개(작성 25, 탐색 12). 모든 작성 과제에 기준 해법·숨은 계약·약한 대조군. 영향 분석(`impact`)·폐포·여러 단위 작성 과제 |
| 2A-04 | `m1b` 동결과 CI | `bench/m1b/MANIFEST`, 하네스가 과제 집합을 인자로 받음, 게이트에서 기준 해법 전부 성공·약한 대조군 전부 실패 | 2A-03 | ✅ `harness.py --set DIR --jobs N`, `bench/m1b/MANIFEST`, 프롬프트는 m1과 동일, `summarize`가 집합 혼합 거부. 게이트: oracle 25/25·12/12, weak 0/25(약 125초 추가). `test_harness.py` 30개 |
| 2A-05 | 모델 비교 절차 | 같은 과제 집합·프롬프트로 모델 여럿을 돌려 `summarize`로 열 비교하는 절차와 README. 결과 파일 이름 규칙 | 1E-12 | ✅ `bench/m1b/README.md`(같은 집합·MANIFEST·컴파일러 커밋, 재시도 없음, 결과 파일 이름 규칙, 열 비교) |
| 2A-06 | `m1b` 실제 모델 실행 | 사용자 머신에서 실행·결과 커밋(유료 API) | 2A-04 | 🚫 사용자 실행 필요 |
| 2A-07 | `m1b` 도구 노출 실행 | 첫 실행 뒤, 새 도구(`impact`·`uses`·`lint`·`fix`)를 MCP 설명에 노출한 실행을 한 번 더 해 도구 효과를 따로 보고(ADR-0034 결정 6). 사용자 머신 | 2A-06, 2D-12 | 🚫 사용자 실행 필요 |
| 2A-08 | BVB: Verus 과제 이식 | 공개 Verus 과제(Verus-Bench, Vericoding Verus, VeriContest)에서 BMB 범위(M1) 과제를 고르고 이식. 원본 명세는 숨은 기준 계약, 출처·변환 규칙·제외 비율과 사유를 함께 커밋. 기준 해법 전부 성공 | 2A-04 | ✅ ADR-0038. `bench/bvb`: 45개(Verus-Bench 30/150, Vericoding 15), 출처·라이선스(MIT) 고정, 과제별 선택·제외 표, 변환 규칙 T1~T7·G1. 게이트: 기준 해법 45/45, 약한 대조군 0/45 |
| 2A-09 | BVB: 원본 Verus 비교 하네스 | 30~50개 표본을 원본 Verus 툴체인에서 같은 모델·조건으로 돌리는 하네스(설치 스크립트, 판정은 Verus 검증 + 같은 명세 강도 검사). 기준 해법으로 CI 확인 | 2A-08 | ✅ `bench/bvb/verus`(고정 Verus 설치 스크립트, 하네스 run/compare). 실제 Verus로 기준 45/45·약한 0/45 확인, CI 작업 `bvb-verus`. 게이트에는 대역 검증기로 하네스 논리 시험 |
| 2A-10 | BSB: 명세 강도 변이체 | `m1b` 작성 과제마다 숨은 변이체 집합(기준 해법의 의미 있는 변이, 동치 변이 제외 확인). 하네스가 에이전트 명세의 변이체 판별률을 지표로 기록 | 2A-04 | ✅ ADR-0035. `bench/m1b/mutants`(변이체 426개 생성, 186개 유지), `bmb mutate`·`emit-llvm --all-checks`, 하네스 `명세 변이체 판별률` 행. 기준 계약 186/186, 약한 대조군 45/186(24.2%), 게이트가 확인 |
| 2A-11 | 페인포인트 조사 반영 | `dev-docs/research/BMB 페인포인트 가설 검증/report.md`(2026-10-25)의 권고를 ADR 제안으로: 전제 E1·E2·E3와 원칙 4 문구 수정(사람 승인), 새 가설 H-SPEC·H-ORACLE·H-GAME·H-STABLE·H-MAINT·H-FAMILIAR·H-SCALE·H-REUSE의 반증 조건과 담당 벤치(BSB·BRB·BNB·BVB·G1) 대응, 범위 밖(N5 코드 밖 장애, N7 도구 보안) 명시 | 2A-10 | ✅ ADR-0040(사람 승인 2026-10-25), DESIGN.md §2·원칙 4·범위 밖 갱신, ROADMAP G7·가설 표, 후속 태스크 2A-12~2A-15 등 |
| 2A-12 | BVB 명세 강도·총 토큰 열 (H-SPEC·H-FAMILIAR) | BVB 요약에 1회 성공률 옆으로 명세 강도(숨은 기준 계약 함의율)와 총 토큰을 BMB·Verus 양쪽에 보고. 기준 해법·약한 대조군으로 CI 확인 | 2A-09 | ✅ `bench/bvb/verus/harness.py`: `compare`에 총 모델 토큰과 명세 강도(검증·시그니처 통과 중 원본 계약 함의) 행, 한 실행용 `strength`. 게이트 `bench bvb oracle` 45/45·`weak` 0/45, CI `bvb-verus`가 Verus 쪽 같은 값, `test_harness.py`의 `test_strength_ignores_implementation_failures` |
| 2A-13 | BMB-Impossible 과제 집합 (H-GAME) | 명세(주어진 계약)와 숨은 기준이 모순되는 과제 10개 이상 동결. 성공 = 불가능을 보고, 실패 = 게이트 통과(명세 약화·우회). 하네스가 계약 변경을 `2D-19`로 판정. 기준 해법(불가능 보고)·게이밍 대조군(계약 약화로 통과)으로 CI 확인 | 2A-04, 2D-19 | ⏳ |
| 2A-14 | BCB: 누적 검증 비용 벤치 (H-AMORT, G7) | 설계는 `dev-docs/research/테스트 실행 비용 누적 근거/report.md` §BCB. `bench/m3` 생성 코드베이스(1천·1만 정의)와 Rust 쌍둥이에 고정 시드 편집 이력(커밋 1,000개, 본문·계약·추가·삭제 비율 프로필, 툴체인 사건 0·1·4회)을 재생. 비교군: R0 전수 `cargo test`, R1 정확 결과 캐시, R2 이상적 선택, B0 캐시 없는 BMB, B1 증분 BMB(+남은 테스트). Rust 테스트는 BSB 변이체 판별률을 BMB 계약과 맞춤. 누적 CPU, 커밋당 무효화 집합 크기, 손익분기 커밋 수, 심은 결함 검출을 보고하고 사전 등록한 반증 조건으로 판정, 결과 커밋. 모델 없이 실행 | 3A-05 | 🔄 하위 태스크 2A-14a~e |
| 2A-14a | BCB 설계·이력 생성기 | 설계와 반증 조건 사전 등록(ADR), 계획 편집 기반 이력 생성·재생(두 언어 같은 편집, 커밋별 다이제스트), 커밋당 재증명 집합을 CLI로 노출, 게이트에서 모든 커밋이 증명·컴파일됨 | 3A-05 | ✅ ADR-0042, `bench/bcb/history.py`, `bench/bcb/test_bcb.py`(게이트 `bench bcb`, N=100·K=40·프로필 3), `bmb verify --stats FILE`(`tests/incremental.rs`의 `verify_stats_name_the_reencoded_definitions`). 발견: body 편집의 35~43%가 `#[uses]`를 바꿔 호출자까지 재증명(2D-22) |
| 2A-14b | BCB Rust 테스트 동등화 | 실행 정의마다 Rust 테스트(계약 `debug_assert!`·패닉 판정). 두 언어 같은 본문 위치의 W0904 변이체에서 BMB 계약이 죽이는 변이체를 Rust 테스트도 모두 죽일 때까지 결정론적 입력을 늘림. 정의별 두 판별률 보고, 게이트 N=100 | 2A-14a | ✅ `bench/bcb/rtests.py`(정의마다 테스트, 계약 `debug_assert!`·패닉 판정), `rustify.py`(BMB→쌍둥이 변환, 원본 변환이 쌍둥이와 바이트 동일), `kills.py`(라운드별 일괄 판별, 오라클 테스트로 계약 위반 여부 판정), 게이트 `bench bcb`(`contract-heavy` 커밋 40). 동등성은 BMB가 거부하고 계약을 실제로 어기는 변이체 집합에서 잰다: 불변식만 깨는 변이체(루프 시작 1·보폭 2, E0806·E0807)는 계약만 보는 테스트로 구별할 수 없어 따로 보고. N=100: 변이체 338, BMB 195 = 계약 165 + 주석만 30, Rust 165, 누락·불건전·초과 0. 커밋 200 뒤 두 프로필도 누락 0. N=1,000 초기 트리: 변이체 3,351, 계약 1,571 = Rust 1,571, 주석만 300, 누락 0 |
| 2A-14c | BCB 재생 하네스 | 다섯 팔(R0·R1·R2·B0·B1)을 커밋마다 실행해 CPU(user+sys)·벽시계·무효화 집합(재증명 정의 수, 재실행 테스트와 그것이 덮는 정의 수)을 JSONL로 기록. BMB 남은 테스트(io 정의 출력), 툴체인 사건 이중 측정(ADR-0042 §3), 기준 팔 표본화. 게이트 N=100·작은 K | 2A-14b | ✅ `bench/bcb/bcb.py`(`run`·`table`): 팔 `rust`(한 재생에서 R0·R1·R2, 빌드 공통), `b1`, `b0`(`--b0-every` 표본). 남은 테스트는 `show`마다 BMB 테스트 함수와 생성 `main`, 출력은 계획 평가값과 비교. B 팔은 `bmb build --cache --stats`(증명+빌드 한 명령, `--stats`를 `emit-llvm`·`build`에 추가). 사건 커밋은 캐시를 지우고 다시 잼. 게이트 `bench bcb`(N=100·K=5, 행·선택 포함 관계·차가운 재증명·표). N=1,000·K=30 시험 실행 3분 42초(예비, 주장 아님) |
| 2A-14d | BCB 결함 심기 | 커밋마다 편집된 실행 정의에 변이체 하나를 심은 곁가지를 각 팔의 선택 규칙으로 돌려 검출률 기록. 게이트에서 B1이 계약이 죽이는 결함을 모두 잡음 | 2A-14c | ✅ `bcb.py defects`: 커밋마다 편집된 실행 정의 하나에 W0904 변이체 하나(생존 포함, 고정 시드)를 두 언어에 심고 R0·R1·R2(선택된 테스트), B1·B0(증명 + 닿는 남은 테스트의 출력)를 기록, 곁가지는 되돌림. `table`에 검출률과 조건 4. 게이트: Rust가 잡으면 증명도 실패, 선택 포함 관계, B1 ⊆ B0. 선택 로직을 `RustSelect`·`BmbSelect`로 분리(선택 결과 전후 동일 확인). 예비(N=1,000·K=50, 주장 아님): 결함 42개 중 R2 10, B1 16 |
| 2A-14e | BCB 동결·N=1,000 측정 | 하네스·이력 MANIFEST 동결, 중단·재개 가능한 측정 계획(`suite`), N=1,000 프로필 3과 깊이 2·6 실행, 결함 심기 3 프로필, 결과·표(조건 2의 N=1,000, 조건 4) 커밋(`bench/bcb/results/`) | 2A-14d | 🔄 동결·`suite` 완료. 첫 측정은 같은 머신의 동시 빌드로 오염되어 버림(컴파일러 교체 위험·CPU 경합) → `suite`가 컴파일러 사본을 고정하고 미커밋 트리를 거부. 측정 재시작 |
| 2A-14f | BCB N=10,000 측정과 판정 | N=10,000 프로필 3 실행(`suite` 이어서), 반증 조건 1~4 판정과 보고서(`bench/bcb/results/`, `docs/ROADMAP.md` G7 현재값) 커밋 | 2A-14e | ⏳ |
| 2A-15 | 조사 수치 원문 대조 | `dev-docs/research/` 두 보고서의 "원문 미확인" 수치를 1차 원문과 대조해 표시를 갱신. 이 환경은 원문 호스트가 열리지 않아 사용자 머신 또는 네트워크 정책 변경 필요 | 2A-11 | 🚫 사용자(네트워크) |
| 2A-16 | 미션 목표값 확정 | 1E-10 결과로 ADR-0034의 잠정 목표값(G1~G9)을 한 번 확정하고 ROADMAP 미션 목표 표에 반영(2A 통과 기준 4, 통과 기준 역산으로 도출). 목표값 변경은 승인 | 1E-10 | ⏳ 승인 |

### 2B 데이터 타입

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2B-01 | ADR 초안: 구조체·열거형·`match`·`impl` 표면과 의미 | Rust 표면을 따르되 M1 규칙(정규형 하나, 명시적 타입, 겹침 없는 impl)과의 관계, 거부할 Rust 습관 목록, 진단 코드 범위. 상태 "제안" | 1E-01 | ✅ 제안 ADR-0026(채택은 승인 필요) |
| 2B-02 | ADR 초안 + 실험: 데이터 타입의 증명 인코딩 | SMT 데이터타입 대 필드 분해를 작은 질의 묶음으로 비교(`rlimit`, 반례 가독성, 결정론). 상태 "제안" | 2B-01 | ✅ 제안 ADR-0027 + `dev-docs/experiments/2B-02`(질의 109종×4회, Z3 실행 436회, 판정 전부 기대대로·시드 무관). 권고: 구조체는 필드 분해(갱신 사슬 6~36배 저렴), 열거형은 데이터타입(match·Option 사슬 1.7~3.6배 저렴) |
| 2B-03 | 파서·AST·정규형 | 구조체·열거형·`match`·`impl` 구문, 정규형 왕복, 해시 | 2B-11 | ✅ 파서·AST: `struct`·`enum`(유닛·튜플 변형), `impl T`, 이름 붙은 타입·`Option<T>`·`Result<T, E>`, 수신자 `self: T`(타입 없는 `&self`는 E0207 + 수정), 구조체 생성(`T {` 다음 `이름 :`로 블록과 구분), 유닛 변형 경로, 필드 읽기, `*p`, `match`·변형 패턴(와일드카드 갈래는 E0207 + 수정, 구문 복구). 정규형: 타입 이름순 + 바로 뒤 `impl`(함수 이름순), 여러 줄 `match`, 구조체 생성을 담은 조건은 괄호(유효 Rust). 검사기는 2B-04 전까지 E0207로 거부. `tests/fmt.rs`의 `data_types_have_a_normal_form`(정규형 고정·멱등), 코퍼스 메시지 갱신. 타입·`impl` 함수 해시와 생성 식 필드의 선언 순서 정렬은 타입 표가 필요해 2B-04로 |
| 2B-04 | 검사기 | 필드·생성자·`match` 망라성·이동 없는 값 의미, 진단 코퍼스. 2B-03에서 넘어옴: 타입·`impl` 함수(`unit::T::f`)의 해시, 구조체 생성 식 필드의 선언 순서 정렬(정규형), 내장 변형 `Some`·`None`·`Ok`·`Err` 예약 | 2B-03 | ✅ 하위 2B-04a~d. 생성 식 필드 정렬만 2B-13(승인)으로 |
| 2B-04a | 검사기: 타입 선언·이름·시그니처 | 구조체·열거형 등록(단위 이름공간 공유), 필드·페이로드 타입(참조·슬라이스·비스칼라 배열 거부), 재귀 타입, 내장 이름 예약, `impl` 위치·개수, 수신자 규칙(`self`), 정규 타입(전체 경로), 시그니처·지역 타입, `T::f` 해석, 타입·`impl` 함수 해시. 증명·코드 생성은 2B-05·2B-06 전까지 E0814 | 2B-03 | ✅ `check.rs`의 `collect_types`·`resolve_types`·`check_type_cycles`·`canon`·`receiver_rule`, 새 코드 E0305·E0308·E0309·E0310·E0420·E0421(`tests/check/err/data_types.bmb`), `fmt::hashes`에 타입(한정 정규형, 이름 `_`)과 `unit::T::f`, 증명기 `data_type_use`·`emit-llvm`의 거부(E0814). `tests/cli.rs`의 `data_types_in_signatures_check_but_do_not_prove_yet` |
| 2B-04b | 검사기: 값과 필드 | 구조체 생성(필드 전부, 정규형은 선언 순서), 필드 읽기·쓰기, `*p`, 변형 생성(`T::V(..)`, 유닛 변형 경로, `Some`·`None`·`Ok`·`Err`의 기대 타입), 명세 `==`(실행 `==`는 거부), 진단 코퍼스 | 2B-04a | ✅ `check.rs`의 `struct_lit`·`field_type_of`·`variant_value`·`enum_prefix`·`builtin_variant`·`place_of`(필드·`*p` 대입), 실행 `==`·`!=` 거부. 새 코드 E0306·E0307·E0422·E0424·E0425(`tests/check/err/data_values.bmb`), 통과 예 `tests/cli.rs`의 `data_values_fields_and_variants_check`. **편차:** 생성 식 필드의 선언 순서 정렬(ADR-0026 §정규형)은 하지 않는다. 필드 식에 효과(`io` 호출)가 있으면 정렬이 평가 순서를 바꿔 의미가 달라진다. 지금은 적힌 순서를 유지하고, 규칙 결정은 2B-13(승인) |
| 2B-13 | 구조체 생성 필드 순서 규칙 결정 | 2B-04b 발견: 정규형의 필드 정렬은 효과 있는 필드 식의 평가 순서를 바꾼다. 안: (가) 검사기가 선언 순서를 요구하고 `replacement`로 고친다(정렬 불필요, ADR-0026 §1 "모든 필드를 선언 순서로"와 같음), (나) 적힌 순서 유지(정규형 둘 이상 허용), (다) 효과 없는 필드 식만 정렬. 채택 ADR-0026 문구 개정이라 승인. 권고 (가) | 2B-04b | ⏳ 승인 |
| 2B-04c | 검사기: `match` | 패턴 해석, 망라성(빠진 변형·중복 갈래·선언 순서), 페이로드 바인딩(셰도잉 금지), 갈래 타입 일치, 실행·명세 문맥, 진단 코퍼스 | 2B-04b | ✅ `check.rs`의 `match_expr`·`arm_variant`·`match_variants`(`Option`은 `None, Some`, `Result`는 `Ok, Err` 순서): 열거형이 아닌 값(E0423, `&E` 매개변수는 `*p` 수정), 빠진 변형·두 번째 갈래·선언 순서(E0426), 페이로드 개수(E0427, `replacement`), 경로 없는 변형 이름(E0307, Rust에서는 전부 받는 바인딩이 된다), 다른 열거형의 패턴(E0401). 갈래 타입은 기대 타입, 없으면 리터럴이 아닌 첫 갈래. 코퍼스 `tests/check/err/match.bmb`, 통과 예는 `tests/cli.rs`의 `data_values_fields_and_variants_check`(실행·명세·문장 위치·중첩). 함께 고침: 변형을 만드는 `spec fn`(정수 시그니처)을 드러내는 정의가 Z3로 가서 E0812(풀이기 오류)가 나던 것 → `verify.rs`의 `data_spec_fns`(호출 폐포)로 E0814, 풀이하지 않음(`a_spec_building_variants_is_not_sent_to_the_solver`) |
| 2B-04d | 검사기: 데이터 타입의 Rust 습관 진단 | ADR-0026 §5 표(E0220~): `#[derive]`, `Option::Some` 전체 경로, 필드 축약, 갱신 구문, `if let`, `p.f()`, `&self`, `(*p).x`, 구조 분해 `let`. 기계적 수정이 있는 것은 `replacement` | 2B-04c | ✅ E0220~E0233(E0212~E0219는 제안 ADR-0028·0029·0031이 씀). 기계적 수정(`replacement`): `#[derive]`(E0220, 그 밖의 타입 속성은 E0605), `Option::Some` 등(E0221, 식·패턴), 필드 축약(E0222), 갱신 구문(E0223, 필드 전부), `_ =>`(E0224)와 `if let`(E0225): 선언 순서의 `match` 전체, 메서드 호출(E0227, 시그니처의 참조 종류로 `&`/`&mut`), 타입 없는 수신자(E0228, `impl` 타입), 구조 분해 `let`(E0230, 필드 타입), `(*p).x`(E0231). 수정 없음: `while let`·`let .. else`·패턴 `let`(E0226), `.unwrap()`·`.expect()`(E0229), 가드·or·리터럴·중첩·구조체 패턴(E0232, 갈래는 나머지를 덮어 연쇄 오류 없음), 튜플·튜플 구조체·구조체형 변형(E0233). 파서가 `if let`을 `match`로, 갱신 구문·구조 분해를 AST(`StructLit.rest`, `Stmt::LetFields`, `PatOrigin`)로 보존해 검사기가 수정을 쓴다. 코퍼스 `tests/check/err/habits.bmb`, `tests/fix_cmd.rs`의 `data_type_habits_are_fixed_into_a_checked_program`(`bmb fix` 한 번에 검사 통과), P3 0.552 → 0.637 |
| 2B-05 | 증명 인코딩 | 2B-02 결정대로. 필드·갈래별 의무, 반례에 구조 값 표시 | 2B-02, 2B-04d | ✅ 하위 2B-05a~d |
| 2B-05a | 증명 인코딩: 구조체(필드 분해) | ADR-0027 §1: 필드별 값, 매개변수 범위 사실, 필드 읽기·쓰기(중첩, `*p`), 생성 식, `&mut` 인자·루프 havoc, 병합, 명세 `==`·`spec fn` 인자 펼침, 반례 `Point { x: 1, y: 2 }`. 열거형·데이터 배열은 E0814 유지 | 2B-04d | ✅ `verify.rs`의 `Val::Struct`·`flatten`·`build`·`leaves`·`set_at`·`place`·`eq_val`·`ite_val`, 검사기 표 `Tables.adts`·`Tables.locals`(정규 지역 타입), 게이트 `Gate::Prover`(열거형·데이터 배열·구조체 반환 `spec fn`만 E0814)와 `Gate::Codegen`(2B-06 전 전부). 코퍼스 `tests/verify/pass/structs.bmb`(8 정의 증명), `tests/verify/fail/structs.bmb`(거짓 주장 6개 실패, 반례 구조 표기). 함께 고침: (1) `T::f` 호출이 검증기에서 해석되지 않아 내장 함수로 처리되던 것(피호출자의 `&mut` 효과 무시, 건전성) → 검사기와 같은 해석, 해석 못 한 호출은 `print_i64` 외에 단언. (2) 증분 키가 `T::f` 피호출자 계약과 쓰인 타입의 필드 타입을 읽지 않던 것(낡은 증명 재사용) → 정규형이 `T::f`를 전체 경로로(`FnSig.in_impl`), 키에 타입 선언 줄(필드·페이로드 폐포). `tests/incremental.rs`의 `data_type_changes_reencode_their_readers`(수정 없이는 실패). (3) `emit-llvm`이 데이터 타입이면 검증 전에 멈추던 것 → 검증 후 코드 생성만 E0814 |
| 2B-05b | 증명 인코딩: 열거형·`match`(데이터타입) | ADR-0027 §3~6: 인스턴스별 `declare-datatypes`, 변형 값, 실행·명세 `match`(판별자 경로 조건), 페이로드 범위는 `match`에서, 반례 `Some(992)` | 2B-05a | ✅ `verify.rs`의 `datatypes`(프로그램이 쓰는 인스턴스, 의존 순서, 페이로드 구조체는 잎 필드로 펼침), `needed_decls`(질의가 이름 대는 정렬과 그 의존만), `variant_value`·`payload`·`match_arms`(실행: 갈래마다 판별자 아래 `branch`, 마지막 갈래는 나머지의 부정)·`spec_match`(판별자 `ite`), 반례 `show_variant`(`Some(-1)`, `Shape::Rect(Point { .. }, Point { .. })`). 배열을 담은 열거형의 명세 `==`는 E0814(길이 밖 원소). 게이트는 데이터 타입 배열과 구조체를 돌려주는 `spec fn`만 E0814. 코퍼스 `tests/verify/pass/enums.bmb`(9 정의: 페이로드 범위, 중첩 `Option<Shape>`, 구조체 안 열거형의 `&mut` 필드 인자, 루프), `tests/verify/fail/enums.bmb`(거짓 주장 6개 실패) |
| 2B-05c | 증명 인코딩: 데이터 타입 배열 | ADR-0027 §2: 구조체 배열은 필드별 배열(길이 공유), 열거형 배열은 `(Array Int E)`. `&[Point]` 매개변수 | 2B-05b | ✅ `Val::Arr`(원소 모양에 잎마다 배열 항: `select_val`·`store_val`), `leaves`·`build`의 구조체 원소 펼침, 읽은 원소의 잎 범위 사실, `a[i].x = v`·`a[i] = p`(장소 단계), 루프 havoc·`&mut` 인자는 길이 유지, 반례 `[Point { x: 13, y: 7 }] (len 1)`·`[Shape::Dot, ..]`. 열거형 슬라이스는 기존 `Val::A`(원소 정렬이 데이터타입). 함께 고침: 데이터 타입 원소의 길이 상한을 `isize::MAX`로(1바이트 원소가 있을 수 있어 `isize::MAX / 8`은 과한 가정). 원소에 배열이 있는 슬라이스는 E0814(2B-05d). 코퍼스 `tests/verify/pass/slices.bmb`(5), `tests/verify/fail/slices.bmb`(거짓 주장 5개 실패) |
| 2B-05d | 증명 인코딩: 마무리 | 구조체를 돌려주는 `spec fn`, 배열 필드를 가진 구조체의 명세 `==`, 기저 인스턴스 재시도(ADR-0023)의 데이터타입 항, `stability`·`rlimit_max` 재측정(ADR-0027 결과 절) | 2B-05c | ✅ 구조체 결과는 잎마다 SMT 함수(`spec_funs`, `|geo::diag.x|`)와 정의 공리, 질의 표시는 접두사(`spec_key`). 배열 필드의 명세 `==`는 길이 아래 원소의 한정 동등(`eq_val`). 기저 인스턴스는 데이터타입 항 인자를 그대로 대입한다(`verify.rs` 단위 시험 `ground_instances_take_datatype_arguments`). 재측정: `solver_stability.py --variants 4` 의무 812개(718 → 812, 2B-05 코퍼스 포함) × 변형 16, 불안정 0. `rlimit_max` 최대 48,712(`enums.bmb`의 `area`, 한도의 0.97%, V5 0.0049 → 0.0097). 남은 E0814: 원소에 배열이 있는 데이터 타입 슬라이스, 배열 페이로드 열거형의 명세 `==`(길이 밖 원소가 SMT 동등에 들어감) — 쓰는 과제가 생기면 연다. 구조체 결과의 기저 인스턴스는 만들지 않는다(완전성만) |
| 2B-06 | 코드 생성 | 구조체 배치, 태그 있는 열거형, `match` 분기 | 2B-04d | ✅ `codegen.rs`: `type_defs`(쓰는 구조체·열거형 인스턴스마다 이름 붙은 타입, 열거형은 `{ i64, [K x i64] }` + 변형 보기 타입, K는 C 배치 계산 `layout`), `CVal::Ptr`·`Slot::Data`, `put`·`copy`(`memmove`, 크기는 `getelementptr null, 1` 상수식)·`read`·`field_ptr`·`variant`·`place`·`match_expr`(`switch` + 바인딩 복사)·데이터 결과는 `ret.out`, `&T`는 주소, 값 인자는 사본, `impl` 함수 수집과 `T::f` 해석. 실행 코퍼스 `tests/run/shapes.bmb`(15줄)·`values.bmb`(13줄: 페이로드·바인딩 복사, 필드 `&mut`, 값 인자·결과 복사, 필드 간 대입), 손 계산과 일치, -O0/-O1/-O2/-O3 출력 같음, `opt -passes=verify`, valgrind 오류 0. 함께 고침: (1) 검사기가 `&T`·`&mut T`(데이터) 인자를 검사하지 않던 것(타입·가변성·`&mut` 별칭 E0410 없음, `noalias` 위반 가능) → `data_ref_arg`, 코퍼스 `data_values.bmb`. (2) 증명기가 구조체 값의 `#[trigger]`에서 멈추던 것 → 잎마다 패턴 대안. 남은 한계: 지역 배열 `[Point; N]`은 검사기가 막아 데이터 슬라이스를 실행 코드에서 만들 수 없다(매개변수로만), 필드 배열 원소 대입 `s.w[i] = v`도 검사기가 막는다(전체 대입은 됨) — 2B-12 마찰 표 후보 |
| 2B-07 | 맵·질의 확장 | 타입 정의를 L0~L2와 `impact`에 포함, 하위 호환(ADR-0020). 2B-05a 발견: 맵 색인에 `impl` 함수(`unit::T::f`)가 없다(`query impact geo::Point::f`가 "unknown definition"). 색인·호출 간선(`T::f`, `map::resolve`는 2B-05a에서 `in_impl`로 고침)·`usable_proofs` 성분에 넣는다 | 2B-04a | ✅ `map.rs`: 색인에 `impl` 함수(`unit::T::f`, 단위 `defs`에도)와 타입(`TypeEntry`: L0 `def`·`def_kind` `struct`/`enum`·해시, L1 `signature`가 선언 정규형), L2 `types` 간선(시그니처·지역·식의 값 타입), 변형 생성은 `calls`가 아님, `query impact T`(`types` + 표면에 쓰는 정의에서 한 단계), `surface`·`edges`·`find`가 타입을 받고 `slice`는 이름 쓴 타입의 선언을 함께 준다. `fmt::type_text`·`type_hashes`. `tests/map.rs`의 `data_types_are_mapped`, `tests/incremental.rs`의 타입 변경 재증명 ⊆ `impact` |
| 2B-08 | 코퍼스 | `check`·`verify`(통과·실패)·`run` 예제 | 2B-05, 2B-06 | ✅ 2B-04~06에서 함께 더함: `tests/check/err/` `data_types`·`data_values`·`match`·`habits`(새 코드 E0220~E0233·E0305~E0310·E0420~E0427 전부), `tests/verify/pass/` `structs`·`enums`·`slices`, `tests/verify/fail/` 같은 셋(거짓 주장 실패와 Rust 표기 반례), `tests/run/` `shapes`·`values`(`.out` 손 계산). `tests/fix_cmd.rs`의 습관 수정 왕복 |
| 2B-09 | 건전성 퍼저 확장 | 구조체·열거형 값과 `match`를 생성·해석. 60배치 위반 0 | 2B-08 | ✅ `tests/soundness_data.rs`: 구조체 생성(필드 순서 무작위)·필드 갱신·조건으로 고른 변형(유닛·정수·구조체 페이로드)·`match`·구조체 결과(`r.a`·`r.b` 사후조건), 독립 해석기 대조(검증된 함수 무결함, 반례 재현, 두 빌드 출력). `BMB_SOUND_ITERS=60`: 함수 360, 검증 64(실행 193), 반례 재현 546, 컴파일 실행 3,672, 위반 0(113초, `bmb` 명령줄 경유: S5 유지). 검출 확인: 증명기의 구조체 생성 필드 정렬을 빼면 seed 1에서 실패. 게이트는 4배치, nightly는 200배치(`nightly.yml`) |
| 2B-10 | 과제 추가와 측정 | `m2` 과제에 데이터 타입 과제 추가, 측정 | 2B-09, 2A-04 | ⏳ |
| 2B-11 | ADR-0026 채택 | 2R-02 평가와 2R-04d 비교(`syntax-eval-2.md`)를 반영해 제안 ADR-0026을 갱신·채택(채택 계획: 2B 착수 때, 2026-10-24 승인). 2B-01 초안과 다른 점을 ADR에 적음 | 2B-01, 2R-04d | ✅ ADR-0026 채택(2026-10-25): 2R-04d를 반영해 내장 변형 `Some`·`None`·`Ok`·`Err` 맨이름(예약), 수신자 `self: &T`(첫 매개변수가 그 타입이면 이름은 `self`, 제안의 `this` 대체). 메서드 호출은 거부 + 기계적 수정 유지(입력 별칭은 쓰기 표면 정책이라 2R-04e·2R-06). ADR-0027 §6 반례 표기 대체, DESIGN.md §4.2 예시 갱신 |
| 2B-12 | 2B 실용 프로그램과 종료 감사 | 구조체·열거형을 쓰는 실행 프로그램 2개 이상을 `tests/run/`에 추가(HEALTH.md §5, P1 래칫), 마찰 표 처리 칸, `health.py table` 스냅샷 `dev-docs/health/2B.md`, 통과 기준 하나씩 확인 | 2B-09 | ✅ 감사 `dev-docs/health/2B.md`: 기준 1~3 충족, 기준 4는 2B-10(사용자 트랙) 대기라 2B는 열려 있음. 실행 프로그램 2개(`tests/run/shapes`·`values`, P1 6 → 8). 마찰 표(`m1.md` §6)에 2B 행 6개, 처리 칸 모두 채움(2B-13~2B-16 태스크, 2R-02 입력, 유지) |
| 2B-14 | 데이터 타입 원소의 지역 배열 | 2B-12 마찰: `[Point; N]`·`[Shape; N]` 지역·필드 허용(검사기), 배열 값의 증명(ADR-0027 §2의 필드별 배열)과 코드 생성(원소 배치), 실행 코퍼스 | 2B-12 | ✅ 검사기 `data_array`·`literal_elem`: 지역·필드·페이로드·리터럴·반복의 구조체·열거형 원소 허용, 원소가 배열·`Vec`을 담으면 E0402, 값 매개변수 `[T; N]`은 E0402(`&[T]`로). 증명기: 고정 길이 `N`(`fresh_val`), 리터럴은 새 잎 배열에 원소별 `store`, 구조체 반복은 잎마다 상수 배열. 코드 생성 `init_data_array`(원소를 복사해 둔 뒤 채움, 반복은 루프, `let`·`=`는 통째 복사)와 필드 배열 원소 장소. 실행 `tests/run/points`(손 계산 출력, 런타임 검사 0, valgrind 깨끗), `verify/pass/structs` `arrs`(복사·반복·열거형 리터럴·필드 길이), `verify/fail/structs`(지역·필드 원소 범위 밖 반례 `i = 2`, 복사 뒤 옛 값), `check/err/data_arrays`. 변이: 리터럴의 `detach` 제거는 실행 출력(505≠510), 고정 길이 제거는 `arrs::last`가 잡음 |
| 2B-15 | 필드 배열 원소 대입 | 2B-12 마찰: `s.w[i] = v`·`p.q.w[i] = v` 장소 허용(검사기 `place_of`), 증명기·코드 생성 경로 시험 | 2B-12 | ✅ 검사기 `place`가 필드 아래 인덱스 대입을 받고 바탕의 타입을 기록(대입 대상은 타입이 없었다). 증명기·코드 생성은 손대지 않음(장소 단계 그대로). 실행 `tests/run/fields`(지역·중첩 필드·`&mut` 매개변수, 손 계산 10·20·9), 실패 `structs.bmb`의 `set_cell`(반례 `i = 3`) |
| 2B-16 | `&T`를 `T` 자리에 줄 때 `*e` 수정 | 2B-12 마찰: `moved(old(self), ..)`의 E0401에 기계적 수정 `*old(self)` | 2B-12 | ✅ `check::deref_fix`: 구조체·열거형 `T` 자리의 `&T`/`&mut T`(호출 인자, `let` 초기값)에 E0401과 교체 `*e`. 왕복 시험 `reference_where_a_value_is_expected_is_fixed`(고친 프로그램이 증명까지 됨) |

### 2C 산술과 변환

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2C-01 | ADR 초안: 정수 변환의 의미 | 범위 의무가 붙는 변환 대 `Option` 반환(2B 의존 여부), `checked_*`·`saturating_*`. 상태 "제안" | 1E-01 | ✅ 제안 ADR-0028(`try_from(..).unwrap()` + 범위 의무 E0815, `print_u64`) |
| 2C-02 | 변환 구현 | 검사·증명 의무·코드 생성(증명되면 검사 없음), 코퍼스 | 2C-05 | ✅ 검사(`check::conversion`: `.unwrap()`→`T`, `.ok()`→`Option<T>`, E0205 교체·E0212·E0213·E0412·E0513), 증명(E0815 = 대상 범위, `.ok()`는 범위로 `Some`/`None`), 코드 생성(증명되면 명령 없음, 아니면 `icmp sge`+트랩), 내장 `print_u64`. 코퍼스 `tests/check/err/conversions`, `tests/verify/{pass,fail}/conversions`(통과 7정의, 반례 4), 실행 `tests/run/conversions`·`search`(`show`·`show_size` 세는 루프 제거, 출력 동일). 테스트 `an_unproved_conversion_traps_exactly_outside_the_range`(경계값 4개), `conversion_habits_are_fixed_into_a_checked_program`. `--all-checks`와 증명 빌드 출력 같음(검사 4 대 2) |
| 2C-03 | `checked_*`·`saturating_*` | 구현과 코퍼스 | 2C-02 | ✅ 검사(`checked_*`→`Option<T>`, 정수 아닌 수신자 E0404, `pow`·`abs`·`overflowing_*` 안내 갱신), 증명(`verify::int_method`: 포화 = 범위로 자르기, 검사 = `ite(범위, Some, None)`, `div`·`rem`은 0·`MIN/-1`·`MIN%-1` 제외, 나눗셈 사실), 코드 생성(`*.sat.i64`, `*.with.overflow`+`select`, 나눗셈은 1로 대체). 코퍼스 `tests/check/err/arith_methods`, `tests/verify/pass/arith_methods`(6정의, 경계값 `assert`), `fail/arith_methods`(반례 3: `u64` 포화 합, `MIN*-1`, `MIN%-1`), 실행 `tests/run/arith_methods`(경계값 28개, Rust 의미와 손으로 대조, -O0·-O3 같음, IR `opt -passes=verify`) |
| 2C-04 | 퍼저 확장 | 부호 없는 산술 퍼저(1B-33)에 변환 추가, 위반 0 | 2C-02 | ✅ `tests/soundness_unsigned.rs`에 `saturating_*`, `checked_*`(`match`와 `None` 기본값), `i64`·`usize` 경유 변환(`i64::MAX` 초과는 결함), 부호 있는 차를 `u64`로(음수는 결함)와 그 `.ok()`형 추가, `main`은 결과를 `print_u64`로 출력해 해석기 값과 비교(전에는 일치 여부 0/1). 60배치: 480개 중 증명 57개(ADR-0028 형태 포함 15개) 342회, 반례 659개 재현, 컴파일 3,800회 일치, 위반 0. 변이 검사: 증명기의 E0815 의무를 빼면 seed 3에서 "verified faults", `.ok()`의 `icmp sge`를 `sgt`로 바꾸면 seed 1에서 빌드 불일치로 잡힘 |
| 2C-05 | ADR-0028 채택 | 2B 표면(2B-11)에 맞춰 제안 ADR-0028(`Option` 반환 여부 포함)을 갱신·채택(채택 계획: 2C 착수 때) | 2C-01, 2B-11 | ✅ ADR-0028 채택(2026-10-25): `checked_*`는 `Option<T>`, 값 경로는 융합형 `T::try_from(x).ok()`(`Option<T>`, 의무 없음, 유효 Rust), 단독 `try_from`은 E0412(BMB에 오류 타입 없음), 진단 번호 E0212·E0213·E0412·E0513·E0815 확정(빈 번호 확인). 표면(`as` 대 `try_from().unwrap()`)은 2R-04c 결과와 함께 2R-06(승인)으로 |
| 2C-06 | 2C 실용 프로그램과 종료 감사 | 변환·`checked_*`를 쓰는 실행 프로그램 2개 이상(M1 마찰: `u64`·`usize` 출력·혼합 해소 확인), `dev-docs/health/2C.md`, 통과 기준 확인 | 2C-03, 2C-04 | ✅ `dev-docs/health/2C.md`: 기준 1·2 충족, 2C 완료. 실행 프로그램 `tests/run/stats`(새, 의무 전부 증명, 손 계산 출력, valgrind 0)·`conversions`·`arith_methods`(P1 8 → 11), `search`의 세는 루프 제거(M1 마찰 해소). `m1.md` §6에 2C 행 4개 |

### 2D 도구

1단계 태스크가 모두 막혀 당겨 온 것(AUTONOMY.md §3.1). 언어 범위를 바꾸지 않는다.

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2D-01 | 해시 키 증명 캐시 | 정의의 본문 해시와 의존 계약 해시를 키로 증명 결과를 저장해 바뀌지 않은 정의는 Z3를 부르지 않음. 캐시 유무로 판정이 같음을 테스트 | 1D-03, 1B-11 | ✅ `cache.rs`, `bmb verify --cache DIR`, `tests/cache.rs`(코퍼스 출력 동일, 재실행 시 증명된 의무는 해법기 생략, 펼친 정의가 바뀌면 무효). `run/search.bmb` 1.0초→0.02초. MCP·`build`에는 아직 연결 안 함 |
| 2D-02 | 사전 영향 분석 질의 | 정의의 계약을 바꾸면 다시 증명해야 하는 정의 집합을 반환하는 `impact` 질의. CLI·MCP 노출, 결정론 테스트 | 1D-06 | ✅ `query impact DEF [body or iface]`, `tests/impact.rs`(본문 변경마다 SMT 스크립트가 바뀐 정의 ⊆ 보고 집합). MCP 설명문은 1E-10까지 유지 |
| 2D-03 | 증명 캐시를 `build`·`emit-llvm`·MCP에 연결 | CLI `--cache DIR`, MCP는 작업 공간의 `.bmb-cache/`. 캐시 유무로 출력·IR 동일 테스트 | 2D-01 | ✅ `lib::prove`, `emit_llvm_sources_cached`·`map_request_cached`, `tests/cache.rs`(IR·맵 동일), oracle 15/15 |
| 2D-04 | 기계적 수정 적용 | `bmb fix`(CLI)가 진단의 `replacement`를 적용하고 재검사. DESIGN §4.1 대응 1, §8.2 Rust 습관 교정. MCP 노출은 1E-10 뒤 | 1A-06 | ✅ `bmb fix [--write or --print]`, `lib::fix_sources`(라운드 반복, 겹침 없는 결정론적 적용). `tests/fix_cmd.rs`(Rust 습관 8개가 2라운드에 정리, 멱등) |
| 2D-05 | `#[uses]` 자동 채움 | 실패 의무의 펼침 후보 조합을 결정론적으로 시도해 최소 `#[uses]`를 `replacement`로 제안(ADR-0017 결과, DESIGN §8.2) | 1D-13 | ✅ `bmb uses [--write]`, `uses_fill.rs`(진단 힌트 + 펼친 본문의 폐포, 단일 제거 최소화, 정의당 검증 32회 상한). `tests/uses_fill.rs` 8건(거짓 성공 없음, 결정론, 멱등) |
| 2D-06 | 중복 정의 질의 | 본문 해시가 같은 정의 묶음(ADR-0019 §2, DESIGN §7.2) | 1D-06 | ✅ `query duplicates`, `tests/map.rs`. 자기 재귀 정의는 본문 해시에 전체 경로가 남아 묶이지 않음(문서화) |
| 2D-07 | 닫힌 세계 코드 생성 | `main` 외 함수를 `internal`로 방출(DESIGN §8.1). 벤치로 효과 확인 | 1C-01 | ✅ `main` 외 모든 함수 `internal`. 벤치 차이는 잡음 범위(±10%, 동시 실행 부하), 바이너리 104~184B 감소. `only_the_entry_point_is_external` |
| 2D-08 | 계약 점검 | 만족 불가능한 `requires`(SMT로 공허), 어떤 증명도 쓰지 않는 사후 조건 보고(DESIGN §8.2 계약 조정). 새 진단 코드 | 1B-12 | ✅ `bmb lint`(옵트인, `check`·`verify` 출력 불변): W0901 함께 성립할 수 없는 전제(SMT unsat), W0902 자기 증명에 불필요한 전제, W0903 어떤 호출자도 쓰지 않는 사후 조건. 재검증 각 32회 상한. 경고 심각도 도입(`W` 코드). `tests/lint.rs` 8건. 변이 기반 약한 명세 검사는 미구현 |
| 2D-09 | 변이 기반 약한 명세 검사 | `bmb lint`에 본문 변이(연산자·상수·비교 뒤집기)를 적용해 여전히 증명되면 "명세가 이 변경을 구별하지 못함" 보고(새 W 코드). 변이 수 상한, 결정론 | 2D-08 | ✅ `bmb lint`에 W0904(본문 단일 변이가 여전히 증명되면 보고). 변이 64개 상한, 결정론. `tests/lint.rs`(약한 명세에서 생존, 정확한 명세에서 전멸, 상한). `run/search.bmb`: 변이 33개 중 7개 생존 |
| 2D-10 | 계약 방화벽 증분 빌드 | 정의별 IR·맵 행을 본문 해시+의존 계약 해시로 재사용. 캐시 유무로 출력 동일 테스트, 큰 입력에서 시간 측정 | 2D-03 | ✅ ADR-0032(키: 컴파일러 식별자+해법기 머리말+정의 원문·정규형+이름 붙은 정의의 iface/본문 해시), `incremental.rs`, `build.rs`(소스 다이제스트). 증명·IR 재사용, 감사 모드로 재계산 대조 불일치 0. 360정의: 캐시 없음 3.6초 → 따뜻한 `verify` 30ms, 본문 하나 변경 75ms. `tests/incremental.rs`. 맵 행 재사용은 안 함(증명 없이 싸게 재생성) |
| 2D-11 | 효과 경로 질의 | `query effects DEF`: `io`에 닿는 호출 경로(DESIGN.md §7.2 효과 뷰). 결정론 테스트 | 1D-06 | ✅ `query effects [DEF]`: 내장 `io` 연산까지 최단 경로(사전순 동률 해소), 증명 없이 동작. `tests/map.rs`, `tests/no_prove.rs` |
| 2D-12 | 새 도구의 MCP 노출 | `fix`·`uses`·`lint`·`impact`·`duplicates`를 MCP 도구 설명에 추가 | 1E-10 | ⏳ |
| 2D-13 | 슬라이스 예산 | 예산 대비 비율 보고, 값은 1E 결과로(ADR-0016 O6). 빌드 거부 승격은 별도 승인 | 1E-07 | ⏳ |
| 2D-14 | 질의 머리말 축소 | 의무마다 실제로 닿는 `spec fn` 선언만 보냄(지금은 전부). `spec fn` 하나의 추가·시그니처 변경이 모든 정의를 다시 인코딩하게 만드는 문제(2D-10 한계) 해소. 코퍼스 판정 불변, rlimit 변화 보고 | 2D-10 | ✅ ADR-0033: 의무마다 참조하는 `spec fn` 선언만. 코퍼스·시드 출력 바이트 동일(rlimit 포함), m1b 질의 스크립트 1.99MB→0.75MB. 증분 키는 도달 가능한 선언으로. `adding_an_unrelated_spec_fn_reencodes_nothing` |
| 2D-15 | `lint`·`uses`의 증분 캐시 사용 | 두 도구의 재검증이 2D-10 캐시를 씀. 출력 동일 테스트, 시간 측정 | 2D-10 | ✅ `lint`·`uses`가 증분 세션·증명 캐시 사용(`--cache`), `verify_defs`는 첫 실패에서 멈춤. lint `search.bmb` CPU 11.7→3.8초, `uses` 따뜻한 캐시 0.01초. 출력 동일 테스트 |
| 2D-16 | 변이 연산 확장 | W0904에 `*`·`/`·`%`·`&&`·`or` 교체, 인덱스 ±1 추가. 상한·결정론 유지 | 2D-09 | ✅ `*`→`+`, `/`↔`%`, `&&`↔`or`, 인덱스 ±1. `tests/lint.rs`의 `OPS`(변이 10개 중 6개 생존). `search.bmb` 변이 48개 중 9개 생존 |
| 2D-17 | 질의 범위 좁히기(BNB가 찾은 빈틈) | `duplicates DEF`(한 정의의 쌍둥이만), `edges`의 종류 필터(`calls`만 등), 정의 집합에 대한 `effects`. BNB 오라클 토큰으로 효과 측정 | 3A-05 | ⏳ |
| 2D-18 | 고칠 길이 없는 오류 조사 | 오류 진단 중 반례·수정 제안이 붙은 비율 P3 = 0.46(2026-10-25). 코드별로 도움이 없는 오류를 나열하고, 기계적 수정이 가능한 것에 `fix`를 붙여 P3를 올림. 목표 비율은 조사 뒤 기준선으로 | INF-09 | ✅ 조사: 코퍼스 오류 153개 중 도움 없는 82개를 코드별로 나열. 기계적으로 고칠 길이 있는 것에 수정 제안: E0605(속성 제거, 대체 텍스트 ""), E0301(보이는 지역 이름 중 편집 거리 2 이내 유일한 것, 대체 텍스트), E0509·E0510·E0603·E0602·E0701·E0103(한 토큰 삽입 안내, 메시지만). P3 0.4641 → 0.5359(기준선 갱신). 남은 것은 M1에서 고칠 길이 없는 표현 한계(E0503 `forall`, E0401 정수 혼합(2C), E0402 배열 반환(2G), E0207 제네릭, E0406·E0105 리터럴 범위, E0410 별칭 등). `tests/cli.rs`의 `an_unknown_local_suggests_the_nearest_one`, 코퍼스 기대 출력 |
| 2D-24 | 한 번에 쓰고 증명하는 쓰기 도구 | 파일럿 발견(`bench/pilot/README.md`): BVB에서 BMB는 쓰기 한 번에 트랜잭션 호출 넷(`tx_begin`·`tx_unit`·`tx_put`·`tx_commit`)과 `verify`를 따로 써서 모델 차례가 Verus의 2.9배(452 대 156), 비용이 2.1배다. 한 호출로 단위 생성·정의 넣기·커밋·증명을 하고 진단·반례를 함께 돌려주는 도구(트랜잭션 의미는 그대로, ADR-0009)를 더하고, 여러 단계 트랜잭션은 남긴다. MCP 도구 구성은 동결 측정 조건이므로 과제 집합의 새 판과 함께(ADR-0021). 확인: BVB 파일럿의 호출·차례·비용 비가 Verus 수준 | T0-02 | ✅ `compiler/src/mcp.rs` `write`(`bmb mcp --write-tool`에서만 보임, 동결 집합의 도구 목록은 그대로), 시험 `write_tool_commits_and_proves_in_one_call`, 집합별 `set.json`(`mcp_flags`, `hide_tools`). 파일럿 `bench/m1b-write`: 37/37 그대로, 도구 호출 −46%, 모델 차례 −40%, 비용 −22%(작성 과제 −27%). BVB·BRB의 같은 판은 `T1-06` |
| 2D-25 | 차례당 문맥 크기 분해 | 파일럿 발견(`bench/pilot/README.md` 쓰기 표면 절): BVB `write` 판에서 차례 수는 Verus보다 적은데(143 대 156) 비용은 1.64배다. 차례당 캐시 읽기가 약 4,700 대 1,400토큰이다. 시스템 프롬프트(3,471 대 1,131바이트), `tools/list`(약 4KB), 도구 결과(`write`·`verify`의 진단과 증명 결과), 출력(1.6배)으로 나눠 재고, 원인 층(프롬프트·도구 결과 형식·언어)에서 줄인다. 프롬프트·도구 구성은 동결 조건이므로 새 판으로 | 2D-24 | ⏳ |
| 2D-23 | 에이전트에게 보이는 맵의 토큰 효율 | T1-05 발견: L1 행의 토큰 약 3분의 2가 정의 해시 둘(64자리 16진)·위치·빈 필드다. 에이전트용 투영(MCP `map`·`query`)에서 해시를 빼거나 짧게(ADR-0019: 맵은 12자 허용), 빈 필드를 생략하고, 같은 정보를 Rust 꼴 시그니처 텍스트로 보이는 안을 비교한다. MCP 출력은 동결 측정 조건이므로 측정 과제의 새 판(재동결)과 함께 바꾼다(ADR-0021). 확인: 같은 질문에 답하는 L1의 토큰이 정규형 코드 이하 | T1-05 | ⏳ |
| 2D-19 | 명세 약화 감지 | 두 버전 사이에서 공개 정의의 계약이 약해졌는지(`requires` 강화·`ensures` 약화)를 SMT 함의로 판정하는 질의·린트. 약화는 별도 경고로 보고(게이트 우회 감사, H-GAME) | 2D-08 | ⏳ |
| 2D-20 | MCP 도구 위협 모델 | `bmb mcp` 표면의 위협 모델(경로 탈출, 정의 이름·문서 문자열을 통한 지시 주입, 자원 고갈) 문서와 시험. 범위 밖(에이전트 일반 보안)과 경계 명시(ADR-0040 §3) | 1D-23 | ✅ `docs/mcp-threat-model.md`(경계, 위협 T1~T10과 대응·시험, 남는 위험). 발견·수정: 작업 공간에 심은 임시 파일 링크(`src/.u.bmb.tmp`)를 커밋이 따라가 밖의 파일을 덮어씀(재현), `src`·`.bmb-cache`·`target`·단위 파일 링크로 밖을 읽고 씀 → `workspace::refuse_symlink`·`write_new`(증명·증분 캐시 파일 포함), 열린 트랜잭션 상한 64. `tests/mcp.rs`의 `untrusted_workspace_cannot_reach_outside`(수정 전 실패 확인, 동시 작성자 거부 포함) |
| 2D-21 | 남은 테스트 면적 질의 | 컴파일러가 테스트가 여전히 필요한 면적(증명되지 않은 의무, 탈출구, `io` 효과 경계, 외부 입력)을 정의 단위로 산출하는 질의와 크기 지표. 5D-02 러너의 입력, G8의 BMB 쪽 값 | 2D-11 | ✅ `bmb query tests`(`map.rs`의 `tests_needed`): 실행 정의마다 이유(`io` 도달, `unproved`)·직접 `io` 지점 수·정규형 줄 수, 요약에 면적(정의·지점·줄). 증명하는 질의(1D-17 집합에 추가). `tests/map.rs`의 `tests_lists_the_definitions_that_still_need_tests`, `tests/no_prove.rs`. 탈출구·`panic` 이유는 2E·2F가 더한다 |
| 2D-22 | 호출자용 계약 해시에서 `#[uses]` 분리 | 증분 키(ADR-0032)의 피호출자 계약 해시가 `#[uses]`를 담아(ADR-0019 계약 해시) 호출자 질의가 그대로인데도 재증명한다(2A-14a 발견, N=100 이력의 body 편집 35~43%). 호출자 키에는 시그니처·효과·계약만 쓰고, 감사 모드 불일치 0과 BCB 무효화 집합 감소를 확인. 채택된 ADR-0019·0032 변경이라 승인 필요 | 2A-14a | ✅ 승인 2026-10-25, ADR-0043. 호출자 키 = `#[uses]`를 뺀 호출자 해시(`fmt::caller_text`, `incremental.rs`). 감사 불일치 0(`edits_that_shift_spans_or_change_context_keep_outputs_exact`에 피호출자 `#[uses]` 편집 2개), `verify_stats_name_the_reencoded_definitions`가 피호출자만 다시 인코딩함을 확인. BCB 동결 이력 200커밋: 재인코딩 합 458→335(`body-heavy-1k`), 448→306(`body-only-1k`), 모든 커밋에서 부분집합. N=1,000의 커밋당 CPU는 변화 없음 |

### 2E 탈출구와 근거

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2E-01 | ADR 초안: 탈출구 문법과 근거 형식 | `assume`·`external_body`, 필수 근거, 노드 해시 결합(ADR-0006 구체화). 상태 "제안" | 1E-01 | ✅ 제안 ADR-0029(신뢰 수준 proved/assumed/external, 가정 정의는 전체 검사 모드로 컴파일) |
| 2E-02 | 탈출구 구현 | 근거 없으면 오류, 증명기는 가정으로 사용, 코드 생성은 검사 유지 여부를 ADR대로 | 2E-01, 1E-07 | ⏳ 승인 |
| 2E-03 | 미증명 면적 집계 | 맵 L1·`status`에 정의별 탈출구·미증명 의무 수 | 2E-02 | ⏳ 승인 |
| 2E-04 | 구조화 근거의 미확인 표시 | 근거가 묶인 노드 해시가 바뀌면 "미확인" | 2E-02 | ⏳ 승인 |

### 2F 효과 확장

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2F-01 | ADR 초안: `alloc`·`panic`·비결정 효과 | 효과 격자와 호출 그래프 규칙(ADR-0003 확장). 상태 "제안" | 1E-01 | ✅ 제안 ADR-0030(효과 집합 alloc·io·nondet·panic, 테스트 필요 면적) |
| 2F-02 | 효과 검사 구현 | 호출 그래프를 따른 효과 검사, 진단 코퍼스 | 2F-01, 1E-07 | ⏳ 승인 |
| 2F-03 | 효과별 정의 질의 | 효과별 정의 목록과 경로(2D-11 확장) | 2F-02, 2D-11 | ⏳ 승인 |

### 2G 메모리

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2G-01 | ADR 초안: `Vec`과 소유·이동 | 힙 값의 모델, 이동 규칙, 핸들·인덱스 패턴(DESIGN.md §5). 상태 "제안" | 1E-01 | ✅ 제안 ADR-0031(`Copy` 원소 `Vec`, 슬라이스와 같은 모델, 이동 검사 E0413·E0414) |
| 2G-07 | ADR-0031 채택 | 2B·2C 표면에 맞춰 제안 ADR-0031을 갱신·채택(2C-05와 같은 절차): `pop`의 `Option`, 핸들 새 타입, 열린 질문의 제안안, `alloc` 효과 연결 시점, 진단 번호 확인 | 2G-01, 2B-11 | ✅ ADR-0031 채택(2026-10-25): 원소는 배열 없는 `Copy` 타입, E0414로 시작, `with_capacity` 허용·`drop` 거부, 버퍼 메타데이터는 측정 뒤, `resize`류 범위 밖, `alloc`은 ADR-0030(2F) 채택 때 연결(그 전에는 효과 없음), `vec!` 교체 제거(2G-06), 주소 검사 빌드는 ASan 또는 valgrind(둘 다 없으면 실패), E0218·E0219·E0413·E0414·E0514·E0816 빈 번호 확인 |
| 2G-02 | 소유·이동 검사 | 이동 후 사용 거부, 진단 코퍼스 | 2G-07 | ✅ 검사기의 `Vec` 표면(`TypeKind::Vec`, `check::vec_type`·`vec_method`·`vec_call`·`slice_of_vec`, E0218·E0219·E0514, `Option<Vec<T>>`·필드·중첩 거부)과 이동 검사 `src/moves.rs`(E0413·E0414, 해제 지점 `MoveInfo` 기록), E0410을 `Vec` 이동·`as_mut_slice`로 확장. 증명기·코드 생성은 `Vec`이 닿는 정의를 E0814로 거절(`verify::touches_vec`, 테스트 `vec_is_gated_until_its_encoding_and_lowering_exist`). 코퍼스 `tests/check/err/moves`(9진단, 허용형: 다시 대입, 같은 회차 이동·대입, 읽기), `vec`(19진단), 수정 왕복 `vec_coercions_are_fixed_into_a_checked_program` |
| 2G-03 | `Vec` 증명 인코딩 | 길이·원소 모델, `push`·`pop`·인덱스 의무 | 2G-02 | ✅ `Tables::slice_view`(`Vec`은 슬라이스 값, 참조는 슬라이스 참조)로 기존 배열·길이·인덱스·반례 경로를 그대로 씀. `Enc::vec_change`(`push` E0816·`pop`의 `Option`·`truncate`), `vec_new`(`with_capacity` E0816), `&mut Vec` 호출 뒤 길이 새로 정함, 루프의 변경 메서드 수신자 havoc. 코퍼스 `tests/verify/pass/vecs`(8정의: 루프로 채운 결과의 계약, `pop`·`truncate` 계약, 복제·이동, 핸들 패턴 `struct Id`와 늘기만 하는 저장소), `fail/vecs`(반례 6). `bmb stability --seeds 8` 불안정 0(rlimit 퍼짐 1.01). **정정(2G-05):** 루프가 `Vec` 길이를 그대로 두는 결함으로 `first_squares`의 루프는 공허하게 증명되었었다. 고친 뒤 선형 `first_triples`로 바꿈(`dev-docs/health/2G.md`) |
| 2G-04 | 런타임과 코드 생성 | 할당·해제, `alloc` 효과와 연결 | 2G-03 | ✅ `codegen.rs`: 머리 `%bmb.vec`, IR 런타임 `VEC_RUNTIME`(`bmb_vec_alloc`·`grow`·`clone`·`free`, 크기 넘침·할당 실패는 `abort`), `vec_method`·`vec_new`, 해제는 `MoveInfo`(블록 끝, 대입 교체)와 영역 끝 임시값(`region`·`release`). `alloc` 효과 연결은 ADR-0031 채택 갱신 7에 따라 ADR-0030(2F) 때. 실행 `tests/run/vecs`(15값 손 대조: 32까지 성장, 복제 독립, `pop`으로 비움, 교체, 루프 조건 임시값, `bool`·구조체 원소, 핸들): 증명 빌드 검사 0·`proved E0816` 7, 증명·전체 검사 빌드 × -O0~-O3 8조합 출력 같음, valgrind `--leak-check=full` 오류·누수 0. 테스트 `a_proved_push_has_no_check_and_an_unproved_one_keeps_it`, `an_unproved_vec_index_traps_exactly_from_the_length` |
| 2G-05 | 퍼저 확장 | 힙 값을 다루는 건전성 퍼저, 위반 0 | 2G-04 | ✅ `tests/soundness_vec.rs`(CLI 구동): `push`·`pop`·`truncate`·원소 쓰기·읽기·길이·복제 교체·`&mut Vec` 호출·`push` 루프·임시 복제·값 인자 이동. 세 번째 빌드는 -O0 주소 검사(valgrind memcheck 누수 포함, 없으면 ASan, 둘 다 없으면 실패; `scripts/setup.sh`가 valgrind 설치). **발견:** 루프 havoc이 `Vec` 길이를 그대로 두어 `push` 루프 뒤 코드가 도달 불가로 증명됨(건전성 결함, seed 1) → `Enc::loop_len`으로 수정, 회귀 `tests/verify/fail/vecs.bmb`의 `after_push_loop`. 루프·`count` 호출 뒤 반례는 다른 퍼저처럼 재생하지 않고 사전 조건만 확인. 60배치: 360개 중 증명 59개 186회, 반례 308개(재생 164), 컴파일 1,344회 일치, valgrind 672회 오류·누수 0. 변이: 블록 끝 해제 하나 빼기 → seed 1 누수, 임시값 이중 해제 → seed 6. nightly 200배치 |
| 2G-06 | `vec!` 수정 재검토 | E0202의 `vec![a, b]` → `[a, b]` 수정은 `Vec` 도입 뒤 의미를 바꾼다(2R-02 발견). 2G에서 `Vec::from` 류로 바꾸거나 수정 제거 | 2G-03 | ✅ 교체 제거(ADR-0031 채택 갱신 8): E0202 `vec!`는 안내만("고정 배열은 `[a, b]`·`[v; N]`, `Vec`은 `Vec::with_capacity(n)`과 `push`"). `fix_cmd` 습관 시험에서 `vec!` 줄을 빼고(적용 8 → 7), `rust_habits.jsonl` 갱신 |

### 2S 바이트·문자열 (셀프호스팅 격차, 4A-01)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2S-01 | ADR 초안: 바이트·문자열 | `u8` 타입, 바이트 문자열 리터럴, 문자열·바이트 라이브러리(비교·검색·분할·숫자 서식), 증명 모델. 상태 "제안" | 4A-01 | ✅ 제안 ADR-0044: `u8`(1바이트, `u64` 규칙에 범위만 다름), `b'a'`·인자 자리 `b"..."`(`&[u8]`), 내장 `print_bytes`, BMB로 쓰고 증명하는 `lib/bytes.bmb`(`eq`·`starts_with`·`find`·`extend`·`push_u64`·`push_i64`·`parse_u64`), 습관 E0234(신규)·E0101 교체·E0402·E0202 안내. 문자열 타입·`char`는 두지 않음(4A-01 권고) |
| 2S-03 | ADR-0044 채택 | 2S 승인과 권장안 채택 지시에 따라 제안 ADR-0044를 채택(진단 번호 확인) | 2S-01 | ✅ ADR-0044 채택(2026-10-25): E0234 빈 번호 확인, 열린 질문은 제안대로(자동 포함 없음, 맵·입력은 밖) |
| 2S-02 | 바이트·문자열 구현 | 검사·증명 인코딩·코드 생성·코퍼스·퍼저 | 2S-03, 2G-03 | ✅ `u8`(검사·증명·코드 생성), `b'a'`·`b"..."`, `print_bytes`, 습관 E0234·E0101·E0402 교체, `lib/bytes.bmb`(8정의 증명, 공허 0, `stability --seeds 8` 불안정 0), 코퍼스 `tests/check/err/bytes`·`tests/verify/pass/bytes`·`fail/bytes`·`tests/run/bytes`·`tests/lib_run/text`(손 계산 출력, 검사 0, valgrind 0), 수정 왕복 `text_habits_are_fixed_into_a_checked_program`. 퍼저: 부호 없는 산술 퍼저에 `u8` 경유·바이트 합(넘침)·래핑·포화 곱과 배치마다 맨 바이트 연산 함수 하나, 60배치 480개 중 증명 90개(`u8` 44개), 반례 566개, 컴파일 3,978회, 위반 0. `Vec` 퍼저에 `Vec<u8>`·바이트 문자열(이스케이프 포함), 60배치 360개 중 증명 65개, 반례 274개(재생 169), 컴파일 1,258회, valgrind 629회, 위반 0. 변이: `u8` 범위를 `u64`로 넓히기 → seed 10, 바이트 문자열 상수 거꾸로 → 20배치 안 |

### 2V 판단 기준 소급 점검 (ADR-0045)

점검 문서 `dev-docs/reviews/2V-01-judgment-retro.md`. 각 태스크의 근거와 정석 해는 그 문서의 같은 R 번호에 있다.

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2V-01 | 기존 결정의 소급 점검 | ADR-0045 §10 형식으로 기존 제약·우회 13건을 점검하고 귀속 층, 권장, 승인 여부, 후속 태스크를 정함 | 2D-18 | ✅ `dev-docs/reviews/2V-01-judgment-retro.md`. 재현: `first_squares`의 `i*i` 넘침 의무 3,793,469(단조성 사실 하나로 4,142), `primes`의 `i*i` 판에서 `tests/incremental.rs` 감사 불일치 `pr::primes` |
| 2V-02 | 곱셈 단조성 보조 사실 (R1) | 인코더가 곱마다 부호·단조성 사실을 넣음(`tests/lemmas.rs`로 정리 증명). `tests/verify/pass/vecs.bmb`를 `first_squares`(`k * k`)로 되돌려 V5 기준 안에서 증명 | 2V-01 | ✅ `verify.rs::mul_facts`: 두 피연산자가 상수가 아닌 곱마다 0·부호 사실(`MUL_LEMMAS`)과, 경로 조건이 말하는 피연산자 상한 쌍마다 비음수 곱의 단조성 사실(`MUL_BOUND_LEMMA`, 상한은 최상위 연언의 `<`·`<=`에서 최근 것 2개). `tests/lemmas.rs::multiplication_lemmas_hold`가 정리로 증명. `tests/verify/pass/vecs.bmb`를 `first_squares`(`k * k`)로 되돌림: 3,832,706 → 63,212(최대 의무 28,239, 사실 없이는 한도의 76%라 V5 실패). `tests/run/primes.bmb`도 `i * i`부터 표시하는 원래 체로 되돌림: 기저 인스턴스 재시도 없이 164,046, 출력 동일. V4 +2.0%(되돌린 코퍼스), 실패 코퍼스는 rlimit 숫자만 바뀜 |
| 2V-03 | 재시도 증명의 증분 감사 불일치 (R2) | 기저 인스턴스 재시도로 증명된 정의의 증분 기록이 재계산과 같음(`outputs_are_identical_with_and_without_the_cache`에 재시도가 필요한 프로그램을 넣어 확인) | 2V-02 | ✅ 원인: 증명 캐시가 적중한 의무는 건너뛰어(`continue`) 재시도 절의 스크립트가 붙지 않았다. 그래서 재시도로 증명된 정의는 캐시 상태에 따라 스크립트(`--emit-smt` 출력, 증분 감사의 다이제스트)가 달랐다(C2). 수정: 증명 캐시 항목이 재시도 여부를 기록하고(`cache.rs` 형식 v2, v1 파일은 빈 캐시로 읽음), 적중 때 같은 재시도 텍스트를 결정론적으로 다시 만든다(`verify.rs`의 `retry_script`). 회귀: `tests/incremental.rs::retried_proofs_are_reused_exactly`(`compiler/tests/data/retry.bmb`, 스스로 다시 맞는 트리거라 재시도로만 증명. 수정 전 `audit_mismatches: ["rt::wrong_outputs"]`) |
| 2V-04a | 배열 결과와 데이터 배열 값 매개변수 (R3) | 함수가 `[T; N]`(스칼라·데이터 원소)을 돌려주고 데이터 배열을 값으로 받음. 거부 진단 제거, 실행·증명 코퍼스 | 2V-01 | ✅ 검사기: 배열 결과는 `data_array` 경계만(배열을 담은 원소는 2V-04b), 데이터 배열 매개변수는 정규화 뒤 같은 경계. 코드 생성: 배열 결과는 `ret.out`(데이터 결과와 같은 경로, 호출 쪽은 `read`), 값 배열 매개변수는 `size_of` 복사(데이터 원소 포함). `tests/run/arrays`(손 계산 출력 765·31·100·1, 런타임 검사 0, valgrind 깨끗), `verify/fail/structs::wrong_first`(반례 a[0] ≠ a[1]). `check/err`의 두 거부 사례 제거 |
| 2V-04b | 중첩 배열과 배열을 담은 원소 (R3) | `[[T; M]; N]`, 배열 필드를 가진 구조체의 배열(지역·필드·결과·슬라이스)을 증명 인코딩(잎 분해 일반화)과 코드 생성이 받음. 거부 진단 제거, 코퍼스, 데이터 퍼저 확장 | 2V-04a | ✅ 증명기: 원소 템플릿이 고정 길이 내부 배열을 품는다(`select_val`·`store_val`이 `Val::A`·`Val::Arr` 안으로 재귀, `elem_leaves`·`elem_template`·`flatten_elem`, 잎 정렬 `Leaf::Leaves`). E0814 게이트의 "원소에 배열" 거부 삭제. 검사기: 원소 규칙 `array_elem_ok`(스칼라·데이터·고정 배열, `Vec`만 E0402), 장소 `g[i][j]`·`b.rows[i].cells[j]`, 배열 원소의 슬라이스 매개변수. 코드 생성: `is_mem_elem`(데이터·배열 원소는 메모리 경로, `mem_ty`). `tests/run/grid`(손 계산 21·137·40·17·93, 런타임 검사 0, valgrind 깨끗), `verify/fail/slices`의 `inner_index`(반례 `j = 3`)·`set_corner`, `check/err/data_arrays`는 `Vec` 원소만 남김. 데이터 퍼저에 중첩 배열 단계(2×2 격자 쓰기·읽기): 60배치 360함수, 64 증명, 반례 551 재현, 컴파일 실행 3,630, 위반 0. 변이(내부 배열 `store` 무시)는 20배치 안에서 실패 |
| 2V-05 | E0814 잔여 제거 (R4) | 명세의 배열 리터럴·반복, `forall` 안 재귀 호출, 배열 페이로드 열거형의 명세 `==`를 인코딩. 코퍼스에서 E0814 발생 0 | 2V-01 | ✅ 명세 `==`는 Rust의 `==`(`Enc::ext_eq`): 구조체 필드별, 배열은 길이 안의 원소(고정 길이 16 이하는 원소별로 펼쳐 수량자 없음, 그 밖은 범위 수량자), 배열 페이로드 열거형은 같은 변형·같은 페이로드. 명세의 배열 리터럴·반복(`const_elems`, 빈 배열은 `default_val`)과 배열·슬라이스 `==`(검사기도 허용). `forall` 안의 재귀 호출은 묶인 변수에 대해 양화된 측도 의무(E0813, Dafny와 같음). 증명기·코드 생성의 데이터 타입 게이트와 `unsupported` 경로 삭제(남은 두 경로는 검사기가 막는 불변식이라 `unreachable!`), E0814 은퇴(`diag::RETIRED`, `docs/diagnostics.md`의 은퇴 절). 코퍼스: `pass/enums` `cells`(매개변수 페이로드와 다시 만든 배열의 `==`: SMT `=`로는 증명 안 됨), `fail/enums` `swapped`(반례 `Cell::Full([a, b])`, 모델 배열 읽기 `model_select`), `pass/structs` `speclit`·`empties`, `fail/structs` `not_123`, `pass/recursion` `frec`·`allpos`(이전 `fail/unsupported`), `fail/contracts` `loops`(E0813). 기존 `element_differs`는 미결 E0811에서 확정 반례 E0802로 |
| 2V-06 | ADR 초안: 제네릭과 trait (R5) | 단형화 코드 생성, 정의마다 한 번 증명, trait 경계와 계약, 겹침 없는 명시적 impl. 상태 "제안" | 2V-01 | ✅ 초안 ADR-0046(상태 "제안", `dev-docs/adr/0046-*.md`). 채택은 2V-12 |
| 2V-07 | ADR 초안: 소유 타입 일반화 (R6·R7) | `Box<T>`, 재귀 열거형·구조체, `Vec`·`Box`를 담는 데이터 타입, `Vec<Vec<T>>`, 이동과 drop glue, 구조적 `decreases`. 상태 "제안" | 2V-01 | ✅ 초안 ADR-0047(상태 "제안", `dev-docs/adr/0047-*.md`). 채택은 2V-12 |
| 2V-08 | ADR 초안: 비트 연산과 16진 리터럴 (R8) | 연산자, 리터럴, bitvector 혼합 인코딩(ADR-0017 보완). 상태 "제안" | 2V-01 | ✅ 초안 ADR-0048(상태 "제안", `dev-docs/adr/0048-*.md`). 채택은 2V-12 |
| 2V-09 | ADR 초안: 조기 탈출과 `for` (R9) | `return`·`break`·`continue`·`?`(같은 오류 타입), `for i in lo..hi` 정규형과 자동 `decreases`. 2R-04f 결과 반영. 상태 "제안" | 2V-01 | ✅ 초안 ADR-0049(상태 "제안", `dev-docs/adr/0049-*.md`). 채택은 2V-12 |
| 2V-10 | ADR 초안: `String`·`&str`·`char` (R10) | UTF-8 불변식, 바이트 보기, ADR-0044와의 관계. 상태 "제안" | 2V-01 | ✅ 초안 ADR-0050(상태 "제안", `dev-docs/adr/0050-*.md`). 채택은 2V-12 |
| 2V-11 | ADR 초안: 메서드 구문 정본화 (R11) | 고유 impl 메서드 구문, 자동 참조 없음, `T::f(&x)`의 기계 수정, 맵 L2 기록. 2R-06과 함께. 상태 "제안" | 2V-01 | ✅ 초안 ADR-0051(상태 "제안", `dev-docs/adr/0051-*.md`). 채택은 2V-12 |
| 2V-12 | 언어 ADR 승인 요청과 로드맵 반영 | 2V-06~2V-11 초안을 한 번에 승인 요청. 채택된 것마다 2단계 페이즈(ADR → 검사 → 인코딩 → 코드 생성 → 코퍼스 → 퍼저)를 ROADMAP·백로그에 올림 | 2V-06, 2V-07, 2V-08, 2V-09, 2V-10, 2V-11 | ✅ 사람 승인(2026-10-26, "1/2 채택, 승인"): ADR-0046~0051 채택(채택 시 확인에 진단 번호와 열린 질문 결정), ADR-0026·0031·0044 상태 줄에 일부 대체, ADR-0017에 보완 표시. 구현 페이즈 2T·2O·2K·2U·2N·2M을 ROADMAP·백로그·통과 기준 추적에 올림 |
| 2V-13 | 2차 점검: 남은 "milestone 1" 제외 항목 (R14) | `const`·`static`, `type` 별칭, `use`, 클로저, 튜플·튜플 구조체·구조체형 변형을 원장(DESIGN §4.1)과 대조해 §10 형식 권장안을 `dev-docs/reviews/2V-01-judgment-retro.md`에 더하고, 언어 층이면 ADR 초안 태스크를 도출 | 2V-01 | ✅ `dev-docs/reviews/2V-01-judgment-retro.md` §R14: `const`은 넣음(2V-14), `use`는 도구 수정(2V-15), 클로저·함수 값은 ADR 초안(2V-16), 불변 `static`·`static mut`·`type` 별칭·튜플류는 원장 근거가 있어 유지하되 원장 표에 명시(2V-17) |
| 2V-14 | ADR 초안: `const` 항목 (R14) | `const NAME: T = e;`(상수 식), 증명에서 값·코드에서 상수, 단위 이름공간, E0207 안내 대체. 상태 "제안" | 2V-13 | ✅ 초안 ADR-0052(상태 "제안", `dev-docs/adr/0052-*.md`). 채택은 2V-18 |
| 2V-15 | `use`를 전체 경로로 고치는 기계 수정 (R14) | 단일 이름 `use a::b::f;`를 받아 E0207에 교체(쓰임을 전체 경로로, `use` 줄 삭제)를 붙임. `fix_cmd` 왕복 시험(고친 프로그램이 검사 통과) | 2V-13 | ✅ 새 코드 E0214: 파서가 `use` 경로를 `Mod.uses`로 받고, 검사기가 `use` 줄에 삭제 교체, 가져온 이름의 호출·타입 쓰임에 전체 경로 교체를 붙인다. glob·묶음은 E0214 안내만(복구는 `;`까지 건너뜀). 시험 `fix_cmd::use_items_are_rewritten_to_full_paths`(고친 결과가 검사·증명 통과), 코퍼스 `tests/check/err/items` E0214 세 줄 |
| 2V-16 | ADR 초안: 클로저와 함수 값 (R14) | 이동 캡처만(2차 참조와 양립), `Fn` 계열 경계(ADR-0046), 함수 값의 계약(Verus 명세 클로저), 코드 생성(환경 구조체 + 함수 포인터). 상태 "제안" | 2V-13, 2V-06 | ✅ 초안 ADR-0053(상태 "제안", `dev-docs/adr/0053-*.md`). 채택은 2V-18 |
| 2V-17 | DESIGN 원장에 유지되는 엄격화 명시 (R14) | §4.1 "이름" 행에 불변 `static`·`type` 별칭, "효과" 행에 전역 가변 상태 없음, "표현" 행에 튜플류. DESIGN 개정이라 승인 | 2V-13 | ✅ 사람 승인(2026-10-26): DESIGN.md v1.2 §4.1 원장 개정("이름"에 `type` 별칭·불변 `static`과 `use` 처리, "효과"에 전역 가변 상태 없음, "표현"에 범위 `for`·`match`·이름 붙은 필드 구조체, 새 행 "이동 상태"(E0414), "호출 해석"에 수신자 한 단계 조정), §5에 소유 타입 |
| 2V-18 | ADR-0052·0053 채택 (승인) | 초안 검토 후 채택하면 진단 번호를 확정하고 구현 페이즈를 ROADMAP에 넣는다(`const`는 2T 앞 또는 병행, 클로저는 2T 뒤) | 2V-14, 2V-16 | ⏳ 승인 대기 |

### 2T 제네릭과 trait (ADR-0046, 승인 2026-10-26)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2T-01 | 제네릭 함수·데이터 타입 (trait 없음, `Copy` 경계) | 타입 매개변수 `<T: Copy>`(함수·구조체·열거형·고유 `impl`), 타입 인자 `f::<T>(..)`, 정규형·맵 L1·L2. 검사(E0237 기계 수정, E0428, E0429, E0430, `Copy` 없는 매개변수 E0402), 증명(매개변수는 정렬, 제네릭 `spec fn`은 인스턴스, 호출자는 대입한 계약), 코드 생성(단형화). 코퍼스 | 2V-12 | ✅ 파서(`parse_generics`, 경로의 `::<..>`는 `TArgs`), 검사(`TypeKind::Param`, `subst`·`unify`, 호출의 타입 인자 기록 `Tables::targs`, 생성·변형 값은 기대 타입이나 값에서), 증명(`verify::prepare`가 제네릭 `spec fn` 인스턴스를 모음, `Enc::tsub`, `sort_decls`), 코드 생성(인스턴스 작업 목록, `Gen::tsub`), 정규형·맵(L2 `type_args`). 덤: `impl` 함수의 재귀도 E0507(이전에는 빠졌음). 코퍼스 `tests/check/err/generics`(19줄, 모두 안내 있음), `tests/verify/pass/generics`(루프·`forall`·재귀·제네릭 호출자, 모두 증명), `tests/verify/fail/generics`(반례 `x = T#1, y = T#0` 등), `tests/run/generics`(21, 137, 40, 17, 93, 13, 2, 9, valgrind 깨끗), `fix_cmd::omitted_type_arguments_are_written_out`. 건강 P1 19→20, P3 0.7088→0.728 |
| 2T-02 | 내장 경계 `Eq`·`Ord` | 경계 `T: Eq`·`T: Ord`(코드의 `==`·`<` 등), 정수·`bool`의 내장 impl, 증명은 정렬 위의 비해석 전순서, 인스턴스는 그 타입의 순서. 검사(E0428). 코퍼스와 제네릭 정렬 실행 | 2T-01 | ✅ 검사 `has_builtin`·`targ_ok`, 증명 `sort_decls`(`'T.le`과 공리 넷)·`ord_term`(매개변수·`bool`·정수). `tests/run/sort_generic`: 정렬됨을 증명한 `insertion_sort<T: Ord + Copy>`(rlimit 128084)을 `i64`·`bool`·`u8`로 실행(-7 -3 0 5 9 9, 0, 200, valgrind 깨끗). 코퍼스 `tests/check/err/generics`(`Ord` 없는 `<`, `Ord` 아닌 구조체 인자), `tests/verify/fail/generics`(`max_first`: 반례 `a = T#1, b = T#0`) |
| 2T-03 | `where` 절과 정규형 | `where T: B`를 받아 정규형(매개변수 옆 경계)으로 고치는 기계 수정, `impl` 매개변수를 함수 해시에 포함 | 2T-02 | ✅ 파서 `parse_where`(함수·`impl`·구조체·열거형): 경계를 이름 붙은 타입 매개변수에 합치고 E0240(새 코드)이 `<` 부터 절 끝까지를 매개변수 옆 경계로 고쳐 쓴다(매개변수가 아닌 타입은 안내만). `fmt::qualified_text`가 제네릭 `impl`의 매개변수·경계와 trait impl·trait 메서드의 머리(`impl<T: Copy + Shape> <unit::Box<T> as unit::Shape>`)를 해시에 담음(경계 변경 → 함수 해시 변경 확인). 시험 `fix_cmd::where_clauses_move_to_the_parameters`(고친 텍스트가 증명됨), 코퍼스 `tests/check/err/generics`(E0240 둘) |
| 2T-04 | 인스턴스별 증분 키 | 인스턴스의 IR과 제네릭 호출을 담은 함수의 증분 IR 키(경로와 타입 인자의 정규 텍스트, 피호출 인스턴스 서명), 캐시 적중 시 부르는 인스턴스 목록 보존 | 2T-01 | ✅ `incremental::IrDeps`(부르는 인스턴스, 쓰는 인스턴스 타입)를 조각과 함께 저장(`deps_text`·`parse_deps`, 타입은 `check::parse_ty`), 인스턴스 키는 기반 정의 IR 키 + 인스턴스 텍스트 + 타입 인자의 데이터 타입 선언 폐포(`Session::ir_key`), 인스턴스·제네릭 호출 함수의 캐시 우회 제거. 시험 `incremental::instances_are_reused_and_follow_their_type_arguments`(warm은 인스턴스를 낮추지 않음, `Pt` 필드 타입 변경은 `id<g::Pt>`·`wrap<g::Pt>`만), `outputs_are_identical_with_and_without_the_cache`(감사 포함, 의존 비교), 단위 `check::tests::parse_ty_reads_what_ty_str_prints` |
| 2T-07 | 사용자 trait | `trait` 선언(메서드 시그니처와 계약, 기본 본문), `impl Trait for T`, 경계 `T: Trait`, 호출 `Trait::m(&x)`(`Self`는 수신자에서). 검사(겹치는 impl·모호한 메서드 E0311, 단위 규칙 E0312, blanket impl E0235·`dyn` E0236 거부, 경계 충족 E0428, impl 메서드는 trait의 `requires`를 받고 `ensures`만 더함 E0817), 증명(경계 trait 메서드는 trait 계약, 기본 본문은 `Self` 정렬로 한 번), 코드 생성(impl 메서드 또는 기본 본문 인스턴스로 정적 호출). 정규형·맵. 코퍼스 | 2T-02 | ✅ 하위 태스크 2T-07a·b·c |
| 2T-07a | trait 선언·impl·경계·정적 호출 (계약 없음) | `trait` 선언(메서드 시그니처, `Self`), `impl Trait for T`(제네릭 impl 포함), 경계 `T: Trait`, 호출 `Trait::m(..)`(`Self`는 인자·기대 타입에서). 검사(겹치는 impl E0311, 단위 규칙 E0312, blanket E0235, `dyn` E0236, 경계 충족 E0428, 빠지거나 남는 메서드 E0315, 서명 불일치 E0316, trait에 없는 `requires` E0817), 증명(impl 메서드는 보통 정의, 구체 `Self`는 impl 계약, 매개변수 `Self`는 trait 서명), 코드 생성(impl 메서드 인스턴스로 정적 호출), 정규형·맵. trait 계약·`spec fn`·기본 본문은 E0207(2T-07b·c). 코퍼스 | 2T-02 | ✅ 파서(`parse_trait`, `ImplItem`, 타입 `Self`, `dyn` E0236), 검사(`collect_trait_names`·`collect_impl_heads`·`collect_trait_sigs`·`conform`, 경계 `has_trait`, 호출의 `Self` 추론, 인스턴스 그래프의 경계 재귀 `trait_cycles`), 공용 `check::dispatch`·`defs_of`(정의 수집 넷을 하나로)·`trait_decls`, 증명(`verify::dispatched`, trait 선언은 증명하지 않는 정의), 코드 생성(impl 함수·인스턴스로 정적 호출), 증분 키(trait 메서드 이름은 모든 impl 함수를 읽음), 정규형(`trait_def`·`trait_impl`, 멱등), 맵(L0·L1 trait, L2 해석된 대상). 새 코드 E0315·E0316(그리고 ADR의 E0235·E0236·E0311·E0312·E0817 등록). 코퍼스 `tests/check/err/traits`(23줄, 모두 안내 있음), `tests/verify/pass/traits`(구체 `Self`의 impl 계약, 구조적 위임, 메서드 자신의 타입 매개변수), `tests/verify/fail/traits`(경계 결과의 넘침 `a = T#0, b = T#0`, trait 밖 성질, impl 계약 밖 단언), `tests/run/traits`(9 10 3 19 7 20 49 5). 증분 캐시 파일의 경로 필드는 공백을 `\s`로(impl 함수 경로). 건강 P1 21→22, P3 0.7295→0.75, V4 +43548·S4 +1210(기능 비용, 기록). 발견: `2T-08` |
| 2T-07b | trait 계약과 정제 의무 | trait 메서드의 `requires`·`ensures`와 trait의 `spec fn`(정렬 위 비해석 함수, impl에서는 impl의 정의), 경계 호출은 trait 계약, impl 메서드는 trait 계약을 정제하는 의무 E0817(전제는 약하게, 결과는 강하게), 반례 코퍼스. 경계 `Self` 호출을 거치는 재귀(2T-07a의 E0207, `check::trait_cycles`)를 trait 메서드의 `decreases`로 풀지 정한다 | 2T-07a | ✅ 검사: trait 메서드의 계약·`spec fn` 메서드를 받고 선언은 `check_fn`으로(본문 없음), trait 메서드의 `decreases`는 E0207(결정: 경계 호출은 측도를 갖지 않으므로 경계 재귀 E0207 유지), 구문적 E0817 제거. `#[uses(Tr::m)]`(`check::uses_target`, 검사·증명·정규형·맵 공용). 증명: `Enc::refines`(정제 의무 E0817 둘: 전제 함의, 결과 함의), `spec_target`·`prepare`의 디스패치(비제네릭 impl은 인스턴스 없음), `revealed`(trait spec 메서드 → impl 정의), `refinement_of`. 증분 키: impl 함수는 trait 메서드 선언을 읽고, `#[uses(Tr::m)]`는 impl 정의 본문을 읽음. `check::defs_of`는 인스턴스 서명을 거름. 코퍼스 `tests/verify/pass/trait_contracts`(spec 메서드, `&mut` 수신자 `old(self)`, 경계·구체 호출), `tests/verify/fail/trait_contracts`(더 강한 전제·더 약한 결과 E0817, 경계 호출의 전제 E0801, 모두 반례), `tests/run/trait_contracts`(경계로 증명한 최댓값을 `Card`·`Hand`로: 11 10 6), `tests/check/err/traits`(trait 메서드 `decreases`) |
| 2T-07c | 기본 메서드 본문 | trait의 기본 본문(`Self` 정렬로 한 번 증명, impl이 정의하지 않으면 그 타입의 인스턴스로 코드 생성), 정규형·맵 L1·L2, 실행 코퍼스 | 2T-07b | ✅ `FnSig::default`, 기본 본문은 `check::defs_of`의 정의(`Self` 정렬로 한 번 증명, `check_fn`이 본문 검사), `check::dispatch`는 정의하지 않은 impl에서 trait 메서드 경로를 돌려줌(증명은 그 계약, 코드 생성은 인스턴스), E0315는 기본 있는 메서드를 빼지 않음, spec 기본 본문은 구체 `Self` 인스턴스에서만 공개(`Prog::revealed`). 맵 단위 목록에 기본 본문. 코퍼스 `tests/run/trait_defaults`(기본 42, 재정의 10, spec 기본·재정의 7·9 증명), `tests/verify/fail/trait_defaults`(기본 본문의 자기 계약 위반 `self = Self#0`, 경계 `T`에서 기본 spec 값을 가정하지 못함) |
| 2T-08 | 스칼라 참조 `&T`의 코드 생성과 표면 | 2T-07a 발견: `T`가 정수·`bool`인 제네릭 `&T` 인스턴스(`get::<i64>(&a)` in `fn get<T: Copy>(x: &T) -> T { *x }`)에서 코드 생성이 패닉한다(`aggregate used as scalar`, 2T-01부터, 유효한 입력에 컴파일러 충돌). 또 `&Self` 수신자 trait을 정수 타입에 impl하려면 `self: &i64`가 필요한데 비제네릭 코드의 `&i64` 매개변수는 E0402다. 코드 생성(스칼라 참조 매개변수·인자·`*x`·`*x = v`), 증명 인코딩 확인, `&i64` 매개변수 허용 여부(ADR-0015 범위, 결정 기록), 실행 코퍼스 | 2T-01 | ✅ ADR-0054(자율 채택): 정수·`bool` 참조 매개변수 허용(`check_param_type`). 코드 생성: 스칼라 참조 매개변수는 호출자 슬롯 포인터의 `Slot::Scalar`, 인자는 `Gen::scalar_ref`(지역·참조 매개변수는 슬롯 그대로, 필드·원소는 임시와 `&mut` 되쓰기 `Back`), `*x = v` 대입. 증명은 그대로이고 건전성 확인(`tests/verify/fail/scalar_refs`: 지역·필드·원소를 `&mut`로 넘긴 뒤의 단언 셋 실패, `good` 증명). `tests/run/scalar_refs`(5 6 12 1 8 0 255 2 255 8 0 16 2 0, 정수 타입·`bool`의 trait impl 포함, valgrind 깨끗). `tests/check/err/places`의 `&i64` 예는 `&()`로 |
| 2T-05 | 퍼저 | 데이터 퍼저에 제네릭 함수·컨테이너와 경계. 안정성(V7) 측정 | 2T-04 | ✅ 퍼저 `tests/soundness_traits.rs`(구체 `Self` 호출 `h`, 경계 호출 제네릭 `f<T: Val>`, 무작위 계약 impl `R`의 정제, 반례 재현, 두 빌드 대조; 24배치 위반 0). 게이트 검출력 35/35(`dev-docs/measurements/gate-power`, trait 변이체 넷: `v-refine-post-dropped`·`v-refine-pre-dropped`·`v-default-spec-leaks`·`c-trait-call-undispatched`; trait 퍼저가 첫째와 넷째를 잡음). 발견: 2T-02 뒤 적용되지 않던 변이체 여섯을 다시 겨누고 게이트 단계 `power` 추가, `v-default-spec-leaks`의 동치에서 기본 spec 공개의 과잉 보수 발견·수정(61105a21). V7 = 0.0(의무 1,431개) |
| 2T-06 | 실용 프로그램과 종료 감사 | 제네릭을 쓰는 실행 프로그램 2개 이상(예: `sort<T: Ord>`, 제네릭 스택), `dev-docs/health/2T.md`, 통과 기준 하나씩 확인 | 2T-05 | ✅ `dev-docs/health/2T.md`: 통과 기준 1~3 충족. 실행 프로그램 7개(`generics`·`sort_generic`·`generic_stack`·`traits`·`trait_contracts`·`trait_defaults`·`scalar_refs`), 마찰 표 2T 행 5개 처리 |

### 2O 소유 타입 (ADR-0047, 승인 2026-10-26)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2O-01 | `Box<T>` (재귀 없음), 전 계층 | 착수 시 재분해(2026-10-27): 계층별 태스크는 중간 커밋에서 검사기만 받는 구성이 생기므로 기능 단위로 나눔. `Box<T>` 타입·`Box::new`·`*b` 읽기·쓰기·`b.f` 투영, 구조적 이동 분류(`Box`와 그것을 담는 데이터 타입은 이동 타입, 구조적 `Copy`), `moves.rs`를 이동 타입 전체로, 필드 대입의 해제, 증명은 `Box`를 값 그대로(테이블 보기), 코드 생성은 `malloc`한 주소와 타입별 drop glue(`@"bmb_drop.<type>"`), `Option<Box<T>>`. 재귀는 2O-02까지 E0421 유지. 습관 E0238(`Rc`·`RefCell`·`Cell`·`Arc`). 코퍼스, valgrind | 2V-12 | ✅ 검사: `TypeKind::Box`, `Box::new`(기대 타입 또는 인자 타입), `*b`·`b.f`(참조와 모든 `Box`를 거친 투영)·`*b = v`·`(*b)[i] = v`, 구조적 이동 분류 `check::is_move`(`Copy` 경계는 이동 타입을 거부, 필드는 `check_owned_fields`), 이동 원소의 배열·`Vec`과 `Box<Vec<T>>`·`Box<()>`는 E0402, 재귀는 E0421 유지, `Box` 예약(E0309), E0238(`Rc`·`RefCell`·`Cell`·`Arc`), 코드의 `Box ==` E0425. `moves.rs`는 모든 이동 타입으로: 블록·갈래 값은 언제나 이동, 이동 페이로드를 묶는 `match`는 피검사 값 이동(`consumed`)과 갈래 끝 바인딩(`arm_drops`), 소유 필드·`*b` 대입의 해제(`replaces`, 대입 대상 타입을 기록해 `&mut Vec` 대입의 누락도 고침), 필드·`*b`에서 꺼내기 E0413. 증명: `slice_view`가 `Box<T>`를 어디서나 `T`로(필드·페이로드·타입 인자·trait impl 머리), `Box::new(v)`는 `v`. 코드 생성: `malloc`한 주소의 칸(`@bmb_box_alloc`), 타입별 drop glue `@"bmb_drop.<type>"`(`@bmb_vec_free` 대체), 읽기만 한 임시값(필드·`*`·`&` 인자·이동하지 않는 `match`)의 영역 끝 해제. 코퍼스: `tests/check/err/boxes`, `tests/verify/pass/boxes`(7 증명), `tests/verify/fail/boxes`(5 실패, 반례), `tests/run/boxes`·`box_places`, 시험 `run_corpus_frees_every_allocation_once`(힙을 쓰는 실행 코퍼스를 `-O0`·valgrind로 누수·이중 해제 없음) |
| 2O-02 | `Box`를 거친 재귀 타입 | E0421을 값 직접 포함으로 좁힘(유한 값이 없는 타입·바뀌는 인자로 자기를 담는 제네릭 타입·상호 재귀 제네릭은 E0421), 재귀 열거형(상호 재귀 포함, 구조체는 열거형·`Option`을 거쳐)은 SMT 데이터타입(상호 재귀는 함께 선언), 재귀 drop glue, 구조적 `decreases x`(열거형 값: 재귀 데이터타입마다 크기 함수 `size`와 생성자 선택자마다 하위 값이 작다는 공리). 착수 시 재분해(2026-10-27): `Vec`을 거친 재귀(`struct Node { kids: Vec<Node> }`)는 `Vec` 필드와 이동 원소가 필요해 2O-03으로. 식 트리 평가 코퍼스 | 2O-01 | ✅ 검사(`check_type_cycles`): 값 직접 포함 E0421(수정 `Box`), 유한 값 없음 E0421(최소 고정점), 제네릭은 자기 매개변수 그대로만(다른 인자·상호 재귀 E0421), 재귀 타입의 배열 E0402, `decreases`는 정수 또는 재귀 열거형(`measure_type`). ADR-0055(자율 채택): 참조 `match`(`match e`, `match &n.next`)의 바인딩은 `&P`, 참조 뒤·필드의 소유 페이로드 값 바인딩은 E0413(기계 수정), `&mut` `match`는 E0207(2O-04). 증명: 재귀·상호 재귀 열거형을 함께 선언(`datatypes`, Tarjan), 구조적 감소는 결정 가능한 조건(`Enc::decrease_goal`: 호출 측도가 호출자 측도에서 생성자 아래의 선택자 사슬, 처음 시도한 크기 함수 공리는 실패 사례에서 해법기가 결정하지 못해 버림), `default_val`·`has_array`가 재귀에서 멈춤. 코드 생성은 재귀 drop glue. 코퍼스: `tests/check/err/recursive`, `tests/verify/pass/recursive`(식·제네릭 리스트·상호 재귀 숲·구조체 리스트, 11 증명), `tests/verify/fail/recursive`(같은 값·새로 만든 값으로 재귀, spec 재귀, 넘침, 사후 조건: 반례 4), `tests/run/expr_tree`(식 트리 평가·크기·접기, 연결 리스트)·`tests/run/forest`(상호 재귀, `List<T>`), 둘 다 valgrind 깨끗 |
| 2O-03 | 소유 필드와 부분 이동 | 착수 시 재분해(2026-10-27): 필드와 원소는 이동 상태·증명 인코딩·코드 생성의 변경이 달라 둘로 나눔(원소는 2O-03b). `Vec`을 담는 구조체·열거형, `Option<Vec<T>>`·`Box<Vec<T>>`, 필드 자리의 `Vec` 메서드·인덱스(`s.buf.push(x)`, `s.buf[i] = v`), 지역 구조체에서 필드를 꺼내는 부분 이동(필드 경로 이동 상태, 꺼낸 뒤 전체 사용 E0433, 합류점 E0414), 남은 필드의 해제, 부분 이동 뒤 전체 대입 | 2O-02 | ✅ 검사: 필드·페이로드·`Option`·`Box`의 `Vec` 허용(E0402 해제), 필드 자리 메서드의 쓰기 가능성(`Checker::writable`). `moves.rs`: 필드 경로 이동 상태(`status_in`, `path_of`), 꺼낸 필드 재사용 E0413, 꺼낸 뒤 전체 사용 E0433(신규), 합류점·루프의 경로별 E0414, 이동된 구조체의 필드 대입 E0413, 남은 소유 필드만 해제(`drop_paths`), 부분 이동된 자리의 통째 대입(`partial_replaces`), 참조 뒤·`Box` 안 필드에서 꺼내기 E0413(수정 안내 2O-04). 증명은 바뀌지 않음(필드의 `Vec`은 슬라이스 잎, 필드 자리 `push`·인덱스·`&mut` 호출 뒤 길이). 코드 생성: `drop_path`, `Vec` 필드 원소 자리(`place`), 임시값 판정을 `fresh`로(필드 받는이의 버퍼를 풀던 위험 제거), `Vec` 머리 선언을 타입 정의 앞으로, 열거형 배치에서 `Vec` 24바이트, `Vec` 사용 판정을 필드까지(`holds_vec`). 코퍼스: `tests/check/err/owned_fields`, `tests/verify/pass/owned_fields`(3), `tests/verify/fail/owned_fields`(반례 4), `tests/run/owned_fields`(valgrind 깨끗) |
| 2O-03b | 중첩 `Vec`과 이동 원소 | 착수 시 재분해(2026-10-27): `Vec`을 거친 재귀(재귀 구조체의 데이터타입 인코딩이 필요)는 2O-03c로, 이동 원소의 고정 배열도 2O-03c로. `Vec<Vec<T>>`·`Vec<S>`(이동 원소)·`Vec<Box<T>>`, 원소 drop glue, 원소 자리의 메서드(`v[i].push(x)`), 증명의 가변 길이 안쪽 잎 | 2O-03 | ✅ 검사: `vec_elem_ok`(`Vec`·`Box` 원소), `&[Vec<T>]` 매개변수, 원소에서 꺼내기 E0413, 이동 원소 `clone` E0404, `Vec`을 거친 재귀 E0421. `moves.rs`: 원소 대입의 해제(`replaces`). 증명: 안쪽 `Vec`의 길이 배열 잎(`elem_leaves`·`elem_template`·`flatten_elem_of`·`select_len`/`store_len`), 안쪽 길이 범위 사실(`inner_len_facts`), 빈 `Vec` 기본값, `Vec::new`의 원소 판정을 `agg_elem`으로. 코드 생성: 이동 원소는 주소 원소(`is_mem_elem`), `Vec` glue 원소 반복, `truncate`의 잘린 원소 해제, 원소 대입의 옛 원소 해제. 코퍼스: `tests/check/err/nested_vecs`, `tests/verify/pass/nested_vecs`(4), `tests/verify/fail/nested_vecs`(반례 5), `tests/run/nested_vecs`(valgrind 깨끗) |
| 2O-03c | `Vec`을 거친 재귀와 이동 원소 배열 | 착수 시 재분해(2026-10-27): 구조체만으로 된 순환(`struct Node { kids: Vec<Node> }`)은 재귀 구조체의 단일 생성자 데이터타입 인코딩(ADR-0047 §3 열린 질문 1)이 필요해 2O-03d로. 열거형이 낀 `Vec`을 거친 재귀(`enum Tree { Many(Vec<Tree>) }`), `Vec` 원소를 거친 구조적 `decreases`, 이동 원소의 고정 배열(배열 glue, 원소 이동) | 2O-03b | ✅ 검사: 구조체만의 순환 E0421(`struct_parts`), 열거형이 낀 순환 허용, 이동 원소 배열 허용(`[v; N]`은 E0402). 증명: 중첩 데이터타입, 감소 사슬의 원소 단계와 범위 조건, 데이터타입 선언을 서곡 앞으로(Z3 4.15.4가 서곡 뒤의 중첩 데이터타입에서 만족 질의를 끝내지 않음, 측정 `dev-docs/measurements/z3-nested-datatypes`), 반례의 `let` 풀기. 코드 생성: 배열 glue, 배열 지역 해제, 임시 배열 해제. 코퍼스: `tests/check/err/vec_recursion`, `tests/verify/pass/vec_recursion`(4), `tests/verify/fail/vec_recursion`(반례 3), `tests/run/vec_tree`·`owned_arrays`(valgrind 깨끗) |
| 2O-03d | 구조체만으로 된 재귀 | `struct Node { v: i64, kids: Vec<Node> }`: 재귀 구조체를 단일 생성자 데이터타입으로(필드 읽기는 선택자, 필드 쓰기는 생성자 재구성), ADR-0047 §3 열린 질문 1(모든 데이터 타입을 데이터타입으로 통일할지)의 V4·V7 측정과 함께. 착수 시 재결정(2026-10-27): 데이터타입은 Z3 4.15.4에서 만족 질의가 끝나지 않아(측정) `Int` 손잡이로(ADR-0056 자율 채택) | 2O-03c | ✅ 검사: 구조체만의 순환 허용(`check::struct_cycle`, 2O-03c의 E0421 해제), `decreases`가 재귀 구조체를 받음, 명세에서 재귀 구조체 값 만들기 E0432(신규), `Vec`을 담는 재귀 타입의 명세 `==` E0431(신규: 2O-03c의 열거형 `==`가 인코더를 끝나지 않게 하던 결함 수정). 증명(ADR-0056): `Int` 손잡이 정렬(`define-sort`, 데이터타입보다 먼저)과 선택자 함수, 값 만들기·필드 쓰기는 새 상수와 선택자 등식(`Enc::mk_struct`, `Step::DtField`), 필드 사실은 의무의 경로 조건과 목표의 선택자 적용마다 지상으로(`Prog::struct_facts`; 처음에 값 정의까지 훑어 `Node { v: 3 * x, .. }`의 넘침이 증명되던 건전성 결함을 확장한 퍼저가 찾아 고침, 회귀 `built`), 빈 `Vec`의 원소는 `'arb`, 감소 사슬에서 구조체 생성자는 판별자 없음, 반례는 한 단계 필드로. 측정 `dev-docs/measurements/z3-nested-datatypes`(데이터타입 20초 무응답, 정해지지 않은 정렬 `unknown` 5M, `Int` 93K). 열린 질문 1은 열어 둠. 코드 생성 변경 없음. 코퍼스: `tests/check/err/rec_structs`, `tests/verify/pass/rec_structs`(상호 재귀·제네릭·두 단계 쓰기·재귀 spec fn, 10 정의), `tests/verify/fail/rec_structs`(반례 7), `tests/run/rose`(valgrind 깨끗). 퍼저 `soundness_owned`에 장미 나무(자식·손자 추가, 두 단계 아래 쓰기, 읽기, 합)·`Vec` 트리(감싸기, 첫 잎, 합) 연산: 80배치(씨앗 100~179, release) 480 함수, 118 증명(384 실행), 반례 539(265 재현), 컴파일 실행 3378, valgrind 1689, 불일치 0. 게이트 변이 `v-struct-field-write-dropped`·`v-struct-facts-from-definitions` |
| 2O-04 | `mem::replace`·`swap`, 습관, `Copy` 없는 타입 매개변수 | 착수 시 재분해(2026-10-27): `&mut` 참조 `match`는 증명 쓰기 되돌림과 코드 생성의 포인터 바인딩이 필요해 2O-04b로. `std::mem::replace(&mut x.f, v)`·`std::mem::swap`, `&mut` 뒤 필드 이동 E0239(기계 수정), `Copy` 경계 없는 타입 매개변수(E0402 해제, ADR-0046 §2: 이동하는 `T`, 인스턴스의 drop) | 2O-03b | ✅ 검사: `mem_op`·`mem_call`(`&mut 자리`나 `&mut T` 매개변수, 명세 E0503, 같은 변수 E0410), `mem::replace` E0214(기계 수정), `mem::take` E0219 유지, E0239(신규, `Vec`·`Option`은 기계 수정), 공유 참조 뒤는 E0413, `Copy` 없는 매개변수(`is_move_in`·`moving_of`, `Copy` 경계 검사, 그 배열 E0402). 증명: `Enc::mem_call`(자리 쓰기). 코드 생성: `Gen::mem_call`·`any_place`(스칼라 슬롯·배열 지역·데이터 자리). 코퍼스: `tests/check/err/mem_moves`, `tests/verify/pass/mem_moves`(5), `tests/verify/fail/mem_moves`(반례 3), `tests/run/mem_moves`·`generic_moves`(valgrind 깨끗) |
| 2O-04b | `&mut` 참조 `match` | ADR-0055의 `&mut` 기본 바인딩(`&mut P`): 증명은 갈래 끝에 바인딩의 새 값으로 피검사 자리를 다시 쓰고, 코드 생성은 페이로드 자리를 가리키는 바인딩(스칼라는 메모리 표현의 임시와 되쓰기), 제자리 트리 수정 코퍼스 | 2O-04 | ✅ 검사: `&mut` 참조 `match`의 바인딩 `&mut P`, 갈래의 피검사 변수 사용 E0410, `place_of`가 `&mut` 인자 자리의 하위 식 타입을 기록(`&mut **a`에서 상자를 풀지 않던 코드 생성 결함 수정). 증명: 갈래 끝 되쓰기(`match_arms`의 `back`). 코드 생성: 페이로드 주소 바인딩, 스칼라는 레지스터 슬롯과 갈래 끝 되쓰기. ADR-0055 결정 갱신. 코퍼스: `tests/check/err/mut_match`, `tests/verify/pass/mut_match`(제자리 재귀 `set_leaf` 포함 3), `tests/verify/fail/mut_match`(반례 2), `tests/run/mut_match`(트리 숫자 두 배, valgrind 깨끗) |
| 2O-05 | 퍼저 | 소유 트리(삽입·제거·교체·`mem::replace`) 퍼저, valgrind로 누수·이중 해제 | 2O-04 | ✅ `compiler/tests/soundness_owned.rs`: 무작위 함수가 소유 연결 리스트(`Option<Box<Node>>`, 앞에 넣기·`mem::replace`로 머리 빼기·`&mut`/공유 참조 `match`로 머리 쓰기·읽기·`mem::swap`), `Box` 교체, `Vec` 필드의 부분 이동과 재대입, 구조적 재귀 합을 돈다. 해석기(`VecDeque`)와 증명된 함수의 실행·반례 재현·두 빌드 출력·valgrind(누수·이중 해제·해제 뒤 접근)를 대조. 40배치(씨앗 100~139, release): 240 함수, 57 증명(180 실행), 반례 268(157 재현), 컴파일 실행 1542, valgrind 771, 불일치 0. 변이: `c-box-free-dropped`(valgrind)와 `v-replace-returns-new`(반례 재현)를 이 퍼저가 잡음(수동 적용 확인, `scripts/gate_power.py`에 소유 타입 변이 4개 추가, 측정은 2O-06) |
| 2O-06 | 실용 프로그램과 종료 감사 | 재귀 자료를 쓰는 실행 프로그램 2개 이상(예: 식 트리 평가기, `Vec` 필드를 가진 문맥 구조체), `dev-docs/health/2O.md` | 2O-05 | ✅ 실행 프로그램: `tests/run/calc`(전위 표기 식을 `Option<Box<Expr>>` 트리로 파싱하고 넘침 없이 평가, 증명·valgrind 깨끗)·`tests/run/vm`(`Vec` 필드 둘을 가진 문맥 구조체의 스택 기계, 결과를 필드에서 꺼내 반환). 쓰면서 고친 것: 지역 구조체 필드의 `match`(`match l.e`)는 부분 이동(E0413이던 것, `tests/check/err/owned_fields`). 게이트 검출력: 소유 변이체 넷 모두 잡음(39/39, `dev-docs/measurements/gate-power`). 마찰 6행(`m1.md` §6, 셋 고침, 하나 승인 대기 2O-07). 게이트 검출력에 2O-03d 변이 둘을 더해 41/41. 종료 감사 `dev-docs/health/2O.md`, ROADMAP 2O 완료 |
| 2O-07 | 한 갈래 이동의 정적 해제 | 한 갈래에서만 소유 값을 옮기면 E0414(ADR-0031 §2). 2O-06의 `calc::parse`에서 실제로 걸렸다(`let unused: Box<Expr> = a;` 우회, `m1.md` §6). 권장: 덜 옮긴 갈래의 끝에 해제를 넣는 정적 정교화(실행 시 플래그 없음, 사용자 `Drop`이 없어 해제 시점은 관찰되지 않음)로 Rust와 같은 프로그램을 받는다. 루프 머리의 "아마 이동"은 E0414 유지 | 2O-06 | ⏳ 승인 대기(ADR-0031 §2 결정 변경, 열린 질문 3) |

### 2K 제어 흐름 (ADR-0049, 승인 2026-10-26)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2K-01 | 파서·검사기 | `return`·`break`·`continue`·레이블·`loop`(값 있는 `break`)·`?`(같은 오류 타입, 다르면 E0241)·`for i in lo..hi`/`lo..=hi`. E0203·E0204·E0206 은퇴(`diag::RETIRED`). 착수 시 재분해(2026-10-28): 층(검사·증명·코드 생성)으로 나누면 중간 커밋이 증명되지 않는 구성을 받게 되므로(원칙: 검증되지 않는 기능은 넣지 않는다) 구성마다 모든 층을 거치는 2K-01a~e로. 2K-02·2K-03은 각 조각의 완료 기준이 된다 | 2V-12 | ✅ 2K-01a~e 모두 완료(커밋 64364c97·7e68041b·c36ed11a·76edfb6f) |
| 2K-01a | `return` | 문장·식 위치의 `return e`/`return`(타입 `!`), 반환 지점의 `ensures` 의무와 경로 종료, 탈출 간선의 소유 지역 해제, 코드 생성, E0203 은퇴. 코퍼스와 실행 프로그램 | 2V-12 | ✅ 검사: `TypeKind::Never`(어떤 기대 타입에도 맞음, `!`인 갈래는 다른 갈래의 타입, 끝나지 않는 문장으로 끝나는 블록은 `!`), `FnCtx::ret`, 명세의 `return` E0503(안내 포함), 문장 자리의 `if`·`match`가 `}`에서 끝남(파서). 증명: `Enc::post_check`(함수 끝과 `return`이 공유), `Enc::dead`(블록의 뒤 문장 생략, `branch`·단락 논리 연산의 오른쪽은 끝난 쪽을 빼고 남은 쪽의 조건·사실로, 루프 몸이 끝나면 유지·감소 의무 없음). 이동 검사: `MoveInfo::exits`(출구의 소유 경로), `Mv::join`. 코드 생성: `Gen::emit_return`(공유 에필로그), 둘레 임시값·출구 해제, 뒤 코드는 도달 불가 블록(`CVal::Never`). E0203 은퇴(`diag::RETIRED`). 구현 중 찾은 기존 건전성 결함 둘: 루프 havoc이 몸 안 `match` 갈래의 대입과 `&mut` 참조 `match`의 바인딩 쓰기를 빠뜨려 거짓 단언이 증명되었다(`assigned_in_expr`, 2O-04b·그 전부터, 회귀 `tests/verify/fail/loops`의 `arm_assigns`·`binding_writes`). 코퍼스: `tests/verify/pass/returns`, `tests/verify/fail/returns`(반례 5), `tests/check/err/returns`, `tests/run/returns`(valgrind 깨끗). 게이트 변이 넷(`v-return-post-dropped`·`v-return-rest-wrong-side`·`v-havoc-match-arms-skipped`·`c-return-exit-drops-skipped`), `v-ensures-dropped`를 `post_check`로 다시 겨눔 |
| 2K-01b | `break`·`continue`·레이블 | `while` 안의 `break`·`continue`, 레이블 `'a: while`, 출구 상태의 합류(정상 출구와 `break` 지점), `continue`의 불변식·감소 의무, 간선 해제 | 2K-01a | ✅ 렉서: `'l`은 `break`/`continue` 뒤와 `:` 앞(주석을 건너)에서만 레이블(`Tok::Label`), 그 밖은 수명 E0208(형식 퍼저가 `'outer: // c` 사이 주석을 찾음). 검사: 루프 스택(`LoopCx`), 루프 밖 E0250, 없는 레이블 E0317(신규 둘). 증명: `Enc::loop_core`(`while`·`loop` 공유), `break`마다 반복의 경로 조건·상태(`VLoop`), 출구는 "조건 거짓 ∨ 각 `break` 조건"과 상태의 `ite`, `continue`는 그 지점의 불변식 유지·감소(`back_to_head`). 이동 검사: 루프 안 지역의 간선 해제(`MoveInfo::exits`), 출구·머리의 합류점 규칙(`MvLoop`, `loop_backs`). 코드 생성: `GLoop`, `Gen::jump`(루프 안 임시값과 간선 해제 뒤 분기). 파서: `StmtOrTail::Stmt`를 상자로(clippy). 코퍼스: `tests/verify/pass/loop_exits`, `tests/verify/fail/loop_exits`(반례 5종), `tests/check/err/loop_exits`, `tests/run/loop_exits`(valgrind 깨끗, `break`·`continue`에서 `Vec` 해제). 게이트 변이 셋(`v-break-state-dropped`·`v-continue-head-skipped`·`c-jump-exit-drops-skipped`) |
| 2K-01c | `loop`와 값 있는 `break` | `loop`(불변식·`decreases`), `break e`의 값 합류, E0204의 `loop` 몫 은퇴 | 2K-01b | ✅ `ExprKind::Loop`(값은 `break e`의 값, 기대 타입이 모든 `break`의 타입, `break`가 없으면 `!`), `decreases` 없음 E0506, `while`의 값 있는 `break` E0251(신규). 증명은 2K-01b의 `loop_core`(조건 없음: 출구는 `break` 조건들뿐, 값은 그 `ite`), 코드 생성은 결과 자리(`Gen::loop_expr`). E0204는 `for`만 남음(2K-01d에서 은퇴). 근거는 2K-01b와 같은 코퍼스(`first_square_at_least`, `zero`, `two_types`, `no_measure`) |
| 2K-01d | 범위 `for` | `for i in lo..hi`·`lo..=hi`, 머리 불변식(`lo <= i <= end`), 자동 감소, 범위 사실, 2R-04f의 펼친 질의 동일성, E0204 은퇴 | 2K-01c | ✅ `Stmt::For`(`'l:` 레이블, `invariant` 절), 두 끝은 들어갈 때 한 번 계산, 변수는 불변이고 루프 안에서만 보임(E0408·E0301에 `for` 안내). 반복자 E0252·`decreases` E0253(신규), E0204 은퇴. 증명은 `loop_core`의 `ForStep`: 머리 불변식 `lo <= i`와 "`i <= hi` 또는 `i == lo`", 측정 `hi - i`, 반복 끝과 `continue`의 증가(`end_iteration`, `..=`는 그 앞에 `i == hi`의 `break`). 펼친 `while`과 증명 스크립트가 바이트 단위로 같음(시험 `for_queries_are_the_while_expansion`: `..`, `..=`, `continue`·`break`). 코드 생성은 증가 블록으로 가는 `continue`, `..=`는 `i == hi`에서 나감(`u8::MAX`까지 넘침 없음). 코퍼스 `tests/verify/pass/ranges`·`fail/ranges`·`check/err/ranges`·`run/ranges`(valgrind 깨끗). 변이 셋 추가(`v-for-continue-no-step`, `v-for-var-not-havocked`, `c-for-range-ends-swapped`) |
| 2K-01e | `?` | `Option`·같은 오류 타입의 `Result`, 다르면 E0241, `match`와 같은 인코딩, E0206 은퇴 | 2K-01a | ✅ `ExprKind::Try`. 검사(`Checker::try_expr`): 다른 오류 타입·`Option`과 `Result`의 혼용·다른 결과 타입 E0241(신규, 안내 있음), 참조·열거형이 아닌 피연산자 E0401, 명세 E0503. 증명은 펼친 `match`(선언 순서의 갈래, `None`/`Err` 갈래가 `post_check`)와 스크립트가 바이트 단위로 같음(시험 `try_queries_are_the_match_expansion`: `Option`, `Result`). 이동 검사는 피연산자를 옮기고 반환 간선의 해제를 기록, 코드 생성은 태그 비교와 오류를 옮겨 담은 반환(`Gen::try_expr`). E0206 은퇴. 코퍼스 `tests/verify/pass/tries`·`fail/tries`·`check/err/tries`·`run/tries`(소유 오류 `Vec<u8>`, 살아 있는 지역의 탈출 해제, `Option<Box<u64>>` 페이로드, valgrind 깨끗). 변이 둘 추가(`v-try-return-post-skipped`, `c-try-exit-drops-skipped`) |
| 2K-02 | 증명 인코딩 | 탈출 경로(반환 지점의 `ensures`, `break` 상태의 합류, `continue`의 불변식 유지), `?`, `for`의 범위 사실과 자동 `decreases`. 2R-04f의 펼친 질의 동일성 | 2K-01 | ✅ 2K-01a~e가 조각마다 함께 구현: 반환 지점의 `post_check`(2K-01a), `break` 상태의 합류와 `continue`의 `back_to_head`(2K-01b·c), `for`의 머리 불변식·자동 측정과 펼친 `while`과의 스크립트 동일성(2K-01d, `for_queries_are_the_while_expansion`), `?`와 펼친 `match`의 동일성(2K-01e, `try_queries_are_the_match_expansion`). 증명 코퍼스 `returns`·`loop_exits`·`ranges`·`tries` |
| 2K-03 | 해제와 코드 생성 | 탈출 간선의 소유 지역 해제(합류점 규칙을 간선마다), 분기 코드 | 2K-02 | ✅ 2K-01a~e가 함께 구현: `MoveInfo::exits`(반환·`break`·`continue`·`?` 간선의 해제), 간선마다 합류점 규칙(`MvLoop`·`loop_backs`), `Gen::return_expr`·`jump`·`loop_expr`·`try_expr`와 `for`의 증가 블록. 실행 코퍼스 `returns`·`loop_exits`·`ranges`·`tries` 모두 valgrind 깨끗 |
| 2K-04 | 정규형과 기계 수정 | 셀 수 있는 `while`을 `for`로 바꾸는 수정, 코퍼스·라이브러리 이주 | 2K-03 | ✅ W0907(ADR-0057 자율 채택: 동결한 벤치 시드·기준선 프로그램에 셀 루프가 있어 `check` 오류가 아니라 린트 경고): `lint::canonical_loops`가 구문으로 찾고, `bmb fix`가 검사를 통과한 프로그램에 적용(원문을 이어 붙여 주석 보존, 중첩은 회차마다 바깥부터). 이주: 실행 프로그램 14개, 증명 코퍼스 7개, `lib/bytes.bmb`(루프 34곳), 모두 그대로 증명되고 실행 출력 같음. `while` 시험 코퍼스(`loops`·`loop_exits`·`returns`)·틀린 코퍼스·벤치 자산은 그대로. 시험 `counting_while_becomes_a_range_for`(바꾸는 것 하나, 안 바꾸는 넷: 루프 뒤의 쓰임, 중간 증가, `continue`, 끝의 대입) |
| 2K-05 | 퍼저 | 루프 퍼저에 `break`·`continue`·`return`·`?` | 2K-04 | ✅ `tests/soundness_loops.rs` 확장: 셀 루프의 절반을 범위 `for`로, 몸에 `if .. { break; }`·`continue`(`while`은 증가 뒤)·`return e`, 함수의 1/4은 `Option<i64>`를 돌려주고 `acc = lim(e, c)?;`(사후조건은 `None`이면 결과를 말하지 않는 절만). 해석기가 같은 흐름(`Flow`·`Out`)을 따로 돌려 증명·반례·두 빌드와 대조. 씨앗 100~179(release) 480 함수·검증 139(그중 `for` 75, 조기 탈출 44, `?` 12)·검증 실행 826(조기 탈출 111)·반례 226(진입 재현 14)·컴파일 출력 17,040, 불일치 0. 종료 감사(2K-06)의 게이트 검출력에서 조기 탈출 변이를 하나도 못 잡아(탈출 갈래의 상태가 이어지는 갈래와 같아 건너뛴 의무가 드러나지 않음) 생성기를 고침: 탈출 갈래가 먼저 `acc`나 원소를 쓰고, 배치마다 첫 함수가 탈출 종류(`continue`·`break`·`return`·`?`)를 씨앗으로 돌리는 아슬아슬한 합(갈래를 타면 불변식·결과 한도를 크게 넘김). 고친 뒤 게이트 크기(씨앗 1~6)에서 `v-continue-head-skipped`·`v-break-state-dropped`·`v-return-post-dropped`·`v-try-return-post-skipped`를 잡고, 씨앗 100~179(release) 480 함수·검증 83·반례 170, 불일치 0 |
| 2K-06 | 실용 프로그램과 종료 감사 | 조기 탈출을 쓰는 실행 프로그램 2개 이상, `dev-docs/health/2K.md` | 2K-05 | ✅ 실행 프로그램 `tests/run/numlist`(수 목록 읽기: `for`·`continue`·`break`·루프 안 `return`·`Option`의 `?`)·`ranges`·`tries`·`returns`·`loop_exits`, 모두 증명·valgrind 깨끗(P1 41 → 46). 게이트 검출력 53/53(제어 흐름 변이 12개 추가, 루프 퍼저의 빈틈을 찾아 고침), 2K 오류 코퍼스 27줄 모두 안내, 마찰 `m1.md` §6 2K 행 4개. 감사 `dev-docs/health/2K.md`, ROADMAP 2K 완료 |

### 2U 문자열 (ADR-0050, 승인 2026-10-26)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2U-01 | 렉서·검사기 | 문자열 리터럴 `"..."`(`&str`, 인자 자리), `char` 리터럴, `String`·`&str`·`char` 타입과 연산(ADR-0050 §2), E0234 은퇴 | 2V-12, 2O-01 | ⏳ |
| 2U-02 | 증명 인코딩 | UTF-8 불변식(생성 지점 의무, 사용 지점 사실), 문자 경계 의무(E0820), `char` 범위 | 2U-01 | ⏳ |
| 2U-03 | 코드 생성과 런타임 | `String`은 `Vec<u8>` 머리, `&str`은 (포인터, 길이), `from_utf8` 검증, `print_str` | 2U-02, 2O-04 | ⏳ |
| 2U-04 | 퍼저 | 바이트 퍼저에 UTF-8 경계와 `from_utf8` 실패 경로 | 2U-03 | ⏳ |
| 2U-05 | 실용 프로그램과 종료 감사 | 문자열을 다루는 실행 프로그램 2개 이상(예: 토큰 나누기, 진단 문구 조립), `dev-docs/health/2U.md` | 2U-04 | ⏳ |

### 2N 정수와 비트 연산 (ADR-0048, 승인 2026-10-26)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2N-01 | 정수 타입 완비 | `i8`·`i16`·`i32`·`u16`·`u32`·`isize`의 검사·인코딩·코드 생성, 변환(ADR-0028). 착수 시 재분해(2026-10-28): 2N-01a(`TypeKind::Int(IntTy)` 하나로 모으는 리팩터링, 동작 불변), 2N-01b(새 폭을 모든 층에, 코퍼스). 페이즈 순서: 2U의 `char` 변환(`u32::from`, `char::from_u32`)이 `u32`를 쓰므로 2N을 2U보다 먼저 한다 | 2V-12 | ✅ 2N-01a·2N-01b |
| 2N-01a | 정수 타입 표현 통합 | `TypeKind`의 `I64`·`U64`·`U8`·`Usize`를 `Int(IntTy)`(폭·부호)로, 부호·범위·이름을 `IntTy`의 메서드로. 파서는 지금의 넷만 받는다(동작 불변, 전체 시험) | 2N-01 | ✅ `ast::IntTy`(열 폭, `bits`·`signed`·`min_value`·`max_value`·`name`·`from_name`, 지금 받는 넷은 `enabled`), `TypeKind::Int(IntTy)`. 범위 사실·되감기·정수 메서드의 한도·`T::MIN % -1`·리터럴 범위·부정·`MIN`/`MAX`·변환 의무를 `IntTy`의 값으로 계산(질의 문자열은 그대로: V4 불변), 부호 판정은 `is_signed`. 전체 시험·clippy·health 통과 |
| 2N-01b | 새 정수 폭 | `i8`·`i16`·`i32`·`u16`·`u32`·`isize`의 검사·범위 사실·넘침 의무·코드 생성(메모리는 제 폭, 레지스터는 `i64`)·변환(`try_from`)·`MIN`/`MAX`, 코퍼스(맞음·틀림·오류·실행) | 2N-01a | ✅ 파서가 열 타입을 모두 받는다(128비트만 E0402). 증명은 범위 사실·넘침 의무·`T::MIN % -1`·변환 의무를 `IntTy`로(2N-01a), `Vec` 길이 한도는 원소 크기별. 코드 생성: 메모리 제 폭과 읽을 때 부호/0 확장, 좁은 타입 산술은 64비트에서 계산 뒤 범위 검사(`narrow_arith`), 정수 메서드(`narrow_method`, `byte_method`를 일반화), 변환 범위 검사(`in_range_of`). 정수 사이 타입 불일치(E0401)·섞인 폭·범위 밖 리터럴(E0406)·부호 없는 부정(E0411)에 안내. 코퍼스 `tests/verify/pass/widths`·`fail/widths`(반례: `x = 127`, `x = 0`, `n = -32768, d = -1` 등)·`check/err/widths`(모든 줄 안내)·`run/widths`(Rust와 같은 출력 16줄을 손으로 확인, 모든 검사 빌드와 같음, valgrind 깨끗) |
| 2N-02 | 비트 연산과 리터럴 진법 | 비트 연산자(and·or·xor·not·`<<`·`>>`), `0x`·`0o`·`0b`·`_`, 혼합 bitvector 인코딩과 결과 범위 사실, 시프트 양 의무(E0803) | 2N-01 | ✅ 어휘: `0x`·`0o`·`0b`·`_`(`Tok::Int(v, Radix)`), 진법 밖 숫자·숫자 없는 접두 E0108(새 코드, 안내 있음), 접미사 E0104는 접미사만 지운다(`literal_fixes_keep_the_value`). 파서: Rust 우선순위(비교 < or < xor < and < 시프트 < `+`), 붙은 `<` `<`·`>` `>`가 시프트(`Vec<Vec<T>>` 그대로). 정규형: 진법 유지, 구분자·앞의 0 없이 소문자(`ast::Radix::text`, 시험 `literal_bases_and_bit_operators_have_one_normal_form`, ADR-0019 상태 줄에 대체 표시). 검사: and·or·xor는 같은 정수 타입 또는 `bool`, `!`는 정수에서 비트 반전, 시프트 양은 어느 정수 타입이나(Rust와 같음, ADR-0048 구현 시 확인), 명세의 섞인 폭은 둘을 담는 타입, 오류는 E0401과 안내. 증명: 정수는 `Int`, 비트 항만 bitvector(`verify::bv_term`, 실행 코드는 피연산자 bitvector 상수 `Enc::operand_bv`), 선형 꼴(상수 시프트·낮은 비트 마스크·`!`, `linear_bit`), 결과 범위 사실(`bit_facts`), 시프트 양 의무 E0803(`shift_obligation`), 의무는 비트 항을 추상한 질의를 먼저(`abstract_bits`, `--emit-smt`의 첫 시도, `solver_stability.py`가 그것을 잰다). 항이 Rust 연산과 같고 사실이 참임은 `tests/lemmas.rs`의 `bit_terms_are_rusts_operations`·`bit_facts_hold`·`named_operands_are_their_bitvectors`(8비트 모든 쌍과 넓은 타입의 경계·의사 난수 값을 계산). 코드 생성: `and`·`or`·`xor`, `!`는 `xor -1`/`xor MAX`, `ashr`/`lshr`, 좁은 `<<`는 `trunc` 뒤 확장, 증명되지 않은 양은 검사(`Gen::shift`, `an_unproved_shift_traps_exactly_from_the_width`). W0904 변이 and↔or·xor→or·`<<`↔`>>`. 코퍼스 `tests/verify/pass/bits`(12 정의, 모두 rlimit 1만 이하)·`fail/bits`(반례 `k = 32`, `k = -1`, `x = 0`, `x = 255`, `a = 2147483646, b = 2147483647`, `x = -2`)·`check/err/bits`(11줄 모두 안내)·`run/bits`(30줄, rustc로 같은 프로그램을 돌려 확인, 모든 검사 빌드와 같음) |
| 2N-07 | 복합 대입 | 2N-02 발견(마찰): `x += 1`·`h ^= k`·`x <<= 1`이 E0103 일반 구문 오류다(`dev-docs/measurements/syntax-eval-1.md` D6: 시드 33곳·기준 해법 36곳이 `x = x op e` 꼴). Rust의 복합 대입 연산자를 받을지(정본 하나, ADR) 또는 `x = x op e`로 바꾸는 기계 수정(E02xx)을 둘지 정하고 구현, 코퍼스 | 2N-02 | ⏳ |
| 2N-08 | 명세의 정수 타입: 갈래와 결과 | T4-01 발견(마찰): ADR-0022로 명세 산술·비교·인자는 정수 타입을 섞지만, `if` 갈래끼리와 `spec fn` 본문 대 선언 결과 타입은 같아야 하고(E0401) 명세에는 변환이 없다(E0513). 그래서 `u8` 바이트를 `u64` 값으로 쓰려면 항등 `spec fn wide(b: u64) -> u64 { b }`에 넘겨야 했다(`lib/sha256.bmb`의 `pad_byte`). 명세 문맥에서 갈래·결과의 정수 타입 혼합을 받을지(결과 범위 사실을 가정하는 곳이 있는지 먼저 확인) ADR-0022 개정으로 정하고 구현, 우회 `wide` 제거 | 2N-01 | ⏳ |
| 2N-03 | `by (bit_vector)` 하위 증명 | Verus 형태의 비트 수준 단언(E0818), 회전·`count_ones` 등 메서드. 수요: CRC-32 표 방식의 보조정리(T4-04 행) | 2N-02 | ✅ 구문·정규형 `assert(e) by (bit_vector);`(`ast::Prover`, 실행 코드·`proof` 블록·`proof fn` 몸 어디서나, 명세 안 E0503), 항이 아닌 것 E0515·다른 증명기와 `by { .. }` E0516(새 코드, 안내 있음). 하위 증명(`src/subproof.rs`): 인코더의 항을 구간마다 정확한 폭의 bitvector로 옮긴 `QF_BV` 질의, 자유 변수만(각 타입 전체, 경로 사실 없음: ADR-0048 "구현 시 확인"), 바깥은 `(=> (and 범위 가드) e)`를 가정, `#[uses]`의 비재귀 spec fn 펼침(타입 밖일 수 있는 인자는 가정·가드), 실패 E0818과 반례·안내, 드러나지 않은 spec fn은 `bmb uses`가 채움. 메서드 `rotate_left/right`·`wrapping_shl/shr`·`count_ones/zeros`·`leading/trailing_zeros`·`swap_bytes`: 검사(Rust 시그니처), 인코딩(`verify::bit_method`: 상수 양은 선형, 그 밖은 이름 붙은 bitvector 함수와 결과 사실), 코드 생성(`fshl`/`fshr`·`ctpop`·`ctlz`/`cttz`·`bswap`). 추상 질의 합동을 같은 모양의 항으로 넓힘(`bit_congruences`, 표 단계 5,000,000 실패 → 5,859, `lib/sha256.bmb` +0.4%). 수요: `lib/crc32.bmb` 표 방식(T4-04), `lib/sha256.bmb`의 `ROTR`을 `rotate_right`로(22정의 전부 증명 유지, 시그마 넷 6,855 → 4,629, 5,488 → 4,310). 시험: `tests/lemmas.rs`의 `bit_method_terms_are_rusts_methods`·`bit_vector_translations_are_exact`·`bit_vector_goals_and_guards`, `tests/codegen.rs`의 `bit_methods_compute_rusts_results`(열 타입 모두 rustc와 같음), 건전성 퍼저 `tests/soundness_bitvector.rs`(새, 3,200 단언에서 증명 1,263·반례 1,937 모두 해석기와 일치), `tests/uses_fill.rs`의 `a_bit_vector_sub_proof_gets_the_spec_fns_it_unfolds`, `tests/fmt.rs`의 `bit_vector_assertions_have_one_normal_form`. 코퍼스 `tests/verify/pass/bit_vector`(13정의, 최대 rlimit 2,216)·`fail/bit_vector`(반례 `x = 1`, `x = 185`, `y = 128`, `k = 6, x = 105`, `x = 255` 등, 번역 불가 셋)·`check/err/bit_vector`(18줄 모두 안내)·`run/bit_methods`(27줄, `tests/run/bit_methods_ref.rs`를 rustc로 돌린 출력과 같음). 게이트 검출력 변이체 여덟(`scripts/gate_power.py`의 `v-subproof-*`·`*-rotate-direction`·`v-count-fact-strict`·`s-widen-unsigned`) 모두 해당 시험이 잡음. 발견: `2N-09`, `2N-10`, `2N-11` |
| 2N-04 | 퍼저 | 부호 없는 산술 퍼저에 새 폭과 비트 연산 | 2N-03 | ⏳ |
| 2N-05 | 실용 프로그램과 종료 감사 | 비트 연산을 쓰는 실행 프로그램 2개 이상(예: BMB로 쓰고 증명한 SHA-256), `dev-docs/health/2N.md` | 2N-04 | ⏳ |
| 2N-06 | 정수 상수 `T::MIN`·`T::MAX` | 2T-06 발견(마찰): `i64::MIN`이 "unknown type `i64`"(E0305, 잘못된 안내)였다. Rust의 연관 상수 `i64`·`u64`·`usize`·`u8`의 `MIN`·`MAX`를 검사(그 타입)·증명(정수 값)·코드 생성(즉치값)에 넣고, 없는 상수는 정확한 안내(E0305). 실행·실패 코퍼스 | 2V-12 | ✅ `check::int_const`, 검사·`Enc::spec`·`Enc::eval`·`Gen::expr`의 `Path` 갈래, 없는 상수 E0305(`i64::MID`, `tests/check/err/types`). `tests/run/int_consts`(MIN·MAX 출력, `i64::MIN`에서 시작하는 최댓값, `u64::MAX - 1`의 다음 값), `tests/verify/fail/int_consts`(상한 근처 넘침 반례 `x = 2`, `b = 254`) |
| 2N-09 | 바이트 배열 메서드와 남은 비트 메서드 | 2N-03 발견(범위 분할): ADR-0048 §2의 `to_be_bytes`·`from_be_bytes`·`to_le_bytes`·`from_le_bytes`(`[u8; N]` 값, 2V-04로 배열 결과가 됨)와 Rust의 `reverse_bits`·`leading_ones`·`trailing_ones`·`checked_shl`·`checked_shr`·`is_power_of_two`. 검사·인코딩(이름 붙은 bitvector 함수와 결과 사실, `tests/lemmas.rs`)·코드 생성·rustc 차등(`bit_methods_compute_rusts_results`에 더함)·코퍼스 | 2N-03 | ⏳ |
| 2N-10 | 명세 비트 연산의 타입 밖 피연산자 | 2N-03 발견: 명세 산술은 타입 밖으로 나갈 수 있는데(ADR-0022), 비트 연산의 값이 인코더가 고른 꼴에 따라 다르다. 상수 양 `>>`·`!`는 정확한 값으로(`div`, `-a - 1`·`MAX - a`), 변수 양 시프트와 `&`·`^`·OR는 하위 `w`비트로 계산한다. 그래서 `(x + y) >> k`는 `k`가 리터럴인지에 따라 다른 값이다. 실행 코드의 값은 언제나 타입 안이라 거짓 증명은 아니지만 명세의 뜻이 하나가 아니다. 한 정의(예: 피연산자의 2의 보수 상)를 ADR로 정하고 인코딩을 맞춘다. 하위 증명은 인코더의 항을 옮기므로(ADR-0048 구현 시 확인) 그대로 따른다. 확인: 같은 명세 식의 상수·변수 양이 같은 값임을 `tests/lemmas.rs`로 | 2N-03 | ⏳ |
| 2N-11 | 하위 증명 번역의 빈칸 | 2N-03의 `subproof::bit_vector`가 E0818("no bit-vector translation")로 거부하는 것: 부호 있는 `/`·`%`(`bmb_tdiv`·`bmb_trem`), 변수로 나누는 부호 없는 `/`·`%`(SMT의 0 나눗셈과 맞추려면 가드 `b != 0`), 재귀 spec fn(유한 펼침), 그리고 검사기의 E0515인 원소·필드(`let`으로 묶어야 한다). 수요가 나오는 것부터 번역과 `bit_vector_translations_are_exact`의 항을 더한다 | 2N-03 | ⏳ |

### 2M 메서드 구문 (ADR-0051, 승인 2026-10-26)

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2M-01 | 파서·검사기·맵 | `x.f(..)`의 한 단계 수신자 조정, 고유 impl 우선과 trait 모호성(E0313), 맵 L2의 해석된 대상 | 2V-12, 2T-07 | ⏳ |
| 2M-02 | 정규형과 이주 | `T::f(&x, ..)`을 메서드 구문으로 바꾸는 정규형, 코퍼스·라이브러리·퍼저 생성기 이주(퍼저가 메서드 구문을 만든다) | 2M-01 | ⏳ |
| 2M-03 | 종료 감사 | `dev-docs/health/2M.md`, 통과 기준 확인 | 2M-02 | ⏳ |

### 2R 문법 재평가

Rust·Verus 표면을 따른 것은 새 언어의 학습 부족을 줄이기 위한 **수단**이었지 정답이어서가 아니다(사람 지시 2026-10-25). BMB의 목표·조건(사용자는 에이전트, 정확성은 제약·성능은 목적함수, 도구 경유 접근, 정규형 하나, 슬라이스 예산)이라는 새 저울에 문법 요소를 하나씩 올려 **차용·확장·변형·기각**을 정한다. 학습된 지식의 이득과 더 효과적인 새 문법 사이의 균형을 측정으로 잡는다. 결과는 DESIGN.md §4("표면은 승계하고 의미는 엄격화한다")의 개정 제안이 될 수 있으므로 마지막 결정은 사람 승인이다.

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2R-01 | ADR: 문법 평가 저울 | 평가 기준과 측정 방법 정의: (1) 사전 지식 전이(Rust 습관 오류율 E02xx, 1회 성공률), (2) 모호성·정규형(원칙 8), (3) 도구 친화성(결정론적 채움·맵 도출·`fix` 가능성, 원칙 6), (4) 검증 친화성(증명 인코딩 비용·반례 가독성), (5) 토큰 비용(슬라이스·질의 크기, 원칙 5·7), (6) 성능 정당화(원칙 9). 판정 범주 차용·확장·변형·기각과 판정 규칙(어떤 근거면 어느 쪽인지) | 1E-01 | ✅ ADR-0036(정준형 기준 평가, 제약 K1~K4 우선·목적 O1~O5 순서, 판정 절차, 근거 수준 E0~E3, 이탈·회귀 규칙) |
| 2R-02 | 현행 문법 목록과 1차 평가 | M1 표면의 모든 구성(단위 `mod`+`#[summary]`, `fn` 시그니처, 명시 타입, `#[effects]`, `requires`·`ensures`·`decreases`·`invariant`, 이름 붙은 술어, `#[uses]`, `forall`·`#[trigger]`, `old`, `while`만, `&`·`&mut` 2급 참조, 리터럴 규칙 등)과 제안 ADR 0026·0028~0031의 구성을 출처(Rust·Verus·신규)와 함께 목록화하고 2R-01 저울로 1차 평가. 근거: 측정 보고서 §6 마찰, 퍼저·코퍼스 작성 경험, 토큰 측정 | 2R-01 | ✅ `dev-docs/measurements/syntax-eval-1.md`(구성 약 70개), `bench/syntax/measure.py`·`habit_probes.txt`. 측정: 실행 코드는 읽기 토큰의 23%, `#[uses]` 7.8%, 기본 규칙(r2c)이 손으로 쓴 목록과 같은 증명(53/53·11/11·28/28). 후보 순위: `#[uses]` 기본값, 계약 인라인 식, `as`+범위 의무, 2B 표면, 자동 수정 적용, 계수 `for`, Verus 별칭 |
| 2R-03 | 측정 근거 수집 | 1E-10·2A-06 실제 모델 결과에서 구성별 Rust 습관 오류·실패 원인·토큰을 집계(하네스 로그에서 진단 코드와 구성 대응) | 2R-02, 1E-10 | ⏳ |
| 2R-04 | 대안 문법 설계와 오프라인 비교 | 1차 평가에서 "변형·기각" 후보인 구성마다 대안 문법을 설계하고, 결정론적으로 잴 수 있는 것(토큰 수, 정규형 유일성, 맵·슬라이스 크기, 증명 비용)을 m1b·m3 코드에 기계 변환을 적용해 비교. 변환기와 결과 커밋 | 2R-02 | 🔄 하위 2R-04a~h, 보고서 `dev-docs/measurements/syntax-eval-2.md` |
| 2R-04a | 대안 비교: `#[uses]` 기본 문맥 = r2c | m3 생성 코퍼스와 시드 0~4에서 r2c 규칙의 판정·`rlimit_max`·질의 바이트, 한도 초과 정의 수, 증분 키 변화(1차 평가 §5 순위 1) | 2R-02 | ⏳ |
| 2R-04b | 대안 비교: 계약 절 인라인 식 | 단일 사용 술어를 인라인하는 변환기로 m1b 시드·기준 해법의 토큰·L1 크기·`contracts` 답·`rlimit`·BSB 판별률(순위 2) | 2R-02 | ⏳ |
| 2R-04c | 대안 비교: 정수 변환 `as` + 의무 | 변환 지점이 있는 쌍둥이 코드의 토큰, 두 형태의 기계 상호 변환(순위 3, 2C 전) | 2R-02 | ✅ `bench/syntax/conv2c.py`, `syntax-eval-2.md` §2R-04c: 초안 5개·변환 11곳, `as`가 730 → 655토큰(−10.3%, 지점당 −6.8), 의무 9곳·없음 2곳, `try_from -> as -> try_from` 왕복 원문과 같음. 권고: 정본 `x as T`(정수 사이만, 범위 의무) 차용 후보, 결정은 2R-06(승인). 2C-05는 의미를 채택하고 표면은 ADR-0028 제안대로 |
| 2R-04d | 대안 비교: 2B 표면 | prelude `Some`/`None`/`Ok`/`Err`, `self: &T`, `p.f()` 입력 별칭의 토큰, 이름 예약 충돌, 수신자 결정(순위 4) | 2R-02 | ✅ `bench/syntax/surface2b.py`, `syntax-eval-2.md` §2R-04d: 초안 코퍼스 6개에서 prelude −4.3%, `self` ±0, 메서드 −4.9%, 예약 충돌 0, 수신자 7개 모두 지역·매개변수. 권고: prelude 차용, `self: &T` 정본, `x.f()`는 입력 별칭(정본은 경로 호출) |
| 2R-04e | 대안 비교: 쓰기 표면 자동 수정 | 습관 형태 → 정본 수정의 의미 보존(`bmb fix` 전후 증명·IR 동일), 습관 주입 변형의 복구율(순위 5) | 2R-02 | ⏳ |
| 2R-04f | 대안 비교: 경계 루프 `for` | `while` → `for` 변환기로 토큰, 펼친 SMT 질의의 바이트 동일성(순위 6) | 2R-02 | ✅ `dev-docs/measurements/syntax-eval-2.md` 2R-04f 절: 질의는 펼친 `while`과 바이트 동일(시험 `for_queries_are_the_while_expansion`), 토큰은 루프당 약 20(이주한 22개 34곳 −3.3%, 벤치 기준선·`while` 코퍼스 22곳 −5.2%), 판정 모두 같음. 하네스 `bench/syntax/for_tokens.py`, 변환기는 W0907(ADR-0057) |
| 2R-04g | 대안 비교: Verus 명세 별칭 | `exists` 인코딩과 트리거, `T::MAX`, `int`/`nat` 별칭의 토큰(순위 7) | 2R-02 | ⏳ |
| 2R-04h | 2R-04 종합 | 하위 결과의 종합 표와 2R-06 입력(차용·확장·변형·기각 권고) | 2R-04a, 2R-04b, 2R-04c, 2R-04d, 2R-04e, 2R-04f, 2R-04g | ⏳ |
| 2R-05 | 대안 문법 A/B 측정 | 상위 후보에 대해 같은 과제·모델로 기존 문법 대 대안 문법 1회 성공률·습관 오류율 비교(과제 집합은 문법만 바꾼 쌍둥이). 실행은 사용자 머신 | 2R-04, 2A-04 | 🚫 사용자 실행 필요 |
| 2R-06 | ADR: 문법 결정과 DESIGN §4 개정 제안 | 구성별 차용·확장·변형·기각 결정표, 마이그레이션(정규형·`fix`로 기계 변환 가능 여부), DESIGN.md §4 개정안. 채택은 사람 승인. 2B~2G 표면 ADR(0026 등)은 이 결정에 맞춰 갱신 | 2R-03, 2R-04 | ⏳ 승인 |

### 2H 단계 측정

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 2H-01 | 2단계 측정 | `m1b`/`m2`로 측정 5종 재측정, 기능별 Rust 습관 오류율 변화 보고(사용자 머신 실행 포함) | 2A-04 | ⏳ |
| 2H-02 | ADR: 3단계 범위 | 2H-01 결과로 3단계 페이즈(3A~3E) 우선순위 확정, ROADMAP 갱신 | 2H-01 | ⏳ |
| 2H-03 | BRB: 증명 대 테스트 비교 벤치 | 같은 과제를 (a) BMB+증명, (b) Rust+에이전트 테스트(proptest 허용)로 풀게 하고 숨은 적대적 테스트·변이체로 결함률, 총 토큰, 시간 측정. 하네스·과제·Rust 쪽 도구 동결 | 2A-04 | ✅ 기반 구축(유료 실행 없음): `bench/brb`(과제 24개 동결, 같은 문제문, 숨은 판정 9,651케이스·변이체 96개, Rust 팔 도구 `rust_tools.py`, proptest 고정, 정의역 검사, G5 판정 줄). 게이트: 양 팔 오라클 24/24·결함 0, 약한 해법은 자기 검사기를 통과하지만 숨은 판정에서 결함. 편향 공개는 `bench/brb/README.md`. 실제 비교는 사용자 머신 |
| 2H-04 | BRB 조건부 오라클 지표 (H-ORACLE) | BRB 요약에 팔별 "게이트·테스트 통과 & 숨은 실패" 비율을 보고하고 H-ORACLE 판정 줄 추가. 기준 해법·약한 대조군으로 CI 확인 | 2H-03 | ✅ `bench/brb/harness.py`: 세션 끝에 검사기를 최종 상태에서 한 번 더 돌림(`final_check_ok`, 에이전트 호출로 세지 않음), 표에 자기 검사 통과·통과 & 숨은 실패 행, `oracle_verdict` 판정 줄(BMB 비율 ≥ Rust면 반증, Rust 통과 & 숨은 실패 0이면 판정 불가). 결과 행에 최종 파일(`final_sources`)을 담아 G8 등을 새 유료 실행 없이 계산. `test_brb.py`의 `test_oracle_line`·`test_summarize`, 게이트 `bench brb oracle`(24/24, 판정 불가 줄)·`weak`(최종 자기 검사 통과). MANIFEST 재동결(결과 없음) |
| 2H-05 | 검토자 실험(명세 대 diff) | 같은 변경을 diff로 볼 때와 명세 요약+반례로 볼 때 사람 검토자의 결함 발견률·시간 비교(E1 수정안 검증). 사람 참여자가 필요 | 2H-04 | ⏳ 사용자 |
| 2H-06 | G8 테스트 코드량 측정 | BRB 과제에서 같은 숨은 판정 통과 기준으로 BMB 남은 테스트 면적(2D-21)과 Rust 에이전트 테스트 코드 줄 수를 비교·보고. 오라클 팔로 CI 확인 | 2H-03, 2D-21 | 🚫 사용자 실행 대기(INF-14). 하네스 완료: `bench/brb/harness.py`가 결과 행에 `test_code`(BMB: 최종 작업 공간의 `query tests` 요약, Rust: `tests/`와 `#[cfg(test)]` 항목의 코드 줄, `rust_test_lines`)를 담고, 표에 테스트 코드 줄(전체·숨은 판정 통과 과제)과 `g8_verdict` 판정 줄. `test_brb.py`의 `test_g8_line`·`test_rust_test_lines`(약한 대조군 24개 모두 테스트 있음, 기준 해법 0)·`test_summarize`, 게이트 `bench brb oracle`(0·0, 판정 불가 줄). MANIFEST 재동결(결과 없음) |
| 2H-07 | BRB 판정기 실행 경쟁 수정 | 병렬 판정에서 막 쓴 판정 실행 파일을 다른 스레드의 자식이 쓰기 기술자를 쥔 채 fork한 순간 exec하면 `ETXTBSY`로 실패(게이트 `bench brb oracle`, 2026-10-25 `ship.sh`가 차단). 판정 결과는 바꾸지 않고 실행만 재시도. 실제 측정 결과가 없을 때라 MANIFEST 재동결 | 2H-03 | ✅ `bench/brb/judge.py` `spawn_retrying`, `bench/brb/MANIFEST` 갱신, `python3 bench/brb/test_brb.py` |

## 3단계 (M3) — 당겨 온 측정 준비

원칙 7(질의 비용이 코드베이스 크기가 아니라 질의 범위에 비례)은 3단계의 가설이다. 언어를 바꾸지 않고 지금 잴 수 있는 부분을 먼저 잰다(차별점 가설을 먼저 잰다).

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 3A-01 | 대형 코드베이스 생성기 | M1 언어로 정의 1,000~10,000개 규모의 결정론적 작업 공간 생성(단위 구조·호출 깊이·계약 분포를 매개변수로). 전부 증명 | 2A-02 | ✅ `bench/m3/gen.py`(결정론, 단위 국소성: N이 커져도 기존 단위 바이트 동일), `test_gen.py`(게이트 `bench m3`, N=100). N=100·1,000·10,000 전부 증명(차가운 검증 1.4초·25.7초·1,733초) |
| 3A-02 | 질의 비용 척도 측정 | 크기 100·1,000·10,000에서 `slice`·`surface`·`edges`·`impact` 응답 바이트와 시간이 질의 범위에 비례하고 전체 크기에 무관함을 측정, 결과와 하네스 커밋 | 3A-01 | ✅ `bench/m3/scale.py`, `results/2026-10-24.md`. 정의 단위 질의의 응답 바이트는 크기와 무관(원칙 7 성립), 그러나 질의 CPU 시간은 N에 선형(10,000에서 질의당 약 0.4~0.5초: 전체 파싱·검사·모델 구성). 후속 3A-03·3A-04, 2D-14 |
| 3A-03 | 질의 시간을 범위에 비례시키기 | MCP 서버가 프론트엔드 결과(파싱·검사·맵 모델)를 메모리에 두고 바뀐 파일만 다시 처리. 10,000 정의에서 정의 단위 질의 CPU가 크기와 무관함을 `scale.py`로 확인 | 3A-02 | ✅ MCP 서버의 `FrontCache`(파일별 파싱 결과 재사용, 바뀐 것이 없으면 검사·색인 재구성 없음), 맵 `Index`와 역방향 간선 표. N=10,000에서 정의 단위 질의 CPU 420~680ms → 10ms 미만, 응답 바이트 동일. `tests/mcp_cache.rs`(장수 서버 대 호출마다 새 서버, 모든 응답 비교), `bench/m3/results/2026-10-24-front-cache.md`. 커밋 직후 첫 질의는 230ms(전체 검사) |
| 3A-04 | `surface`의 증명 상태를 요청 정의만 계산 | 지금은 전체 프로그램의 증분 세션을 돌림(10,000에서 1.3초). 요청 정의와 그 영향 범위만 | 3A-02, 2D-10 | ✅ `surface DEF`는 DEF만 증명(`verify_only`, `Session::new_for`). N=10,000에서 1.15~1.43초 → 80~140ms, 행은 전체 증명과 바이트 동일. 남은 N 비례 비용(캐시 파일 읽기·세션 준비)은 후속 후보 |
| 3A-05 | BNB: Rust 쌍둥이와 파일 기반 비교 | `bench/m3/gen.py`와 같은 구조의 Rust 쌍둥이 생성, 같은 탐색·수정 질문을 맵 도구(BMB)와 파일·grep 도구(Rust)로 푸는 하네스. 토큰·정확도 | 3A-01 | ✅ `bench/bnb`(Rust 쌍둥이 `twin.py`, Rust 쪽 분석기, 과제 27개 동결, 두 팔의 오라클, 게이트 `bench bnb`). 양쪽 진실 차이 0. 오라클 하한: 범위 과제 토큰 Rust/BMB 3.9배(1k·10k 동일), 전체 2.1배(10k). G4 잠정 목표는 범위 과제에서만 충족. 실제 모델은 사용자 머신. `results/2026-10-25.md` |
| 3A-06 | BNB 크기 축 (H-SCALE) | 정의 수 1k·3k·10k에서 같은 질문 집합을 BMB·Rust 쌍둥이로 생성하고, 오라클 팔로 크기별 정답·비용을 보고. 실제 모델 기울기 비교는 사용자 실행 | 3A-05 | ⏳ |
| 3A-07 | BNB 중복·범위 초과 지표 (H-REUSE) | 수정 과제에서 기존 정의와 같은 기능의 새 정의(정규형 해시·`duplicates`로 판정)와 요구 밖 정의 추가를 세는 판정기. 오라클·대조군으로 CI 확인 | 3A-05 | ⏳ |
| 3A-08 | 수정 과제 집합 (H-MAINT 에이전트 측) | 기존 정의를 바꾸는 과제에서 재증명 범위와 토큰을 BVB의 Verus 쌍둥이와 비교하는 과제·하네스 | 2A-09, 1B-41 | ⏳ |

## 4단계 자립: 부트스트랩 (ADR-0037, 필수)

포팅 착수(4D) 시 stage0(Rust 컴파일러) 기능을 동결한다. 두 컴파일러를 함께 키우지 않는다.

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 4A-01 | 셀프호스팅 격차 분석 | 현재 Rust 컴파일러(`compiler/src`)가 쓰는 언어·라이브러리 기능을 목록화(문자열·바이트, 컬렉션, 데이터 타입, 오류 처리, 재귀 깊이, 파일·프로세스, 서식, 해시 등)하고 기능별 필요도·대체 가능성·2단계 페이즈 대응을 표로. 2단계 우선순위(1E-07)와 2R의 입력 | — | ✅ `dev-docs/measurements/selfhost-gap.md`(+`selfhost-gap-stats.py`). 컴파일러 13,970줄. M1으로는 렉서조차 표현 불가(입력·가변 크기 데이터·바이트 없음). 최소 집합: 2B·2C·2G(+`alloc`)·2E + 미배정 기능(바이트·문자열, 정렬된 맵, `Vec` 필드를 가진 구조체, 시스템 인터페이스). 권고 순서 2B→2C→2G(+alloc)→바이트·문자열→2E→2F 나머지, 4B·4C를 2G 직후로 |
| 4A-02 | 포팅 구성 요소 지도 | 구성 요소별 크기·의존·포팅 순서·차등 시험 방법(같은 입력, 같은 JSONL·IR·맵 바이트) 설계 | 4A-01 | ✅ `dev-docs/measurements/selfhost-port-map.md`. 구성 요소 14개, 모듈 순환 2개(공유 타입 분리로 해소), 증명기 없이 부분 고정점 가능(42%: 기반·렉서·파서·포매터·검사기·코드 생성), 차등 시험 P0~P11·E1/E2, 동결 전 stage0 정리 목록, 탈출구 추정 2~3%(4F 상한 5% 제안) |
| 4A-03 | 동결 전 stage0 정리 | `bmb dump tokens`·`bmb dump parse`(구성 요소별 차등 시험용), `f64` 서식·`{:?}` 출력 제거, 퍼저 말뭉치 내보내기. 출력 불변 테스트 | 4A-02 | ⏳ |
| 4A-04 | 통합 테스트의 CLI 이전 | 라이브러리 API만 쓰는 통합 테스트 15개를 CLI 경유로 바꿔 stage0 은퇴 뒤에도 같은 테스트가 BMB 컴파일러를 시험 | 4A-02 | ⏳ |
| 4B-01 | 표준 라이브러리 설계 ADR | 범위(문자열·바이트, `Vec`, 정렬된 맵, `Option`·`Result`, 서식), 계약 스타일, 증명 비용 | 4B-03 | ⏳ 승인 |
| 4B-02 | 표준 라이브러리 구현 | 4B-01 범위를 BMB로 구현하고 전부 증명, 퍼저 대상 추가 | 4B-01, 2B-09, 2G-05 | ⏳ 승인 |
| 4B-03 | ADR 초안: 표준 라이브러리 | 4A-01 격차와 2B-11·제안 ADR-0031 표면으로 범위·계약 스타일·증명 비용을 정한 초안(상태 "제안"). 2G·2S 설계가 셀프호스팅 요구를 놓치지 않게 하는 입력 | 4A-01, 2B-11, 2V-12 | ⏳ 점검 R12: 제네릭·소유 타입·`String` 결정(2V-12) 위에서 쓴다. 단형 맵 전제는 철회 |
| 4C-01 | 시스템 인터페이스 ADR | 파일·프로세스·인자·종료 코드, 내장 대 FFI, 효과(`io`)와 신뢰 경계 | 4C-03 | ⏳ 승인 |
| 4C-02 | 시스템 인터페이스 구현 | stage0의 외부 호출(Z3·clang 실행, 파일 입출력)을 모두 대체할 수 있음을 예제로 확인 | 4C-01 | ⏳ 승인 |
| 4C-03 | ADR 초안: 시스템 인터페이스 | 파일·프로세스·인자·종료 코드, 내장 대 FFI, `io` 효과와 신뢰 경계(ADR-0030 제안과 대조). 상태 "제안" | 4A-01, 2F-01 | ⏳ |
| 4D-01 | stage0 기능 동결 선언 | 동결 범위와 예외(버그 수정·부트스트랩 필수 변경) ADR, CI 검사(동결 뒤 언어 기능 변경 감지) | 4A-02, 2H-02 | ⏳ 승인 |
| 4D-02 | 렉서·파서 포팅 | 코퍼스·퍼저 입력에서 stage0과 진단·정규형 출력 동일 | 4D-01, 4B-02 | ⏳ |
| 4D-03 | 검사기 포팅 | 진단 JSONL 바이트 동일 | 4D-02 | ⏳ |
| 4D-04 | 증명기 포팅 | SMT 스크립트·상태 줄 바이트 동일 | 4D-03, 4C-02 | ⏳ |
| 4D-05 | 코드 생성 포팅 | IR 바이트 동일, 실행 코퍼스 출력 동일 | 4D-04 | ⏳ |
| 4D-06 | 맵·도구·MCP 포팅 | 맵·질의·도구 출력 동일, MCP 세션 퍼저 통과 | 4D-05 | ⏳ |
| 4E-01 | 부트스트랩 고정점 | stage0→stage1→stage2→stage3, stage2 = stage3 바이트 동일, stage2로 전체 게이트 통과, CI 매 커밋 | 4D-06 | ⏳ |
| 4F-01 | 자기 증명 | BMB 컴파일러가 자기 증명 게이트 통과, 미증명 면적 상한(값은 ADR) 이하 | 4E-01, 2E-03 | ⏳ |
| 4G-01 | stage0 은퇴 | 씨앗 동결·보존(태그·바이너리), 씨앗에서 고정점까지 재현 절차 CI | 4E-01 | ⏳ 승인 |

## 5단계 언어 성숙과 생태계

| ID | 태스크 | 완료 기준 | 의존 | 상태 |
| --- | --- | --- | --- | --- |
| 5A-01 | 언어 레퍼런스 초안 | 현재 언어(M1 + 채택된 확장)의 문법·의미·증명 의무를 절 단위로 기술(`docs/reference/`). 각 절이 진단 코드·코퍼스 사례를 가리킴 | 1D-02 | ⏳ |
| 5A-02 | 적합성 시험 묶음 | 구현과 독립된 시험(입력 → 기대 진단·증명 판정·실행 출력), 명세 절 대응표, 빠진 절 0 | 5A-01 | ⏳ |
| 5B-01 | 안정성·버전 정책 ADR | 버전·에디션, 호환 보장, 폐기 절차, `bmb fix` 이주 | 5A-01 | ⏳ 승인 |
| 5C-01 | 재현 가능한 빌드 | 같은 소스 두 번 빌드 바이트 동일(컴파일러·산출물), CI 확인 | 4E-01 | ⏳ |
| 5C-02 | 다중 타깃 | aarch64 추가, 두 타깃에서 게이트 통과 | 5C-01 | ⏳ |
| 5C-03 | 디버그 정보 | DWARF 줄 정보, 반례 입력으로 디버거 재현 예제 | 1C-01 | ⏳ |
| 5C-04 | 릴리스 절차 | 버전 태그, 바이너리 배포, 라이선스, 변경 기록 | 5B-01 | ⏳ 승인 |
| 5D-01 | 문서 생성기 | 맵에서 단위·정의 문서 생성(설계서 도구, ADR-0007과 범위 조정) | 1D-05 | ⏳ |
| 5D-02 | 테스트 필요 면적 러너 | 컴파일러가 산출한 테스트 필요 정의만 대상으로 하는 테스트 실행 도구 | 2F-03 | ⏳ |
| 5D-03 | 측정 도구의 Python 의존 결정 | 벤치 하네스·스크립트의 Python을 BMB로 옮길지 ADR로 결정(자립 사다리 L4 뒤, ADR-0039 §3) | 4A-04 | ⏳ |
| 5E-01 | 생태계 개방 판단 ADR | 원칙 4·6의 측정 근거(2H·3E)로 패키지·바인딩 개방 여부 | 2H-01 | ⏳ 승인 |

# m1b 측정 (ADR-0025)

2단계 판정용 두 번째 동결 과제 집합. 언어는 M1 부분집합(ADR-0016) 그대로이고, 절차·판정·지표는 ADR-0021과 같다. 하네스는 `bench/m1/harness.py`를 `--set bench/m1b`로 쓴다.

| 파일 | 내용 |
| --- | --- |
| `prompt.md` | 에이전트 시스템 프롬프트(동결). `bench/m1/prompt.md`와 바이트 단위로 같다(ADR-0025 §1, 테스트 `test_m1b_prompt_is_m1_prompt`) |
| `seed/` | 시작 작업 공간(동결). 단위 9개, 정의 116개(실행 함수 53, `spec fn` 63), 모두 증명 |
| `tasks/` | 과제 37개(동결): 작성 25, 탐색 12. 작성 과제마다 숨긴 기준 계약·기준 해법·약한 대조군 |
| `MANIFEST` | 동결 파일의 SHA-256. 다르면 하네스가 실행을 거부한다 |
| `mutants/` | BSB 숨은 변이체(작성 과제마다 `<과제>.json`)와 그 SHA-256 `mutants/MANIFEST`. 위 `MANIFEST`와 따로 동결한다(아래 "명세 변이체 판별률") |

```
python3 bench/m1/harness.py --set bench/m1b manifest                       # 동결 확인
python3 bench/m1/harness.py --set bench/m1b run --agent oracle --jobs 4    # 기준 해법: 전부 성공(CI)
python3 bench/m1/harness.py --set bench/m1b run --agent weak --jobs 4      # 약한 대조군: 전부 실패(CI)
```

## 과제 유형

| 유형 | 과제 | 무엇을 보나 |
| --- | --- | --- |
| 새 정의 | w01~w11, w22~w24(`tasks` 단위), w13(`arr`), w25(`ledger`) | 큰 시드에서 쓸 것을 찾아 재사용하는가(`search::lower_bound`, `order::min_index`, `ledger::deposit`, `arith::sum_to` 등) |
| 기존 정의 강화 | w12, w14~w19 | 계약을 바꾸고 영향받는 정의까지 증명을 유지하는가. w12·w14·w17·w19는 피호출자 계약도 강화해야 한다 |
| 여러 단위 작성 | w20(`num::abs_diff` + `tasks::max_gap`) | 과제의 `also_check` 정의마다 시그니처·명세 강도를 따로 판정 |
| 탐색 | n01~n12 | 직접·전이 호출 관계, `impact`(다시 증명할 정의, 한 단계), 계약 뷰, `&mut` 쓰기, `#[uses]`, 효과. 정답은 하네스가 맵에서 계산 |

## 모델 비교 절차 (백로그 2A-05)

모델 여럿을 같은 조건에서 돌려 `summarize` 표의 열로 비교한다. 실제 모델 실행은 유료라 사용자 머신에서 한다(AUTONOMY.md §4, `bench/m1/README.md`의 "실제 모델 실행").

1. **같은 입력.** 모든 실행이 같은 과제 집합(`--set`), 같은 MANIFEST, 같은 컴파일러 커밋을 쓴다. 먼저 `manifest`와 `run --agent oracle --expect all-pass`로 확인한다. 하네스는 요약 행에 `task_set`, `manifest`(MANIFEST의 SHA-256), `commit`을 기록한다. 커밋이 `+dirty`인 실행은 보고에 쓰지 않는다.
2. **모델마다 한 번.** 과제당 세션 하나, 재시도 없음(ADR-0021 §3). 실제 모델은 `--jobs 1`(기본)로 돌린다. 실패한 실행을 다시 돌려 더 나은 쪽을 고르지 않는다.
3. **결과 파일 이름.** `bench/<집합>/results/<날짜>-<모델>.jsonl`. 날짜는 `date +%F`, 모델은 API 모델 ID 그대로.
   ```
   python3 bench/m1/harness.py --set bench/m1b run --agent anthropic:claude-opus-5-5 \
       --out bench/m1b/results/$(date +%F)-claude-opus-5-5.jsonl
   python3 bench/m1/harness.py --set bench/m1b run --agent anthropic:MODEL_B \
       --out bench/m1b/results/$(date +%F)-MODEL_B.jsonl
   ```
4. **표.** `python3 bench/m1/harness.py summarize bench/m1b/results/A.jsonl bench/m1b/results/B.jsonl`. 파일 하나가 한 열이다. 첫 표는 과제 집합·에이전트·컴파일러 커밋과 ADR-0021 §4 지표(작성·탐색 1회 성공률, 탐색 정밀도·재현율, 질의당 토큰, Rust 습관 오류율, 미증명 의무, `io` 정의, 도구 호출, 한도 도달, 모델 입력·출력 토큰)와 명세 변이체 판별률(BSB, 아래), 둘째 표는 과제별 성공·실패 원인이다.
5. **섞지 않는다.** `summarize`는 과제 집합이 다른 파일을 함께 받으면 거부한다(`task_set`이 없는 이전 결과는 `m1`). `m1`과 `m1b`는 따로 표를 만들고, 수치를 더하거나 평균 내지 않는다. 컴파일러 커밋이 다른 열은 표에 드러나므로 보고에 그 차이를 적는다.
6. **보고.** 원자료(결과 파일)를 커밋하고, 요약은 `dev-docs/measurements/`에 쓴다. 과제 편향(정수·배열 한정), 과제 수, 한도, 실패 사례를 함께 적는다(ADR-0021 §5).

## 명세 변이체 판별률 (BSB, ADR-0034 G3, 백로그 2A-10)

작성 과제마다 기준 해법의 숨은 변이체 집합이 있다(`mutants/<과제>.json`, 동결은 `mutants/MANIFEST`). 하네스는 세션이 끝난 뒤 에이전트 계약이 변이체를 몇 개 거부하는지 세고(`bsb_mutants`·`bsb_killed`·`bsb_survivors`), `summarize`의 "명세 변이체 판별률" 행에 합을 낸다. 판정 코드는 `bench/m1/bsb.py`, 생성기는 `bench/m1/bsb_gen.py`.

- **변이체.** 목표 정의와 그것이 (전이적으로) 부르는 실행 함수마다 `bmb mutate`가 만드는 W0904 변이 하나(연산자·상수·조건·인덱스±1·`if` 조건 부정). 텍스트를 직접 고치지 않고 컴파일러가 구문 트리를 바꿔 정규형으로 출력한다.
- **실행.** 기준 해법과 각 변이체를 과제마다 고정한 입력(모두 숨은 `ref_pre`를 만족)으로 돌린다. 구동기는 실제 백엔드로 빌드한다: `bmb emit-llvm --all-checks`(증명 없음, 계약을 믿지 않으므로 넘침·범위·나눗셈 검사가 모두 trap으로 남음) + clang. trap이 나면 다음 입력부터 다시 빌드한다.
- **분류.** `no_check`(검사 실패) / `equivalent`(모든 입력에서 같은 출력 — 고정 입력 안에 구별 입력이 없음) / `trap_only`(trap·무한 루프로만 다름: 계약은 결과를 말하므로 증명 게이트의 몫) / `allowed`(정상 종료하며 다르지만 모든 출력이 숨은 기준 계약을 만족: 같은 최솟값의 다른 인덱스 등) / **보존**(숨은 기준 계약이 거부하는 출력을 내는 입력, 즉 증인이 있음). 증인 판정은 아래와 같은 죽임 검사를 숨은 계약을 가진 스텁에 대고 `bmb verify`로 한다. 증인은 입력 순서로 최대 2개.
- **죽임.** 변이체마다 판정용 함수 하나: 증인 입력으로 에이전트 함수를 부르고(전제 조건 증명) 그 결과가 변이체 출력과 모두 같을 수는 없다는 `assert`를 증명한다. 함수는 계약으로만 보이므로(ADR-0017 §4) 증명되면 계약이 그 변이체를 거부한 것이다. 호출들이 독립이라 "증인 중 하나라도 계약이 배제" 와 같다. 과제당 `bmb verify` 한 번(세션 캐시 사용). 작업 공간이 증명되지 않았거나 시그니처가 다르면 0개를 죽인 것으로 센다.
- **한계.** 변이 연산자는 W0904 집합뿐이라(변수 바꾸기·문장 삭제 없음) 일부 약한 계약의 약점(예: w11 범위 안 원소 보존)을 드러내는 변이체가 없다. 동치 판정은 고정 입력에 대한 시험이지 증명이 아니다. 계약의 전제 조건이 `ref_pre`보다 강해 증인 입력을 거부하면 그 변이체는 죽지 않는다.

| 생성(2A-10) | 수 |
| --- | --- |
| 생성한 변이체 | 426 (목표 26개: 작성 과제 25 + w20의 `num::abs_diff`) |
| 보존 | 186 |
| 버림: `equivalent` / `trap_only` / `allowed` / `no_check` | 31 / 114 / 95 / 0 |
| 기준 해법(oracle) 계약 | 186/186 (100.0%) |
| 약한 대조군(weak) 계약 | 45/186 (24.2%) |

```
python3 bench/m1/bsb_gen.py --jobs 4            # 다시 생성(약 8분, 결과는 결정적). 바꾸면 MANIFEST도 새로 씀
python3 bench/m1/bsb_gen.py --check --tasks w03-max3   # 다시 만들어 파일과 같은지만 확인
```

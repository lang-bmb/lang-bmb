# BRB: BMB+증명 대 Rust+테스트 (ADR-0034 G5, 백로그 2H-03)

전제 E2·E3(DESIGN.md §2)를 직접 재는 비교 벤치다. 같은 과제를 같은 모델·같은 예산으로 (a) BMB(증명, MCP 도구)와 (b) Rust(에이전트가 쓴 테스트, `cargo test`, proptest)로 풀게 하고, 세션이 끝난 뒤의 **구현**을 숨은 적대적 판정기로 행동 기준에서 잰다. 지표는 숨은 결함률, 총 모델 토큰(쓰기+검증), 도구 호출, 벽시계다.

이 디렉터리는 하네스와 과제(동결)와 CI 검사까지다. 실제 모델 실행은 유료라 사용자 머신에서 사람 승인 뒤에 한다(AUTONOMY.md §4). 아직 측정 결과는 없다.

| 파일 | 내용 |
| --- | --- |
| `tasks/tNN_*.py` | 과제 24개(동결). 자연어 문제·정의역·시그니처, 숨은 판정 자료(모델, 케이스 생성기, Rust 변이체), 기준 해법과 약한 대조군(BMB·Rust 각각) |
| `prompt-bmb.md` | BMB 비교군 시스템 프롬프트(동결). `bench/m1/prompt.md`의 언어 규칙과 도구 설명 그대로, "How to work"만 BRB용 |
| `prompt-rust.md` | Rust 비교군 시스템 프롬프트(동결) |
| `rust-template/` | Rust 비교군의 시작 크레이트 `solution`(동결). `proptest =1.12.0` 개발 의존성과 `Cargo.lock` |
| `rust_tools.py` | Rust 비교군 도구 서버(동결): `list_files`, `read_file`, `write_file`(`src/`·`tests/`의 `.rs`만), `cargo_build`, `cargo_test` |
| `judge.py` | 숨은 판정기(동결) |
| `cases.py` | 케이스 생성 도구(동결). 과제 id로 시드한 결정론적 난수 |
| `harness.py` | 세션 실행·판정·요약(동결) |
| `MANIFEST` | 위 파일들의 SHA-256. 다르면 하네스가 실행을 거부한다 |
| `test_brb.py` | 하네스 단위 테스트(게이트) |

```
python3 bench/brb/harness.py manifest                                   # 동결 확인
python3 bench/brb/harness.py run --arm bmb  --agent oracle --jobs 4     # 기준 해법: 결함 0 (CI)
python3 bench/brb/harness.py run --arm rust --agent oracle --jobs 4     # 기준 해법: 결함 0 (CI는 --proptest off)
python3 bench/brb/harness.py run --arm bmb  --agent weak --jobs 4       # 약한 대조군: 자기 검사는 통과, 숨은 판정이 결함 검출 (CI)
python3 bench/brb/harness.py adequacy --jobs 4                          # 숨은 케이스가 Rust 변이체 96개를 모두 죽인다 (CI)
python3 bench/brb/harness.py summarize BMB.jsonl RUST.jsonl             # 표와 G5 판정 줄
```

## 과제

M1 BMB 부분집합(ADR-0016: 정수, 슬라이스, `while`)과 Rust 양쪽에서 풀 수 있는 함수 과제 24개. 각 과제는 두 비교군에 **같은 문장**(문제와 입력 정의역)을 주고, 시그니처 한 줄만 언어별로 다르다(`judge.task_prompt`, 테스트 `test_both_arms_get_the_same_statement`). 시그니처는 매개변수 타입이 양쪽에서 같다(`i64`, `u64`, `usize`, `&[i64]`, `&mut [i64]`; BMB는 `-> (r: T)`, Rust는 `pub fn .. -> T`).

| 묶음 | 과제 | 흔한 결함 |
| --- | --- | --- |
| 넘침·표현 범위 | t02 midpoint, t03 sat_sum, t09 div_floor, t10 mod_floor, t16 gcd, t17 isqrt, t18 count_digits, t24 pow_mod | `a + b`, `abs(MIN)`, `MIN % -1`, `mid * mid`, `b * b`, 부동소수 `sqrt` |
| 경계·동률·하나 차이 | t01 clamp, t04 count_in_range, t05 first_max_index, t06 last_min_index, t07 strictly_increasing, t08 lower_bound, t12 second_max, t19 range_sum, t20 majority, t21 longest_run | `<` 대 `<=`, 첫/마지막, 빈 슬라이스, `max_by_key`가 마지막을 고름, 검증 없는 Boyer–Moore |
| 제자리 변환 | t11 max_subarray, t13 rotate_right, t14 dedup_sorted, t15 move_zeros, t22 merge_sorted, t23 sort | `k > len`, 꼬리 정리 누락, 순서 보존, 남은 쪽 복사 누락 |

정의역이 있는 과제 13개는 BMB용 정의역 술어(`BMB_DOMAIN`)도 갖는다(아래 판정 2단계).

## 숨은 판정 (`judge.py`)

에이전트는 판정 자료를 보지 못한다(과제 프롬프트에는 문장·정의역·시그니처만 들어간다). 두 비교군을 같은 방식으로 판정한다.

- **기대값은 Python 모델에서** 나온다(`model`). 기준 구현에서 뽑지 않으므로 기준 구현의 오류가 판정에 섞이지 않는다. 기준 해법(BMB·Rust)이 모든 케이스에서 모델과 일치하는지를 CI가 확인한다.
- **케이스**: 과제당 267~810개, 합계 9,651개. 경계값(`i64::MIN`, `MIN+1`, `-1`, `0`, `MAX` 등)의 조합, 넘침 근처 값, 길이 0·1·2와 난수 길이, 중복이 많은 원소 분포와 전 범위 분포, `k^2-1` 같은 과제별 함정. 정의역 밖 케이스는 생성 단계에서 버린다(테스트가 확인).
- **판정기 적정성**: 과제마다 그럴듯한 오답 Rust 구현(변이체) 4개, 합계 96개. 숨은 케이스가 모두 죽여야 한다(`adequacy`, CI).
- **결함**: 출력이 다르거나, 패닉·트랩·충돌·시간 초과. 프로세스가 죽거나 멈추면 그 케이스를 실패로 적고 다음 케이스부터 다시 돈다(`run_cases`, 프로세스당 20초, 재시작 25회 한도).
- **미제출**: 구현이 빌드되지 않으면 모든 케이스가 실패다.

비교군별 절차:

| 단계 | BMB | Rust |
| --- | --- | --- |
| 1. 제출 | 대상 `tasks::NAME`이 요구 시그니처(이름은 무관, 타입만)를 갖고, 작업 공간 전체가 증명된다(`bmb emit-llvm` 종료 코드 0 = `bmb build` 조건) | 판정 크레이트가 에이전트 크레이트에 경로 의존해 빌드된다(에이전트의 테스트·개발 의존성은 쓰지 않음) |
| 2. 정의역 | 숨긴 단위가 `requires dom(..)`에서 대상을 부르는 함수를 증명한다. 즉 에이전트의 `requires`가 문장에 적힌 정의역에서 따라 나와야 한다. 실패하면 출력이 맞아도 결함(정의역 위반)이다 | 해당 없음(Rust에는 전제가 없다. 정의역의 입력에서 패닉하면 결함) |
| 3. 실행 | 생성한 LLVM 래퍼 + C 드라이버로 대상을 호출. 증명된 프로그램이라 런타임 검사가 없다 | 생성한 `main`이 `catch_unwind`로 호출. 디버그 빌드(`overflow-checks = true`)라 넘침은 패닉 |

## 비교군과 공정성 규칙

ADR-0021(동결, 1회, 재시도 없음, 편향 명시)과 ADR-0034 §2(같은 모델·같은 토큰 상한, Rust 쪽 도구도 같은 수준)를 따른다.

1. **같은 모델·같은 설정.** 두 비교군 모두 `bench/m1/anthropic_agent.py`의 같은 어댑터(같은 `EFFORT`, 요청당 `MAX_TOKENS`)로 돈다. 도구는 MCP 모양(`name`, `description`, `inputSchema`)이라 어댑터가 구분하지 않는다.
2. **같은 예산.** 세션당 도구 호출 40회(ADR-0021 §3), 세션당 모델 출력 토큰 64,000(`SESSION_OUTPUT_TOKENS`, 모델 차례마다 확인). 한도에 닿으면 그 시점의 구현을 판정한다.
3. **둘 다 검사기를 받는다.** BMB: `verify`(진단·반례), Rust: 컴파일러 오류(`cargo_build`)와 테스트 실행(`cargo_test`, proptest 허용, ADR-0034 결정 5). 둘 다 셸·네트워크는 없다(Rust의 cargo는 늘 `--offline`, 의존성은 세션 전에 한 번 받아 미리 빌드한다).
4. **같은 판정.** 위 숨은 판정기. BMB만 정의역 검사를 더 받는 것은 BMB에서 "증명된 함수"의 계약이 호출 조건이기 때문이다(아래 편향 공개).
5. **1회.** 과제당 세션 하나, 실패해도 다시 돌리지 않는다. 같은 커밋·같은 MANIFEST로 두 비교군을 돌린다(`summarize`는 MANIFEST가 다른 결과를 거부한다).

## 지표와 G5 판정

`summarize`는 결과 파일 하나를 한 열로 하는 표를 낸다: 성공(숨은 케이스 전부 통과), 결함률(결함 과제 / 과제), 미제출, 정의역 위반, 실패 케이스 비율, 총 모델 토큰(입력+출력)과 입력·출력 각각, 도구 호출, 검사 도구 호출, 한도 도달, 세션 벽시계, 도구 시간. 그다음 과제별 표.

BMB 결과 하나와 Rust 결과 하나를 주면 G5 판정 줄을 붙인다(ADR-0034 잠정 기준, M2):

```
G5 (잠정 기준: 결함률 비 ≤ 1/2, 총 토큰 비 ≤ 2): 결함률 비 X, 총 토큰 비 Y → 통과 | 미달 | 판정 불가(..)
```

결함률 비 = BMB 결함률 / Rust 결함률, 총 토큰 비 = BMB 총 토큰 / Rust 총 토큰. Rust 결함이 0이면 비를 정의할 수 없어 "판정 불가"다(그때는 표의 결함률을 그대로 보고한다).

## 실제 모델 실행 (사용자 머신, 유료)

```
python3 bench/brb/harness.py manifest
python3 bench/brb/harness.py run --arm bmb  --agent anthropic:claude-opus-5-5 --out bench/brb/results/$(date +%F)-claude-opus-5-5-bmb.jsonl
python3 bench/brb/harness.py run --arm rust --agent anthropic:claude-opus-5-5 --out bench/brb/results/$(date +%F)-claude-opus-5-5-rust.jsonl
python3 bench/brb/harness.py summarize bench/brb/results/<날짜>-claude-opus-5-5-bmb.jsonl bench/brb/results/<날짜>-claude-opus-5-5-rust.jsonl
```

- Rust 비교군은 기본으로 proptest를 켠다(`--proptest on`). 처음 한 번 crates.io에서 `Cargo.lock`의 크레이트를 받는다(네트워크 필요). 그 뒤 세션은 오프라인이다.
- 실제 모델은 `--jobs 1`(기본). 두 비교군을 같은 커밋에서 연달아 돌리고, `+dirty` 커밋의 결과는 보고에 쓰지 않는다.
- Rust 비교군에서는 에이전트가 쓴 코드(테스트)가 세션 중에 실행되고, 판정 단계에서는 두 비교군의 구현이 실행된다. 격리된 환경(컨테이너·VM)에서 돌린다.
- 원자료는 `bench/brb/results/`에, 요약은 `dev-docs/measurements/`에 커밋한다(ADR-0021 §5). 모델 2개(주 모델, 작은 모델, ADR-0034 결정 3)는 따로 표를 만든다.

## CI (`scripts/check.sh`)

| 작업 | 확인 |
| --- | --- |
| `bench brb oracle` | 두 비교군의 기준 해법이 숨은 판정을 모두 통과(24/24, 결함 0), BMB 기준 해법은 정의역 검사도 통과. 요약 표와 G5 줄("판정 불가(Rust 결함 0)") |
| `bench brb weak` | 두 비교군의 약한 대조군이 자기 검사(BMB `verify`, Rust 자기 테스트)는 통과하는데 숨은 판정이 24과제 모두에서 결함을 찾는다 |
| `bench brb adequacy` | Rust 변이체 96개가 모두 죽는다 |
| `bench brb tests` | `test_brb.py`: 과제 집합 불변식(결정론, 정의역, 크기, 같은 문장), G5 계산과 요약 표, 충돌·멈춤 복구, Rust 도구 샌드박스, 검사기 판정, 실제 모델 어댑터(가짜 클라이언트)로 Rust 세션과 출력 토큰 상한. proptest 세션 테스트는 proptest 크레이트가 오프라인으로 있을 때만 돈다(없으면 건너뜀) |

CI는 Rust 비교군을 `--proptest off`로 돌린다. 대본 에이전트는 proptest를 쓰지 않고, 게이트는 네트워크에 기대지 않는다.

## 편향과 한계 (공개)

- **과제 선택.** 과제는 우리가 썼고, M1 BMB에서 풀리고 증명되는 것만 골랐다: 정수와 슬라이스, 작은 알고리즘 함수. 구조체·컬렉션·문자열·입출력이 없다. 넘침·경계 결함이 드러나기 쉬운 과제가 많고(넘침 8개, 경계 10개), 이는 정수 넘침을 증명 의무로 강제하는 BMB에 유리할 수 있다. 공개 과제를 이식하지 않았으므로 빠진 과제의 비율은 해당 없음이다. 과제 묶음별 결과를 따로 보고한다.
- **정의역 검사는 BMB에만 있다.** BMB 에이전트의 `requires`가 문장의 정의역과 의미가 같아도 증명기가 함의를 못 찾으면 결함으로 센다(증명기 불완전성). 이런 경우가 나오면 과제별 표의 "정의역 위반"으로 따로 보인다.
- **BMB에는 실행 도구가 없다.** BMB 에이전트는 테스트를 돌릴 수 없고 증명만 한다(설계상 비교 대상). `build`로 실행 파일은 만들 수 있지만 실행 도구는 없다.
- **입력 토큰.** BMB 시스템 프롬프트(언어 규칙 포함)가 Rust 프롬프트보다 길다. 총 토큰은 모델이 보고한 입력+출력 그대로 센다(프롬프트 비용 포함).
- **판정 범위.** 숨은 케이스는 짧은 슬라이스(대개 길이 20 이하)다. 긴 입력에서만 나는 결함(시간 초과 등)은 재지 않는다. 시간은 벽시계만 기록하고 성능은 판정하지 않는다.
- **Rust 판정 빌드.** 디버그 빌드(넘침 검사 켬)로 판정한다. 릴리스 빌드에서 감싸 넘기는 코드도 결함으로 센다(정의역의 입력에서 수학적으로 틀린 값이므로).

# REVIEW: 명세 diff 검토 대 코드 diff 검토 (H-REVIEW, ADR-0078, 백로그 T1-44d)

가설 H-REVIEW: 검토자가 명세 diff만 보고도 코드 diff 검토만큼 결함을 잡는다. 전제는 "증명 게이트를 통과한 결함은 명세 결함"이다. 그래서 변경의 앞뒤 프로그램은 모두 증명된 프로그램이고, 결함은 계약이 의도를 말하지 않게 된 변경이다.

이 디렉터리는 변경 집합(동결)·보기·판정·하네스·게이트까지다. 실제 모델 실행은 아직 없다(측정 머신에서 한다).

| 파일 | 내용 |
| --- | --- |
| `changes/*.json` | 변경 143개(동결된 산출물). `gen.py`가 BVB 과제(`bench/bvb/tasks`, 동결)에서 만든다 |
| `gen.py` | 생성기(동결). `--check`: 컴파일러 없이 구조 확인. `--regen`: 컴파일러로 다시 만들어 바이트 단위 비교 |
| `mutants.py` | 본문 변이 변경의 도출(동결): `bmb mutate`, `bmb verify`, 실행 비교, 숨은 계약의 거부 증명 |
| `set.json` | 팔(보기 부분의 목록), 비교하는 두 팔, 토큰 비 기준, 생성에 쓴 컴파일러 커밋, MANIFEST에 더 넣는 파일(동결) |
| `prompt.md` | 리뷰어 시스템 프롬프트(동결). 두 팔이 같은 프롬프트를 쓴다 |
| `views.py` | 리뷰어가 보는 것(동결): 계약 텍스트 추출, diff, `bmb weaken` 실행, 메시지 |
| `judge.py` | 판정(동결): 답 형식, 탐지·오탐·형식 오류, 가설 판정 규칙 |
| `harness.py` | 실행·결과 파일·표(동결) |
| `MANIFEST` | 위 동결 파일과 `../bvb/MANIFEST`의 SHA-256. 다르면 실행을 거부한다 |
| `test_review.py` | 게이트 |

```
python3 bench/review/harness.py manifest                                   # 동결 확인
python3 bench/review/gen.py --check                                        # 구조 확인(컴파일러 없음)
python3 bench/review/gen.py --regen                                        # 다시 만들어 바이트 단위 비교(컴파일러, 약 13분)
python3 bench/review/harness.py sizes                                      # 팔·종류별 보기 크기와 토큰 비(모델 없음)
python3 bench/review/harness.py verdicts                                   # 변경마다 `bmb weaken` 판정, 종류별
python3 bench/review/harness.py view --arm spec --change vb-misc-sum.weaken  # 리뷰어가 받는 메시지
python3 bench/review/harness.py run --arm spec --agent oracle --expect all-right      # 대본: 전부 맞힘
python3 bench/review/harness.py run --arm code --agent blind --expect none-detected   # 대본: 전부 놓침
python3 bench/review/harness.py run --arm spec --agent openai:MODEL --out bench/review/results/DATE-MODEL-spec.jsonl
python3 bench/review/harness.py run --arm code --agent openai:MODEL --out bench/review/results/DATE-MODEL-code.jsonl
python3 bench/review/harness.py table [--d PP] SPEC.jsonl CODE.jsonl       # 팔별 표와 H-REVIEW 판정 줄
```

## 변경 집합

BVB 과제 45개는 저마다 의도 문장(프롬프트의 설명), `reference`(의도를 말하는 계약), `weak_reference`(같은 코드에 일관되고 증명되지만 의도를 말하지 않는 계약)를 갖는다.

| 변경 | 종류(`kind`) | 앞 → 뒤 | 숨긴 라벨 | 개수 | 과제 |
| --- | --- | --- | --- | --- | --- |
| `<과제>.weaken` | `contract-weakened` | `reference` → `weak_reference` | `defect` | 45 | 45 |
| `<과제>.restore` | `contract-restored` | `weak_reference` → `reference` | `clean` | 45 | 45 |
| `<과제>.mutant-defect-N` | `contract-weakened+body` | `reference` → 약한 계약 + 본문 변이체 | `defect` | 26 | 17 |
| `<과제>.mutant-clean-N` | `body-only` | `reference` → 같은 계약 + 본문 변이체 | `clean` | 27 | 20 |

합계 143개: `defect` 71, `clean` 72. 본문이 바뀌는 변경은 53개다.

- 손으로 쓴 프로그램은 없다. 의도는 과제 프롬프트에서 "Create unit" 줄과 "Required signature" 줄을 뺀 설명이고(주어진 명세 함수의 코드 포함), 프로그램은 과제의 `put` 코드를 단위 `tasks` 안에 넣은 파일 하나와 BVB 씨앗(`given`)이다. 정의 순서는 과제 그대로다(컴파일러의 정규형이 아니다). 변이 변경의 뒤 프로그램은 그 텍스트에서 컴파일러(`bmb mutate`, W0904)가 댄 자리 한 곳을 바꾼 것이고, `bmb fmt` 결과가 컴파일러가 낸 변이체와 같은 것만 쓴다.
- **본문 변이의 규칙(측정 전에 고정):** 과제마다 `bmb mutate`가 내는 순서(단위의 실행 함수를 소스 순서로, 함수 안은 컴파일러의 자리 순서)에서 조건을 충족하는 첫 K = 2개를 종류마다 쓴다.
  - `body-only`: `reference` 프로그램의 변이체가 바뀌지 않은 기준 계약에서 `bmb verify`를 통과한다(동치이거나 계약이 허용하는 고쳐 쓰기).
  - `contract-weakened+body`: `weak_reference` 프로그램의 변이체가 약한 계약에서 `bmb verify`를 통과하고, 증거 입력이 있다: 숨은 사전 조건이 증명된 입력에서 변이체가 끝나고, 출력이 기준 프로그램과 다르며, 숨은 기준 계약이 그 출력을 거부함이 증명된다(`bench/m1/bsb.py`의 죽임 검사와 같은 꼴, `emit-llvm --all-checks`와 clang으로 실행). 증거는 변경 파일의 `witness`에 남는다(리뷰어는 못 본다).
  - 입력은 과제별로 쓰지 않았다. 시그니처의 타입에서 과제 id로 시드한 생성기가 60개를 뽑고(작은 정수, 길이 0~5 배열), 숨은 사전 조건이 증명되는 것만 쓴다. 그 입력에서 차이가 드러나지 않은 변이체는 넣지 않는다(결정된 것만).
- 생성 결과(커밋 `9938d706`): 기준 프로그램의 변이체 602개(자리를 텍스트에 옮기지 못한 것 6). 약한 계약에서 증명된 변이체가 있는 과제는 28개인데 그중 11개는 증거가 나오지 않았다(1개는 사전 조건이 증명된 입력이 없음).
- **커밋된 `changes/`가 동결된 산출물이다.** 어떤 변이체가 조건을 충족하는지는 컴파일러에 달려 있으므로 실행 때 다시 만들지 않는다. 생성에 쓴 컴파일러 커밋은 `set.json`의 `generated_with`에 있다. `gen.py --check`는 컴파일러 없이 확인한다: 계약만 바뀌는 90개는 바이트 단위로 다시 만들어 비교하고, 변이 변경은 앞이 기준 프로그램이고 뒤가 기록된 한 자리 치환을 적용한 프로그램인지, 과제·종류당 1..n(n ≤ K)인지 본다. `gen.py --regen`은 전체를 다시 만들어 바이트 단위로 비교한다(`9938d706`에서 일치, 약 13분).
- 라벨의 근거(게이트): 서로 다른 프로그램 143개(기준 45, 약한 45, 변이 53)가 모두 `bmb verify`를 통과하고, m1 판정(ADR-0021 §2: 계약이 과제의 숨은 기준 계약을 함의)이 기준 계약을 가진 프로그램(`clean`의 뒤)을 모두 받아들이고 약한 계약을 가진 프로그램(`defect`의 뒤)을 모두 거부한다.
- `bmb weaken`은 두 계약을 서로 비교할 뿐 의도를 모르고, 결정하지 못할 수 있다. 커밋 `9938d706`에서:

| 종류 | 라벨 | `weakened` | `undecided` | `refined` | `compatible` | `same` |
| --- | --- | --- | --- | --- | --- | --- |
| `contract-weakened` | `defect` | 33 | 12 | 0 | 0 | 0 |
| `contract-weakened+body` | `defect` | 19 | 7 | 0 | 0 | 0 |
| `contract-restored` | `clean` | 0 | 2 | 33 | 10 | 0 |
| `body-only` | `clean` | 0 | 0 | 0 | 0 | 27 |

  게이트는 개수가 아니라 "결정한 것은 라벨과 어긋나지 않는다"(`defect`가 `same`·`refined`·`compatible`로, `clean`이 `weakened`로 나오지 않고, `body-only`는 `same`)를 확인한다.

## 팔 (`set.json`)

| 팔 | 보는 것 |
| --- | --- |
| `spec` | 계약 diff(함수의 시그니처 줄, `requires`·`ensures` 줄, 그 줄들이 직접·간접으로 부르는 단위 안의 `spec fn` 정의 전체의 unified diff)와 `bmb weaken before after`의 출력 그대로 |
| `code` | 프로그램 파일(`src/tasks.bmb`) 전체의 unified diff |

두 팔 모두 의도 문장을 먼저 받는다. 계약이 바뀌지 않은 변경(`body-only`)의 계약 diff는 빈 문자열이 아니라 `(no difference)` 한 줄이다. 본문, 속성(`#[uses]`·`#[effects]`), `decreases`, 루프 불변식, 증명만 쓰는 `spec fn`은 계약 텍스트가 아니다. 씨앗의 `given::..`은 바뀌지 않으므로 어느 diff에도 없다(그 코드를 주는 과제는 의도 문장에 들어 있다). `bmb weaken`은 실행 때 그 커밋의 컴파일러로 돌리므로 결과 머리의 커밋이 보기를 정한다. 크기는 보기의 바이트와 `ceil(바이트 / 4)` 토큰이다(ADR-0021 §4). 의도 문장을 더한 메시지 크기도 함께 적는다.

## 판정 (첫 실행 전에 등록, 2026-11-12)

리뷰어는 변경마다 요청 한 번(도구 없음, 재시도 없음)에 JSON 객체 하나로 답한다: `{"verdict": "defect" | "ok", "reason": "한 줄"}`.

- **탐지:** `defect` 변경에 `"defect"`. **오탐:** `clean` 변경에 `"defect"`.
- **형식 오류:** 그 객체 하나(또는 코드 울타리 하나 안의 그 객체)가 아닌 답. 앞뒤의 다른 글, 다른 키, 다른 `verdict` 값, 문자열이 아닌 `reason`, API 오류로 빈 답이 모두 여기다. 따로 세고, 탐지로 치지 않으며 오탐으로도 치지 않는다. 분모에는 남는다.
- **탐지율** = 탐지 / `defect` 71, **오탐율** = 오탐 / `clean` 72, **읽은 분량** = 보기 토큰의 평균(변경 143개). 판정은 집합 전체로 한다. `table`은 종류별 `defect` 답의 비율도 함께 낸다(판정에 쓰지 않음).

ADR-0078의 H-REVIEW 행 그대로:

> 확정: 탐지율이 같거나 높고(D 안이면 같음), 읽은 분량(토큰)이 1/5 이하. 반증: 탐지율이 D를 넘게 낮음

- 이 집합에서: `spec`의 탐지율이 `code`보다 D를 넘게 낮으면 **반증**, 그렇지 않고 `spec`의 평균 보기 토큰이 `code`의 1/5 이하면 **확정 조건 충족**, 그 밖은 **확정 아님**(탐지율은 D 안, 토큰 비가 기준 초과). D는 ADR-0077 §4의 최소 차이이고, 이 집합의 반복 측정이 없는 동안 전역 값 30%p다(`table --d`로 바꾼다, ADR-0086 §2).
- `table`은 규칙의 값을 늘 계산하지만 판정 줄은 한 모델 실행의 두 팔일 때만 낸다: 팔마다 결과 파일 하나, 같은 집합·MANIFEST·모델·커밋·변경 전체, 미커밋 없음. 그 밖에는 `not judged (사유)`다.
- 모델은 ADR-0066대로 qwen 먼저, sonnet 교차. 가설의 확정은 두 모델 모두 충족일 때다.
- 오탐율은 판정 규칙에 없다. 함께 보고한다(늘 `defect`라고 답하는 리뷰어는 탐지율 100%, 오탐율 100%다).

**평가자는 모델 리뷰어뿐이다.** ADR-0078 §3은 사람 표본을 교정용으로 두었지만 사람 표본 교정은 하지 않는다(사람 결정 2026-11-11, 백로그 `2H-05` 취소). 이 집합의 판정은 모델 리뷰어에 한하며 사람 검토자에게 옮기지 않는다.

## 미리 알려 둘 것

- **토큰 기준(1/5)은 이 집합에서 충족될 수 없다. 본문이 바뀌는 변경만 보아도 같다.** 커밋 `9938d706`의 평균 보기 토큰(`harness.py sizes`):

| 종류 | 변경 | `spec` (계약 diff + `bmb weaken`) | `code` | 비 spec/code | 의도 문장 포함 비 |
| --- | --- | --- | --- | --- | --- |
| 전체 | 143 | 289.8 (106.4 + 154.7) | 165.4 | 1.752 | 1.497 |
| `contract-weakened` | 45 | 364.8 (125.8 + 210.3) | 170.9 | 2.135 | 1.777 |
| `contract-restored` | 45 | 265.3 (125.8 + 111.0) | 170.9 | 1.553 | 1.379 |
| `contract-weakened+body` | 26 | 380.5 (145.7 + 206.1) | 228.3 | 1.667 | 1.474 |
| `body-only` | 27 | 118.3 (4.0 + 85.3) | 86.7 | 1.364 | 1.170 |

  (괄호 안은 두 부분의 토큰이고, 나머지는 제목 줄이다.) 본문 변이는 한 자리 치환이라 코드 diff가 문맥 3줄을 더해 80~230토큰이고, `bmb weaken`의 출력만으로 그보다 크거나 비슷하다. `body-only`에서 계약 diff는 4토큰이지만 도구 출력 85토큰이 붙는다. 본문이 바뀌는 53개의 비는 1.58이다((26 × 380.5 + 27 × 118.3) / (26 × 228.3 + 27 × 86.7)). 그래서 이 집합이 낼 수 있는 판정은 **반증**과 **확정 아님** 둘뿐이고, 재는 것은 탐지율 쪽(명세 diff와 도구 판정만으로 코드 diff만큼 잡는가)이다. 1/5에 닿으려면 코드 diff가 계약 diff의 다섯 배가 넘는 변경, 곧 본문을 여러 줄 고치는 변경이 있어야 하는데, 동결 자료와 한 자리 변이로는 만들 수 없다. 구성 규칙(K, 순서)은 재기 전에 정했고 비에 맞춰 바꾸지 않았다. 기준도 바꾸지 않았다.
- **결함은 모두 계약 약화를 낀다.** `defect` 71개는 "계약만 약해짐" 45와 "계약이 약해지고 본문도 바뀜" 26이다. 계약은 그대로인데 본문만 틀리게 바뀐 변경은 없다: 기준 계약에서 증명되는 변이체는 정의상 계약을 지키므로 `clean`이다(전제 "증명을 통과한 결함은 명세 결함"의 뜻). 처음부터 약한 계약 아래에서 본문이 틀리게 바뀌는 변경(약한 → 약한 + 변이)은 넣지 않았다. 계약 diff가 비어 `spec` 팔이 원리상 잡을 수 없는 종류이고, 가설에 불리한 쪽의 빈칸이다. 뜻이 같은 계약 고쳐 쓰기와 정당한 약화도 없다.
- **`bmb weaken` 판정이 라벨을 거의 갈라 준다.** `weakened`는 `defect`에서만(52/71), `refined`·`compatible`·`same`은 `clean`에서만(70/72) 나오고, `undecided` 21개(`defect` 19, `clean` 2)만 도구로 갈리지 않는다. `spec` 팔의 탐지율은 도구 판정을 읽은 값일 수 있으며, 도구가 가르지 못하는 종류의 변경으로 일반화하지 않는다.
- **`contract-weakened+body`는 과제 17개, `body-only`는 20개에만 있다.** 변이체가 조건을 충족한 과제만이고(과제당 최대 2), 증거 입력은 일반 생성기의 것이라 사전 조건이 좁은 과제(예: 정렬된 배열)는 빠지기 쉽다.
- **짝.** 한 과제의 `weaken`·`restore`는 서로의 역이고, 변이 변경은 같은 기준 프로그램에서 시작한다. 리뷰는 변경마다 독립된 요청이라 리뷰어는 다른 변경을 보지 못한다.
- 과제 편향은 BVB의 것이다(정수·배열, 함수 하나짜리 단위, `bench/bvb/SOURCES.md`). 한 변경이 탐지율의 1.4%p다.
- 의도 문장은 "계약이 무엇을 말해야 하는가"를 직접 적는다(BVB 프롬프트의 문장). 실제 변경의 의도 설명보다 명세에 가깝다.
- `--regen`은 증명과 실행을 모두 다시 하므로 느리고(4코어에서 13~14분) 어떤 변이체가 증명되는지가 컴파일러에 달려 있다. 그래서 푸시 게이트에는 없고, 집합을 다시 만들 때 `set.json`의 `generated_with` 커밋의 컴파일러로 돌린다. 게이트는 `--check`(구조)와 `test_review.py`(저장된 프로그램 143개를 지금 컴파일러로 전부 증명)다.

## 다른 집합을 더하는 법 (H-INTENT, T1-44e)

하네스는 `--set DIR`로 다른 집합을 돈다. 집합은 자료다: 디렉터리에 `set.json`, `prompt.md`, `changes/*.json`, `MANIFEST`를 둔다.

- `set.json`의 `arms`는 팔 이름 → 보기 부분의 목록이다. 부분은 `contract_diff`, `weaken`, `code_diff`, `contract`(뒤 프로그램의 계약 텍스트), `code`(뒤 프로그램 파일), `field:이름`(변경 파일에 동결해 둔 텍스트, 제목은 `titles`)이다. H-INTENT는 `{"code+spec": ["code", "contract"], "spec+examples": ["contract", "field:examples"]}`, `"compare": {"candidate": "spec+examples", "baseline": "code+spec", "token_ratio": [1, 2]}`로 쓰고, 예시·반례는 생성 도구(T1-44c)의 출력을 변경의 `examples`에 넣어 동결한다.
- `frozen`에 그 집합의 생성기와 `../review/views.py`, `../review/judge.py`, `../review/harness.py`를 적으면 MANIFEST가 함께 묶는다(`harness.py --set DIR manifest --write`는 새 집합을 만들 때만).
- 변경 파일의 키: `id`, `intent`, `seed`(집합 디렉터리 기준 씨앗 경로, 없으면 생략), `file`, `before`, `after`, `label`(`defect`·`clean`), 그리고 `field:`로 보일 텍스트.
- 새 부분이 필요하면 `views.py`에 더한다. `views.py`와 `harness.py`는 이 집합의 MANIFEST에 있으므로 고치면 이 집합을 다시 동결해야 하고(정식 결과가 생기기 전에만), 기존 부분의 출력은 바꾸지 않는다.
- 게이트의 `OtherSet` 시험이 이 꼴(H-INTENT의 두 팔, 비 1/2)을 임시 집합으로 돌려 본다.

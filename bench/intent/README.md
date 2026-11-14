# INTENT: 명세 + 생성한 예시 검토 대 코드 + 명세 검토 (H-INTENT, ADR-0078, 백로그 T1-44e)

가설 H-INTENT: 명세에서 생성한 예시·반례를 보여 주면, 의도와 어긋난 명세를 코드와 명세를 함께 읽는 검토보다 잘 잡는다. 항목은 증명된 프로그램 하나와 그 의도 문장이고, 리뷰어는 "이 계약이 의도를 말하는가"를 답한다.

이 디렉터리는 `bench/review` 하네스의 둘째 집합이다(자료: `set.json`, `prompt.md`, `changes/`, `MANIFEST`). 보기·판정·실행은 `bench/review`의 `views.py`·`judge.py`·`harness.py`를 그대로 쓴다. 실제 모델 실행은 아직 없다(측정 머신에서 한다).

| 파일 | 내용 |
| --- | --- |
| `changes/*.json` | 항목 90개(동결된 산출물). 하네스가 집합을 `changes/`에서 읽으므로 이름이 그렇고, 프로그램은 항목의 `after`다(`before`는 없다) |
| `gen.py` | 생성기(동결). `--check`: 컴파일러 없이 확인. `--regen`: 컴파일러로 다시 만들어 바이트 단위 비교. `--stats`: 예시 출력의 라벨별 성질 |
| `set.json` | 팔, 비교하는 두 팔, 토큰 비 기준(1/2), 예시를 만든 컴파일러 커밋, MANIFEST에 더 넣는 파일(동결) |
| `prompt.md` | 리뷰어 시스템 프롬프트(동결). 두 팔이 같은 프롬프트를 쓴다. `bench/review`의 것과 다른 질문이다 |
| `MANIFEST` | 위 동결 파일, `../review/{gen,views,judge,harness}.py`, `../bvb/MANIFEST`의 SHA-256. 다르면 실행을 거부한다 |
| `test_intent.py` | 게이트 |

```
python3 bench/review/harness.py --set bench/intent manifest                 # 동결 확인
python3 bench/intent/gen.py --check                                         # 구조 확인(컴파일러 없음)
python3 bench/intent/gen.py --regen                                         # 예시를 다시 만들어 바이트 단위 비교(컴파일러, 약 1분)
python3 bench/intent/gen.py --stats                                         # 예시 출력의 라벨별 성질(아래 표)
python3 bench/review/harness.py --set bench/intent sizes                    # 팔·라벨별 보기 크기와 토큰 비(모델 없음)
python3 bench/review/harness.py --set bench/intent view --arm spec+examples --change vb-mbpp-62-smallest.weak
python3 bench/review/harness.py --set bench/intent run --arm spec+examples --agent oracle --expect all-right
python3 bench/review/harness.py --set bench/intent run --arm code+spec --agent blind --expect none-detected
python3 bench/review/harness.py --set bench/intent run --arm spec+examples --agent openai:MODEL --out bench/intent/results/DATE-MODEL-examples.jsonl
python3 bench/review/harness.py --set bench/intent run --arm code+spec --agent openai:MODEL --out bench/intent/results/DATE-MODEL-code.jsonl
python3 bench/review/harness.py --set bench/intent table [--d PP] EXAMPLES.jsonl CODE.jsonl   # 팔별 표와 H-INTENT 판정 줄
```

## 항목 집합

BVB 과제 45개(`bench/bvb/tasks`, 동결)는 저마다 의도 문장, `reference`(의도를 말하는 계약), `weak_reference`(일관되고 증명되지만 의도를 말하지 않는 계약)를 갖는다. 과제마다 의도 문장이 같은 항목 둘이다.

| 항목 | 종류(`kind`) | 프로그램 | 숨긴 라벨 | 개수 |
| --- | --- | --- | --- | --- |
| `<과제>.reference` | `reference-contract` | `reference` | `clean` | 45 |
| `<과제>.weak` | `weak-contract` | `weak_reference` | `defect` | 45 |

합계 90개: `defect` 45, `clean` 45.

- 손으로 쓴 것은 없다. 의도와 프로그램 텍스트는 `bench/review/gen.py`가 만드는 그대로다(과제 프롬프트의 설명, 과제의 `put` 코드를 단위 `tasks`에 넣은 파일 하나와 BVB 씨앗). 두 항목은 `bench/review`의 `<과제>.weaken` 변경의 앞과 뒤다(게이트가 확인).
- `examples`는 `bmb examples <과제의 def> -- <씨앗>/src/given.bmb src/tasks.bmb`의 표준 출력 그대로다(기본 개수 3). 도구는 `requires`·`ensures`만 읽고 본문은 읽지 않는다. 대상은 과제의 함수 하나이고, 계약 텍스트에 함께 보이는 보조 함수(세 과제)의 예시는 없다.
- **커밋된 `changes/`가 동결된 산출물이다.** 리뷰어는 커밋된 예시 텍스트를 본다(실행 때 도구를 다시 돌리지 않는다. 보기는 컴파일러 없이 만들어진다). 생성에 쓴 컴파일러 커밋은 `set.json`의 `generated_with`(`8ba498b0`)다. `gen.py --check`는 `examples`를 뺀 전부를 바이트 단위로 다시 만들어 비교하고, `examples`는 꼴(대상 정의의 JSONL 행, 마지막이 요약, 요약의 개수가 행과 일치, 경로 없음)을 본다.
- 라벨의 근거(게이트): 프로그램 90개가 모두 `bmb verify`를 통과하고, m1 판정(ADR-0021 §2: 계약이 과제의 숨은 기준 계약을 함의)이 `clean`을 모두 받아들이고 `defect`를 모두 거부한다.

## 팔 (`set.json`)

| 팔 | ADR-0078 | 보는 것 |
| --- | --- | --- |
| `code+spec` | (a) 코드 + 명세 | 프로그램 파일(`src/tasks.bmb`) 전체: 계약과 본문 |
| `spec+examples` | (b) 명세 + 생성한 예시·반례 | 계약 텍스트(함수의 시그니처 줄, `requires`·`ensures` 줄, 그 줄들이 부르는 단위 안의 `spec fn` 정의 전체: `bench/review/views.py`의 추출)와 동결해 둔 `bmb examples` 출력 |

- 두 팔 모두 의도 문장을 먼저 받는다. 씨앗의 `given::..`은 어느 팔에도 없다(그 코드를 주는 과제는 의도 문장에 들어 있다).
- **예시의 표현(동결): 도구가 찍은 JSONL 그대로다.** 행(`allowed`·`rejected`·`excluded`)과 요약 행을 줄 그대로, 순서 그대로 보인다. 다시 쓰거나 줄이지 않았다(요약의 `rlimit`·`max_len`도 그대로다). 프롬프트가 세 종류의 뜻, 같은 입력의 `allowed` 두 줄의 뜻, 그리고 행이 없거나 `undecided`가 0보다 큰 것은 도구가 예시를 만들지 못했다는 뜻이지 예시가 없다는 뜻이 아님을 적는다.
- `code+spec`은 프로그램 파일 하나만이다. 파일이 계약을 담고 있으므로 계약 텍스트를 따로 한 번 더 붙이지 않았다(붙이면 (a)의 토큰이 늘어 토큰 기준에서 가설에 유리해진다).
- 크기는 보기의 바이트와 `ceil(바이트 / 4)` 토큰이다(ADR-0021 §4). 의도 문장을 더한 메시지 크기도 함께 적는다.

## 판정 (첫 실행 전에 등록, 2026-11-12)

답의 꼴과 판정은 `bench/review`와 같다(`judge.py`). 리뷰어는 항목마다 요청 한 번(도구 없음, 재시도 없음)에 `{"verdict": "defect" | "ok", "reason": "한 줄"}` 하나로 답한다.

- **탐지:** `defect` 항목에 `"defect"`. **오탐:** `clean` 항목에 `"defect"`. **형식 오류:** 그 객체 하나가 아닌 답. 따로 세고 탐지도 오탐도 아니며 분모에는 남는다.
- **탐지율** = 탐지 / 45, **오탐율** = 오탐 / 45, **읽은 분량** = 보기 토큰의 평균(항목 90개).

ADR-0078의 H-INTENT 행 그대로:

> 확정: (b)의 탐지율이 (a) 이상이고(D 안이면 같음으로 본다), (b)가 읽은 토큰이 (a)의 1/2 이하. 반증: (b)의 탐지율이 (a)보다 D를 넘게 낮음

- 이 집합에서: `spec+examples`의 탐지율이 `code+spec`보다 D를 넘게 낮으면 **반증**, 그렇지 않고 `spec+examples`의 평균 보기 토큰이 `code+spec`의 1/2 이하면 **확정 조건 충족**, 그 밖은 **확정 아님**. D는 ADR-0077 §4의 최소 차이이고 이 집합의 반복 측정이 없는 동안 전역 값 30%p다(`table --d`).
- `table`은 규칙의 값을 늘 계산하지만 판정 줄은 한 모델 실행의 두 팔일 때만 낸다(팔마다 결과 파일 하나, 같은 집합·MANIFEST·모델·커밋·항목 전체, 미커밋 없음). 그 밖에는 `not judged (사유)`다.
- 모델은 ADR-0066대로 qwen 먼저, sonnet 교차. 가설의 확정은 두 모델 모두 충족일 때다. 오탐율은 규칙에 없고 함께 보고한다.

**평가자는 모델 리뷰어뿐이다.** ADR-0078 §3은 사람 표본을 교정용으로 두었지만 사람 표본 교정은 하지 않는다(사람 결정 2026-11-11, 백로그 `2H-05` 취소). 이 집합의 판정은 모델 리뷰어에 한하며 사람 검토자에게 옮기지 않는다.

## 미리 알려 둘 것

모델 실행 전에 결정론으로 잰 값이다(커밋 `8ba498b0`의 재동결, ADR-0134; 첫 동결은 `ba563ed4`, 게이트가 수치를 고정한다). 구성과 표현을 먼저 정하고 쟀으며, 비에 맞춰 바꾸지 않았다.

- **토큰 기준(1/2)은 이 집합에서 충족될 수 없다.** 평균 보기 토큰(`harness.py --set bench/intent sizes`):

| 라벨 | 항목 | `code+spec` | `spec+examples` (계약 + 예시) | 비 (b)/(a) | 의도 문장 포함 비 |
| --- | --- | --- | --- | --- | --- |
| 전체 | 90 | 216.1 | 359.0 (85.3 + 241.7) | 1.662 | 1.486 |
| `clean` | 45 | 221.4 | 341.9 (91.7 + 218.3) | 1.544 | 1.403 |
| `defect` | 45 | 210.7 | 376.1 (78.9 + 265.2) | 1.785 | 1.572 |

  (괄호 안은 두 부분의 토큰이고 나머지는 제목 줄이다.) 예시 출력만으로(241.7) 프로그램 전체(216.1)보다 크다. 계약 텍스트만은 프로그램의 0.39배라, 예시가 평균 22토큰(JSONL 한 줄이 못 됨) 아래여야 1/2에 닿는다. 비가 1/2 이하인 항목은 90개 중 0개다. 과제가 함수 하나이고 본문이 짧아서다. 그래서 이 집합이 낼 수 있는 판정은 **반증**과 **확정 아님** 둘뿐이고, 재는 것은 탐지율 쪽이다. 기준은 바꾸지 않았다.
- **도구가 내는 예시가 라벨에 따라 다르다(`gen.py --stats`).**

| 예시 출력의 성질 | `clean` (45) | `defect` (45) |
| --- | --- | --- |
| 행이 하나도 없음 | 0 | 0 |
| `allowed` 행 없음 | 0 | 0 |
| `rejected` 행 없음 | 0 | 2 |
| `excluded` 행 없음 | 16 | 16 |
| `undecided` > 0 | 4 | 3 |
| 요약에 `reason`(질의 없음) | 0 | 0 |
| 한 입력에 `allowed` 결과가 둘 | 5 | 44 |
| `allowed` 행이 있고 입력마다 결과 하나 | 40 | 1 |
| 평균 행 수 | 7.93 | 9.93 |

  - **모든 항목에 `allowed` 행이 있다.** 첫 동결(`ba563ed4`)에서는 `allowed` 행이 없는 항목이 `clean` 10개, `defect` 2개였다(재귀 `spec fn`을 거치는 기준 계약에서 해법기가 모형을 찾지 못함). 그 차이는 "예시가 없으면 `ok`"라는 지름길이었다. 도구가 재귀 정의를 재귀 함수로 쓰고 작은 입력을 써 넣어 묻게 된 뒤(백로그 T1-44g, ADR-0134) 다시 동결했고 그 지름길은 없다. `undecided` > 0인 항목은 4 대 3이고, 모두 행이 있다(한 종류의 행이 더 나오지 않은 것).
  - **"한 입력에 `allowed` 결과가 둘"이 라벨을 거의 가른다.** 그 행이 있으면 `defect`, 없으면 `ok`라고 답하는 규칙이 계약도 의도도 읽지 않고 90개 중 84개(93.3%)를 맞힌다: 탐지 44/45, 오탐 5/45. 약한 계약은 결과를 정하지 않으므로 도구가 둘째 결과를 보이는 것이고(도구가 하려는 일), 프롬프트도 그 뜻을 적는다. 그래서 `spec+examples`의 탐지율은 이 한 가지 신호를 읽은 값일 수 있다. 의도가 결과를 하나로 정하지 않는 과제에서는 틀린다: `clean`인데 둘째 결과가 보이는 5개(`vb-clover-binary-search`, `vb-mbpp-145-max-difference`, `vb-misc-choose-odd`, `vb-misc-findmax`, `vc-vd0465-increasing-run`)와 `defect`인데 입력마다 결과가 하나인 1개(`vb-misc-max-index`). 오탐율을 함께 읽어야 하고, 이 다섯의 오탐을 따로 본다.
  - **분량도 라벨을 따른다.** 예시 행이 `defect` 쪽이 더 많은 과제가 45개 중 39개(같음 4, 적음 2)이고 예시 토큰은 평균 265.2 대 218.3이다. 행의 합계: `clean`은 `allowed` 147·`rejected` 135·`excluded` 75·미결정 8, `defect`는 245·127·75·4.
- **두 항목의 예시 텍스트가 같은 과제는 없다.** 45개 과제 모두 예시가 라벨에 따라 다르다.
- **`code+spec` 팔에도 크기 차이가 있다.** 약한 계약이 짧아 프로그램이 평균 210.7 대 221.4토큰, 계약 텍스트가 78.9 대 91.7토큰이다.
- **결함은 모두 "계약이 덜 말함"이다.** `defect` 45개는 BVB의 `weak_reference`: 의도보다 약한 계약이다. ADR-0078이 적은 "일관되지만 다른 뜻"의 다른 꼴(의도와 다른 것을 강하게 말하는 계약, `requires`가 지나치게 좁은 계약)은 없다. 둘째 `allowed` 결과는 약한 계약에서만 생기는 신호이므로, 그런 결함으로 일반화하지 않는다.
- **짝.** 한 과제의 두 항목은 의도가 같다. 리뷰는 항목마다 독립된 요청이라 리뷰어는 짝을 보지 못한다.
- 의도 문장은 "계약이 무엇을 말해야 하는가"를 직접 적는다(BVB 프롬프트의 문장). 실제 의도 설명보다 명세에 가깝다. 과제 편향은 BVB의 것이다(정수·배열, 함수 하나짜리 단위). 한 항목이 탐지율의 2.2%p다.
- **재생성과 게이트.** 예시는 해법기의 답이지만 고정 시드·`rlimit`이라 같은 컴파일러에서는 같은 바이트다(이 머신에서 다시 만들어 일치, 약 1분). 그래서 게이트(`test_intent.py`)가 지금 컴파일러로 다시 만들어 커밋된 텍스트와 비교한다. 컴파일러가 바뀌어 도구의 출력이 달라지면 그 시험이 실패한다. 그때 커밋된 텍스트는 고치지 않는다(모델이 보는 것이 동결물이다). 정식 결과가 없으면 새 커밋으로 다시 만들어 다시 동결하고, 있으면 `bench/review`의 `--regen`처럼 그 비교를 푸시 게이트에서 빼고 `generated_with` 커밋의 컴파일러로만 돌린다(결정은 ADR로).
- `views.py`·`judge.py`·`harness.py`·`../review/gen.py`는 두 집합의 MANIFEST에 모두 있다. 고치면 두 집합을 다시 동결해야 한다.

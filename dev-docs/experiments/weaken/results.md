# 명세 약화 감지: 이력과 변이체 (백로그 2D-19)

> 2026-10-29. 컴파일러: 2D-19 커밋의 `target/debug/bmb`(디버그 빌드). 도구와 판정의 정의는 `compiler/README.md` "명세 약화 감지".

## 요약

| 측정 | 판정한 정의 | 기대와 같은 판정 | 편집하지 않은 정의 |
| --- | ---: | ---: | --- |
| BCB `contract-heavy`, 정의 200개, 커밋 1,000개 | 1,544 | 1,544 | 227,720행 모두 `same` |
| BCB `body-heavy`, 정의 200개, 커밋 300개 | 461 | 461 | 47,158행 모두 `same` |
| 실제 프로그램 88개의 본문 변이체 233개 | 1,472행 | 1,472 | — |

계약이 바뀐 편집(`contract`·`spec`)으로 판정한 정의는 `contract-heavy`에서 620개: 약화 258(42%), 정제 310(50%), 같음 52(8%). `body-heavy`에서 64개: 약화 28, 정제 34, 같음 2. 본문 편집(`body`) 966개는 모두 `same`이다.

## 해석

- **정답이 있는 이력.** BCB 생성기(`bench/bcb/history.py`)는 편집마다 계약에 무엇을 했는지 안다: 잎의 `ensures`에 `_ok` 술어를 넣으면 정제, 빼면 약화, `_ok`의 상한을 올리면 그 술어를 쓰는 잎의 약화, 내리면 정제, 그 잎이 술어를 쓰지 않으면 같음, 본문 편집은 같음. `bmb weaken`의 판정은 편집한 정의 모두에서 이 정답과 같았고, 편집하지 않은 정의(피호출자가 바뀌어 `id`만 바뀐 것 포함)는 모두 `same`이었다. `spec` 편집은 이름이 그대로이고 본문만 바뀐 `spec fn`이므로, "이름이 아니라 본문으로 비교"가 이 열에서 확인된다.
- **비율은 생성기의 성질이다.** 생성기가 `_ok`를 넣고 빼기를 반씩 고르므로 약화와 정제가 거의 같다. 실제 에이전트가 게이트를 통과하려고 계약을 약하게 하는 빈도는 H-GAME 과제 집합(2A-13)이 잰다.
- **비용.** 커밋당 `bmb weaken` CPU 약 0.6~1.2초(디버그 빌드, 정의 200개, 다른 에이전트의 게이트와 머신을 나눔). 해법기 의무는 커밋당 약 19(`contract-heavy`), 25(`body-heavy`). 질의는 `id`가 바뀐 정의마다 낸다. 피호출자의 본문만 바뀐 호출자도 `id`가 바뀌어(내용 주소) 질의를 낸다. 교환 관계(T1-16)가 이 판정을 쓴다면 호출자용 해시(`caller`, 계약만)가 같은 정의는 질의 없이 넘길 수 있다.
- **찾은 결함.** 변이체 비교의 첫 실행에서 1,472행 중 4행이 `undecided`였고 둘은 정렬 결함이었다: 필드가 같은 두 구조체가 같은 `id`라 다른 경로의 타입과 공유했고(`structs.bmb`의 `geo::Point`·`arrs::Point`, T1-28), trait를 공유하지 않아 trait에 닿는 `spec fn`이 모두 사본이 되어 재귀 정의를 비교하지 못했다(`trait_contracts.bmb`). 이력 측정에서 하나 더: 비교할 옛 정의가 다른 새 정의와 `id`가 같으면 공유되어 역방향 비교가 빠졌다(`body-heavy` 커밋 168의 `m2::reset_14`가 `compatible`). 셋 다 고쳤고 위 표는 고친 뒤의 결과다.

## BCB 이력

생성: `python3 dev-docs/experiments/weaken/run.py --work W --profile contract-heavy --defs 200 --commits 1000`, 같은 명령에 `--profile body-heavy --commits 300`. 이력은 `history.py gen`이 고정 시드로 만든다(커밋하지 않음). 커밋마다 `bmb weaken 직전 직후`.

### `contract-heavy`, 정의 200개, 커밋 1,000개

| 편집 | 기대 | 판정 | 정의 수 |
| --- | --- | --- | ---: |
| add | added | added | 226 |
| body | same | same | 633 |
| contract | refined | refined | 279 |
| contract | weakened | weakened | 202 |
| remove | removed | removed | 65 |
| spec | refined | refined | 31 |
| spec | same | same | 52 |
| spec | weakened | weakened | 56 |

- 편집한 정의: 1,544개 중 기대와 같은 판정 1,544개(한 커밋의 두 편집이 같은 정의를 건드린 것은 `mixed`로 판정하지 않음).
- 편집하지 않은 정의(행 227,720개): `same` 227,720.
- 전체 판정: added 226, refined 311, removed 65, same 228,409, weakened 259.
- 해법기 의무 18,969개, `bmb weaken` CPU 1,159.0초.

### `body-heavy`, 정의 200개, 커밋 300개

| 편집 | 기대 | 판정 | 정의 수 |
| --- | --- | --- | ---: |
| add | added | added | 42 |
| body | same | same | 333 |
| contract | refined | refined | 27 |
| contract | weakened | weakened | 18 |
| remove | removed | removed | 22 |
| spec | refined | refined | 7 |
| spec | same | same | 2 |
| spec | weakened | weakened | 10 |

- 편집한 정의: 461개 중 기대와 같은 판정 461개.
- 편집하지 않은 정의(행 47,158개): `same` 47,158.
- 전체 판정: added 42, refined 35, removed 22, same 47,493, weakened 28.
- 해법기 의무 7,520개, `bmb weaken` CPU 175.0초.

## 실제 프로그램의 본문 변이체

생성: `python3 dev-docs/experiments/weaken/mutants.py tests/run/*.bmb tests/verify/pass/*.bmb`(파일마다 `bmb mutate`가 받는 첫 정의 셋, 각 정의의 가운데 변이체 하나). 변이체는 계약을 바꾸지 않으므로 모든 행이 `same`이어야 한다. 검사를 통과하지 못하는 변이체(타입 범위를 넘는 리터럴) 3개는 뺐다.

```
{"files": 88, "mutants": 233, "mutants_not_checking": 3, "rows": {"same": 1472}, "queries": 350}
```

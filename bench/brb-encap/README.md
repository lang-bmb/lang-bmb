# BRB-ENCAP: 타입 불변식 대 private + 테스트 (H-ENCAP, ADR-0060 결과 6, 백로그 T0-09)

BRB의 새 판이다. 함수 과제 대신 **자료 구조 수정** 과제를 준다. 세션은 이미 있는 자료 구조(불변식을 가진 타입과 그 연산들)에서 시작하고, 에이전트는 연산 하나를 더한다. BMB 팔은 불변식을 타입 불변식(`#[invariant(p)]`, ADR-0060)으로 갖고 증명 게이트가 모든 경계에서 지킨다. Rust 팔은 private 필드, 문서에 적힌 불변식, 시드의 테스트를 갖고 에이전트가 테스트를 더 쓴다. 숨은 판정기가 세션이 끝난 구현에 연산 순서를 돌리고, 연산마다 결과와 **원시 필드의 불변식**을 확인한다.

이 디렉터리는 과제(동결)·판정기·하네스·CI 검사까지다. 실제 모델 실행(파일럿 조건, "통과 & 숨은 불변식 위반" 비율 비교)은 `T0-21`이 한다. 결과는 아직 없다.

## 가설과 반증 조건 (실제 모델 실행 전에 정함)

- **H-ENCAP(ROADMAP 가설 등록부, ADR-0060 결과):** 자료 불변식을 타입 불변식 증명으로 지키면, 에이전트가 자료 구조 내부를 고쳐도 불변식 위반이 바이너리에 들어가지 않는다(0). Rust의 private + 테스트는 0보다 크다.
- **지표:** 비교군마다 "자기 검사 통과 & 숨은 불변식 위반 / 자기 검사 통과". 자기 검사는 세션이 끝난 상태에서 하네스가 한 번 더 돌린 그 비교군의 검사기다(BMB `verify`, Rust `cargo test`, BRB의 `final_check_ok`와 같음). 숨은 불변식 위반은 숨은 스크립트 중 하나라도 어떤 연산 뒤의 원시 상태가 불변식을 깨는 것이다(`violation_cases > 0`).
- **반증 조건:** BMB 팔에서 위반이 게이트를 통과한다. 즉 BMB 세션 하나라도 증명 게이트를 통과했는데 숨은 판정이 위반을 찾으면 반증이다. 에이전트가 불변식 술어를 바꿔(약화해) 통과한 경우도 반증으로 세고, 그 수를 따로 적는다(`invariant_kept`).
- **판정 줄(`summarize`, BMB 결과 하나와 Rust 결과 하나):**

```
H-ENCAP (통과 & 숨은 불변식 위반 비율, 반증: BMB 팔에서 위반이 게이트를 통과): BMB a/p = x, Rust b/q = y → 반증(BMB 위반 a건 중 불변식 변경 k건) | 지지 | 판정 불가(..)
```

  - BMB 위반이 하나라도 있으면 반증.
  - 아니면 Rust 위반이 있으면 지지(BMB 비율 0 < Rust 비율).
  - 둘 다 0이면 판정 불가(Rust 통과 & 위반 0: 이길 대상이 없음).
  - 한쪽에 자기 검사를 통과한 세션이 없으면 판정 불가.

## 파일

| 파일 | 내용 |
| --- | --- |
| `tasks/eNN_*.py` | 과제 4개(동결): 문장·시그니처, 연산 표(`OPS`), 원시 필드(`RAW`), 숨은 판정 자료(불변식 `invariant`, 추상값 `view`, Python 모델 `step`, 스크립트 생성기 `cases`), 기준·약한·게이밍 해법(BMB), 기준·약한 해법과 약한 해법의 테스트(Rust), Rust 변이체 |
| `seed/eNN_*/` | 시작 상태(동결): BMB 단위(`<unit>.bmb`), Rust `src/lib.rs`와 `tests/seed.rs`. 두 팔의 연산은 같은 알고리즘의 쌍둥이다 |
| `prompt-bmb.md` | BMB 비교군 시스템 프롬프트(동결). BRB 판의 도구 절에 데이터 타입·타입 불변식 규칙과 "자료 구조 수정" 작업 방법 |
| `prompt-rust.md` | Rust 비교군 시스템 프롬프트(동결). BRB 판에 private 필드·문서의 불변식·작업 방법 |
| `judge.py` | 숨은 판정기(동결) |
| `harness.py` | 세션 실행·판정·요약(동결). BRB 하네스(`bench/brb/harness.py`)를 라이브러리로 쓴다 |
| `MANIFEST` | 위 파일들과 `../brb/MANIFEST`의 SHA-256(`harness.py manifest --write`가 만든다). BRB 쪽 해시가 이 판이 기대는 BRB 하네스·판정기·도구 서버·Rust 틀을 묶는다. 다르면 하네스가 실행을 거부한다 |
| `test_brb_encap.py` | 과제 집합·채점·판정 줄·판정기의 시험(게이트) |

```
python3 bench/brb-encap/harness.py manifest                                    # 동결 확인
python3 bench/brb-encap/harness.py run --arm bmb  --agent oracle --expect all-pass       # 기준 해법(CI)
python3 bench/brb-encap/harness.py run --arm rust --agent oracle --proptest off --expect all-pass
python3 bench/brb-encap/harness.py run --arm bmb  --agent weak --expect weak-caught      # 약한 해법(CI)
python3 bench/brb-encap/harness.py run --arm rust --agent weak --proptest off --expect weak-caught
python3 bench/brb-encap/harness.py run --arm bmb  --agent game --expect gamed            # 게이밍 대조군(CI)
python3 bench/brb-encap/harness.py adequacy                                    # Rust 변이체 16개가 모두 잡힌다(CI)
python3 bench/brb-encap/harness.py summarize BMB.jsonl RUST.jsonl              # 표와 H-ENCAP 줄
```

## 과제

용량 8의 고정 배열 위 자료 구조 넷. 각 과제는 두 비교군에 같은 문장을 주고, 타입이 있는 곳과 시그니처 줄만 다르다(`judge.task_prompt`, 시험 `test_both_arms_get_the_same_statement`).

| 과제 | 타입(BMB / Rust) | 불변식 | 시드 연산 | 더할 연산 | 약한 해법(두 팔 같은 결함) |
| --- | --- | --- | --- | --- | --- |
| e01-sorted-set | `sset::Set` / `SortedSet` | `len <= 8`, `items[0..len]` 엄격히 증가 | `new` `size` `at` `contains` `insert` `remove` | `replace(from, to) -> bool` | 제자리 덮어쓰기: `to`가 이웃 사이에 맞지 않으면 순서가 깨진다 |
| e02-ring | `ring::Ring` / `Ring` | `head < 8`, `len <= 8` | `new` `push` `pop` `get` `size` | `push_overwrite(x) -> Option<i64>` | 가득 찼을 때 `head + 1`을 감싸지 않는다. `get`은 `% 8`로 읽어 값은 맞아 보이고, `pop`은 `buf[head]`를 직접 읽는다 |
| e03-intervals | `spans::Spans` / `Spans` | `len <= 8`, 구간마다 `lo <= hi`, 이웃 사이에 빈 값 하나 이상(`hi[i] + 1 < lo[i+1]`): 집합마다 목록 하나 | `new` `count` `first` `last` `contains` `push`(끝에 덧붙임) | `remove(x) -> bool`(값 하나 빼기: 구간 삭제·축소·분할, 분할은 빈 자리가 있을 때만) | 끝점 검사를 한 값 구간 검사보다 먼저: 한 값 구간에서 빼면 빈 구간(`lo > hi`)이 남는다 |
| e04-min-heap | `heap::Heap` / `Heap` | `len <= 8`, `a[(i-1)/2] <= a[i]` | `new` `size` `peek` `push`(올려 거르기) | `pop() -> Option<i64>` | 내려 거르기에서 왼쪽 자식만 본다 |

- **기준 해법.** BMB e01·e02는 시드 연산 호출이나 몇 줄이고, e03·e04는 시드의 정렬 집합처럼 배열 사본 위에서 옮기며 양화 루프 불변식으로 증명한다(e03 `remove` rlimit 약 0.7M, e04 `pop` 약 1.3M). Rust 기준 해법은 같은 알고리즘이다.
- **BMB 약한 해법:** Rust 약한 해법과 같은 결함. 게이트가 타입 불변식 경계에서 거부한다(E0823, e01은 해법기가 결정하지 못해 E0811; e03은 넘침 E0803도). 판정기는 증명되지 않은 빌드도 돌려 위반을 보인다.
- **BMB 게이밍 대조군(`BMB_GAME`):** 불변식 술어를 시드 연산의 안전에 필요한 만큼(`len <= 8` 등)으로 약화하고 결함 있는 연산을 넣는다. 게이트를 통과하고, 판정기는 `invariant_kept: false`와 위반을 낸다. e02는 없다: `pop`이 `buf[head]`를 직접 읽어 `head < 8`을 뺀 어떤 술어도 시드 연산의 증명을 깨뜨린다.
- **Rust 약한 해법:** 결함 있는 메서드와 그것을 통과하는 그럴듯한 테스트(`tests/<연산>.rs`). 시드 테스트와 함께 `cargo test`가 통과한다.
- **Rust 변이체:** 과제마다 4개(약한 해법 포함), 합계 16개. 숨은 스크립트가 모두 잡아야 한다(`adequacy`). 위반으로 잡히는 것과 결과 차이로 잡히는 것이 섞여 있다.

## 숨은 판정 (`judge.py`)

에이전트는 스크립트·모델·불변식 검사를 보지 못한다(프롬프트에는 "숨은 판정기가 연산마다 결과와 필드의 불변식을 본다"는 말만 두 팔 같게 들어간다).

- **스크립트:** 과제당 300~400개(합계 1,500개, 연산 29,856개), 길이 1~47 연산, 빈 구조(`new()`)에서 시작. 과제 id로 시드한 결정론적 생성(`bench/brb/cases.py`의 `Gen`). 연산의 31~38%가 더할 연산이고, 가득 찬 구조와 `i64` 끝값(`MIN`·`MAX`)에서의 호출을 담는다(시험 `test_scripts_reach_the_hard_states`).
- **기록:** 연산마다 결과(두 수: `bool`은 0/1, `Option`은 있음·값)와 원시 필드.
- **채점(`score_case`):** 연산 순서대로 (1) 원시 상태가 불변식을 깨면 `violation`, (2) 결과가 모델과 다르거나 불변식이 지켜진 상태의 추상값(`view`: 정렬 집합·구간·고리의 값 순서, 힙의 다중집합)이 모델과 다르면 `wrong`, (3) 패닉·트랩·충돌·시간 초과. 스크립트의 종류는 첫 실패다. `violation_cases`는 앞에서 결과가 틀렸더라도 기록된 어느 상태든 불변식을 깨는 스크립트 수다.
- **BMB:** (1) 게이트 `verify`(통과해야 제출), (2) 불변식 유지: 타입의 `id`가 시드와 같은가(`id`는 필드와 불변식 술어의 본문을 담고 술어 이름은 담지 않는다, ADR-0059·0060; 시험 `test_renaming_the_predicate_keeps_the_invariant`), (3) 숨은 단위 `brb_encap_judge::run(ops, out)`이 스크립트를 타입의 연산과 대상 연산으로 돌리고 필드를 복사한다. 불변식을 가정하지 않는다(필드를 읽기만). 작업 공간과 함께 검사되지 않으면 대상이 없거나 시그니처가 다른 것(미제출), 증명되지 않으면 정의역 위반(`domain_ok: false`: 대상이 어떤 상태·인자에서 불릴 수 없음), (4) `emit-llvm` + LLVM 진입점 + C 드라이버. 드라이버는 스크립트마다 자식 프로세스에서 앞부분(연산 1, 2, ..개)을 다시 돌려 기록을 하나씩 보내므로, 트랩·충돌 앞의 상태도 판정한다. 증명된 코드에는 런타임 검사가 없다.
- **Rust:** 에이전트 크레이트를 복사해 `src/lib.rs` 끝에 숨은 자식 모듈 `__brb_encap_probe`(크레이트 루트의 private 필드를 읽음)를 붙이고, BRB 판정 크레이트와 같은 설정(디버그 빌드, 넘침 검사)으로 빌드한다. 타입을 옮기거나 필드를 바꾸면 빌드되지 않아 미제출이다(시험 `test_changed_fields_are_not_delivered`). 스크립트마다 `catch_unwind`, 패닉 앞의 기록도 판정한다.
- **성공:** 제출(BMB는 증명됨), 모든 스크립트 통과, 정의역 위반 없음.

## 비교군과 공정성 규칙

BRB와 같다(ADR-0021, ADR-0034 §2): 같은 어댑터·같은 모델 설정, 세션당 도구 호출 40회·모델 출력 토큰 64,000, 두 팔 모두 검사기(BMB `verify`, Rust `cargo_build`·`cargo_test`, proptest), 같은 숨은 판정, 과제당 1회. 세션 루프·Rust 샌드박스·도구 서버·최종 자기 검사는 `bench/brb/harness.py`의 것을 그대로 쓴다. 복사한 세션 루프는 같은 코드에 두 가지만 다르다: 시드를 쓰고, Rust 세션마다 미리 빌드한 대상 디렉터리의 사본을 쓴다. 모든 세션의 크레이트 이름이 `solution`이라, 대상 디렉터리를 함께 쓰는 세션이 동시에 돌면 통합 테스트(`tests/*.rs`, 시드에 있음)가 다른 세션의 라이브러리를 링크한다(재현: `--jobs 2`에서 `use solution::Ring`이 정렬 집합 크레이트를 봄). BRB 하네스의 같은 위험은 `T0-22`. 결과 행은 세션마다 쓰고(INF-16), `--sessions DIR`로 세션 기록(INF-17)을 남긴다. 세션 기록의 작업 공간은 시드와 다른 파일이다.

## CI (`scripts/check.sh`)

| 작업 | 확인 |
| --- | --- |
| `bench brb-encap oracle` | 두 팔의 기준 해법이 숨은 판정과 자기 검사를 통과(4/4), 위반 0, H-ENCAP 줄 "판정 불가(Rust 통과 & 위반 0)" |
| `bench brb-encap weak` | Rust 약한 해법: 자기 테스트 통과 & 숨은 위반 4/4. BMB 약한 해법: 게이트가 4/4를 불변식 경계에서 거부하고, 증명되지 않은 빌드의 실행은 위반을 보인다. BMB 게이밍 대조군: 게이트 통과, 불변식 변경, 위반 3/3 |
| `bench brb-encap adequacy` | Rust 변이체 16개가 모두 잡힌다 |
| `bench brb-encap tests` | `test_brb_encap.py`: 동결 목록, 같은 문장, 스크립트 결정론·범위·어려운 상태, 채점(위반·틀림·패닉), H-ENCAP 줄과 요약 표, `--expect` 조건, 시드 증명, 정의역 위반, 다른 시그니처, 술어 이름 바꾸기, Rust 필드 변경 |

컴파일러 시험 `cargo test --test verify proofs_are_not_vacuous`는 BMB 시드 넷의 증명이 공허하지 않음(어떤 도달 지점도 자기 증명에 반박되지 않음)을 확인한다.

CI는 Rust 팔을 `--proptest off`로 돈다(대본 에이전트는 proptest를 쓰지 않고, 게이트는 네트워크에 기대지 않는다).

## 편향과 한계 (공개)

- **BMB의 새 연산은 자유 함수다.** 트랜잭션(`tx_put`, `write`)은 `fn`·`spec fn`만 쓴다. 타입과 `impl` 블록은 쓸 수 없고, 함수와 섞어 보내면 `impl` 블록이 말없이 버려진다(재현: `impl Set { fn two(..) }`와 함수 하나를 한 `put`으로 보내면 커밋은 함수만 바꾸고 성공으로 보고한다). 그래서 BMB 시그니처는 `fn replace(s: &mut Set, ..)`(단위의 함수)이고 Rust는 메서드다. 프롬프트가 이 제약을 말한다. 도구 층의 발견으로 `T0-20`에 올렸다. 같은 이유로 BMB 에이전트는 시드의 `impl` 연산과 타입을 바꿀 수 없다(바꿀 수 있는 것은 함수와 `spec fn`, 불변식 술어 포함).
- **Rust의 private는 에이전트를 막지 않는다.** 에이전트는 타입의 모듈 안(크레이트 루트)에서 쓰므로 필드를 직접 고칠 수 있다. 이것이 비교 대상이다: 내부를 고칠 때 불변식을 무엇이 지키는가(테스트 대 증명). private가 막는 바깥 코드는 이 과제에 없다.
- **불변식이 보이는 방식이 다르다.** BMB는 술어 코드로, Rust는 문서 주석으로 본다. BMB 시드 파일의 주석은 질의·맵에 나오지 않으므로(맵은 주석을 담지 않는다) BMB 에이전트에게는 보이지 않는다. 두 팔 모두 "숨은 판정이 연산마다 필드의 불변식을 본다"는 같은 말을 받는다.
- **증명 난이도.** e03·e04의 BMB 해법은 양화 루프 불변식이 필요하다. BMB 세션이 증명에 실패하면 "미제출"이고 위반 분자에 들어가지 않는다. 그래서 BMB의 위반 비율은 자기 검사를 통과한 세션만으로 계산되고, 통과 수가 작으면 비율이 낮게 나오기 쉽다. 보고는 비율과 함께 성공·자기 검사 통과 수를 적는다(요약 표에 있음).
- **증명되지 않은 BMB 빌드의 실행은 정보용이다.** 증명된 연산에는 런타임 검사가 없어서, 깨진 상태를 받은 연산이 정의되지 않은 동작(배열 끝 밖 읽기)으로 트랩할 수 있다. 판정기는 트랩 앞의 상태까지 판정한다. 이런 세션은 미제출이라 지표에 들어가지 않는다.
- **과제 범위.** 용량 8의 고정 배열 넷, 정수 값만. 실제 자료 구조(`Vec`·재귀 타입·제네릭)가 아니다. 숨은 스크립트는 짧다(최대 50 연산).
- **측정 경로.** `bench/run_models.py`·`bench/condition.py`의 집합 목록에 넣지 않았다. 실행(`T0-21`)과 함께 정한다.
- 동결 뒤 과제·프롬프트를 바꾸려면 새 집합 이름으로 만든다(ADR-0021 §1).

# BCB: BMB Cumulative-cost Bench (ADR-0040 G7·H-AMORT, ADR-0042, 백로그 2A-14)

같은 커밋 이력을 BMB 작업 공간과 Rust 쌍둥이에 재생해, 커밋마다 검증에 드는 CPU를 누적한다. BMB는 증분 재증명(계약 방화벽, ADR-0032)과 남은 테스트, Rust는 테스트 실행(전부·선택·캐시)이다. 설계와 사전 등록한 반증 조건은 ADR-0042에 있다.

| 파일 | 내용 |
| --- | --- |
| `history.py` | 이력 생성·재생. `bench/m3/gen.py`의 계획을 편집해 두 렌더러(BMB `gen.py`, Rust `bench/bnb/twin.py`)로 같은 편집을 쓴다. 커밋마다 트리 다이제스트를 기록하고 재생이 확인한다 |
| `rtests.py` | Rust 테스트: 실행 정의마다 `#[test]` 하나(단위 파일 끝의 `mod tests`). `requires`를 만족하는 고정 입력(경계값 + 이름 시드 무작위)으로 부르고, 판정은 쌍둥이의 계약 `debug_assert!`와 넘침·범위 패닉뿐이다. 오라클용 큰 입력 집합(`strong_cases`)은 쌍둥이에 들어가지 않는다 |
| `rustify.py` | BMB 실행 정의 → 쌍둥이 형식 Rust 변환기(생성 부분집합). 원본을 변환하면 쌍둥이와 바이트 단위로 같음을 매번 확인한다. 변이체를 두 언어의 같은 자리에 적용하는 데 쓴다 |
| `kills.py` | 판별률: W0904 변이체(`bmb mutate`)를 두 언어에 적용해 BMB 증명·Rust 테스트·오라클 테스트의 판정을 비교 |
| `bcb.py` | 재생 하네스(`run`)와 표(`table`). 팔 `rust`(한 번의 재생에서 R0·R1·R2: 빌드는 같고 테스트 선택만 다르다), `b1`(유지 캐시), `b0`(표본 커밋마다 새 캐시). 커밋마다 명령별 CPU(user+sys, `wait4`)·벽시계, 선택한 테스트 수, R2가 덮는 정의 수, B1이 다시 증명한 정의(`build --stats`)를 JSONL로 쓴다. 사건 커밋은 캐시를 지우고 한 번 더 잰다(`"cold": true`) |
| `test_bcb.py` | 게이트: 이력 결정성, 다이제스트 재현, 모든 커밋에서 BMB 전부 증명·Rust `cargo check --tests`(경고 = 오류), 편집 종류별 성질(body 편집은 시그니처·계약 불변), `contract-heavy` 이력 끝에서 판별 동등성 |

```
python3 bench/bcb/bcb.py run H.jsonl --work W --out R.jsonl [--b0-every 50]   # 측정(팔 rust,b1,b0)
python3 bench/bcb/bcb.py table R.jsonl                                     # 누적 CPU 표와 비율
python3 bench/bcb/kills.py --defs 1000                                     # 판별 동등성(초기 트리)
python3 bench/bcb/kills.py --history H.jsonl --at 500                      # 이력의 커밋 500에서
python3 bench/bcb/history.py gen H.jsonl --defs 1000 --commits 1000 --profile body-heavy
python3 bench/bcb/history.py stats H.jsonl
python3 bench/bcb/history.py replay H.jsonl --bmb W/bmb --rust W/rust --upto 500
```

## 편집

| 종류 | BMB | Rust 쌍둥이 |
| --- | --- | --- |
| `body` | 본문만(상수·템플릿·피호출자·결과 형태). `#[uses]`는 피호출자 단위에 따라 바뀔 수 있다 | 같은 본문 |
| `contract` | 잎의 `ensures`에 `_ok` 술어를 넣거나 뺀다(처음이면 술어 추가) | 문서 줄과 `debug_assert!` |
| `spec` | `_ok` 술어의 상한 | 술어 함수 본문 |
| `add` / `remove` | 잎 추가 / 호출자 없는 잎 삭제 | 같음 |

프로필(`history.py`의 `PROFILES`): `body-heavy`, `contract-heavy`, `body-only`. 툴체인 사건 커밋은 머리 줄의 `events`(K·j/5)이고 재생 하네스가 적용한다.

## 팔의 실제 명령

| 팔 | 커밋마다 |
| --- | --- |
| R0·R1·R2 | `cargo build`, `cargo test --no-run`(공통), 테스트 실행 파일을 `--exact 이름..`으로(R0는 전부). R1은 바뀐 단위와 그것에 (전이적으로) 의존하는 단위의 테스트, R2는 호출·계약 검사로 바뀐 정의에 닿는 정의의 테스트와 바뀐 테스트 |
| B1 | `bmb build --cache C --stats S src tests driver -o exe`(증명 게이트와 빌드가 한 명령), `exe` 실행과 출력 비교. 남은 테스트는 `show` 정의마다 `tests/tU.bmb`의 테스트 함수(Rust 테스트와 같은 입력)이고, `driver/bcb_main.bmb`의 `main`이 R2 규칙으로 고른 것을 부른다. 기대 출력은 계획에서 계산한다 |
| B0 | 같은 명령을 새 캐시로, 남은 테스트 전부. `--b0-every S` 커밋마다(마지막 커밋 포함) |

## 테스트 동등성(ADR-0042 §4)

변이체 하나는 BMB에서 정의가 더는 증명되지 않으면, Rust에서 그 정의의 테스트가 실패하면 죽는다. 오라클(입력 수백 개)이 계약 위반 입력을 찾으면 그 변이체는 계약을 실제로 어긴다.

| 범주 | 뜻 |
| --- | --- |
| `contract` | BMB가 거부하고 오라클도 위반을 찾음. **동등성은 이 집합에서 잰다**: Rust 테스트가 모두 죽여야 한다(`gaps` = 0) |
| `annotation_only` | BMB가 거부하지만 계약은 지켜짐. 루프 시작 1, 보폭 2 같은 변이가 불변식 `k <= a.len()`을 깨는 경우다(E0806·E0807). 계약만 보는 테스트로는 원리상 구별할 수 없다. BMB가 더 엄격한 몫으로 따로 보고한다 |
| `unsound` | 오라클이 위반을 찾았는데 BMB가 증명함. 0이어야 한다(건전성 교차 확인) |
| `extra` | Rust 테스트가 죽였는데 BMB가 증명함. 0이어야 한다 |

측정(2026-10-25, release): N=100 초기 트리 변이체 338개 중 BMB 거부 195개(계약 165, 주석만 30), Rust 165개, 누락·불건전·초과 0. 커밋 200개 뒤 `body-heavy` 424개 중 계약 179·Rust 179, `contract-heavy` 566개 중 계약 209·Rust 209, 모두 누락 0. N=1,000 초기 트리 3,351개 중 계약 1,571·Rust 1,571, 주석만 300, 누락·불건전·초과 0.

## 진행

- 2A-14a: 설계(ADR-0042), 이력 생성기, 게이트, `bmb verify --stats`(재증명 집합).
- 2A-14b: Rust 테스트와 판별 동등성(`rtests.py`, `rustify.py`, `kills.py`).
- 2A-14c: 재생 하네스(`bcb.py`), `build --stats`.
- 다음: 결함 심기(2A-14d), 동결·측정(2A-14e).

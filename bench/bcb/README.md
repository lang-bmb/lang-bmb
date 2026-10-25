# BCB: BMB Cumulative-cost Bench (ADR-0040 G7·H-AMORT, ADR-0042, 백로그 2A-14)

같은 커밋 이력을 BMB 작업 공간과 Rust 쌍둥이에 재생해, 커밋마다 검증에 드는 CPU를 누적한다. BMB는 증분 재증명(계약 방화벽, ADR-0032)과 남은 테스트, Rust는 테스트 실행(전부·선택·캐시)이다. 설계와 사전 등록한 반증 조건은 ADR-0042에 있다.

| 파일 | 내용 |
| --- | --- |
| `history.py` | 이력 생성·재생. `bench/m3/gen.py`의 계획을 편집해 두 렌더러(BMB `gen.py`, Rust `bench/bnb/twin.py`)로 같은 편집을 쓴다. 커밋마다 트리 다이제스트를 기록하고 재생이 확인한다 |
| `test_bcb.py` | 게이트: 이력 결정성, 다이제스트 재현, 모든 커밋에서 BMB 전부 증명·Rust `cargo check`(경고 = 오류), 편집 종류별 성질(body 편집은 시그니처·계약 불변) |

```
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

## 진행

- 2A-14a: 설계(ADR-0042), 이력 생성기, 게이트, `bmb verify --stats`(재증명 집합).
- 다음: Rust 테스트 동등화(2A-14b), 다섯 팔 재생 하네스(2A-14c), 결함 심기(2A-14d), 동결·측정(2A-14e).

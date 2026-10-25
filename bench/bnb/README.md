# BNB: BMB Navigation Bench (ADR-0034 G4, 백로그 3A-05)

같은 코드 구조에 같은 탐색 질문을 두 팔로 풀게 해 토큰과 정확도를 비교한다.

| 팔 | 코드 | 도구 |
| --- | --- | --- |
| `bmb` | `bench/m3/gen.py`가 만든 작업 공간 | `bmb mcp`의 기존 도구(`query`, `map` 등) + `answer` |
| `rust` | `twin.py`가 같은 계획으로 만든 Rust 쌍둥이 크레이트 | `list_dir`, `read_file`(줄 범위), `grep`(정규식, 경로 여럿, 앞뒤 문맥 줄) + `answer` |

두 팔은 에이전트 인터페이스(`next_calls`, `set_tools`; `bench/m1/harness.py`와 같음)와 예산 규칙(도구 호출 40회, `answer` 포함)이 같다. 비용은 모든 도구 응답 바이트이고 토큰은 ceil(bytes/4)(ADR-0021)다. 요청 바이트(도구 인자)는 따로 기록한다. 시스템 프롬프트·도구 설명 같은 세션 고정비는 세지 않는다.

| 파일 | 내용 |
| --- | --- |
| `twin.py` | Rust 쌍둥이 생성기. `gen.Unit`(이름·종류·상수)과 `gen.choose`(피호출자·결과 형태)를 BMB 생성과 함께 쓰므로 정의·호출 그래프·`&mut` 쓰기 경로·io 함수가 구성상 같다 |
| `analyze.py` | 쌍둥이 원문만 읽는 Rust 쪽 분석기(이 생성기의 고정 배치만 다룸). 호출·쓰기 간선, 계약 술어, io, 중복 묶음을 다시 만든다 |
| `bnb.py` | 과제 생성, 정답 계산, 두 팔의 도구, 스크립트 오라클, 세션 실행, 표 |
| `tasks/` | 과제 27개(동결). `bnb.py tasks`가 계획에서 역할을 골라 만든다 |
| `prompt-bmb.md`, `prompt-rust.md` | 팔마다의 시스템 프롬프트(동결, 실제 모델용) |
| `MANIFEST` | 동결 파일의 SHA-256 |
| `test_bnb.py` | 게이트: 동결 확인, 쌍둥이 결정성·국소성, 파일 도구, N=100에서 `cargo check`·정답 일치·두 오라클 전부 성공 |
| `results/` | 측정 기록 |

```
python3 bench/bnb/bnb.py manifest                                   # 동결 확인
python3 bench/bnb/bnb.py run --sizes 100 --check-truth --cargo-check --expect all-pass   # CI와 같은 점검
BMB=target/release/bmb python3 bench/bnb/bnb.py run --sizes 100 1000 10000 \
    --check-truth --cargo-check --work W --out R.jsonl                # 측정
python3 bench/bnb/bnb.py table R.jsonl                              # 표
python3 bench/bnb/bnb.py run --agent anthropic:MODEL --arms bmb rust --sizes 10000 --out R.jsonl   # 실제 모델(유료, 사용자 머신)
```

## BMB → Rust 대응

| BMB | Rust 쌍둥이 |
| --- | --- |
| 단위 `mod mU { .. }`, `#[summary("..")]` | 파일 `src/mU.rs`(모듈 문서 `//! ..`), `lib.rs`에 `pub mod mU;` |
| 호출 `mV::f(..)` / 같은 단위 `f(..)` | 같은 표기. 다른 단위는 `use crate::{mV, ..};`로 모듈만 들여온다 |
| `fn`, `#[effects(pure\|io)]` | `pub fn`(효과 선언 없음) |
| `spec fn P` | `pub fn P(..) -> bool`, 문서 줄 `/// Predicate (specification only).` 슬라이스 `forall`은 `.iter().all(..)` |
| `requires P(..)` | 문서 줄 `/// Requires: \`P(..)\`.`와 진입 시 `debug_assert!` |
| `ensures Q(..)`, `-> (r: T)` | 문서 줄 `/// Ensures: \`Q(..)\`.`, 결과를 `let r = ..;`로 묶고 반환 전 `debug_assert!` |
| `invariant`, `decreases` | 루프 위 주석 `// invariant: ..`(`decreases`는 버림) |
| `#[uses(..)]` | 없음(Rust에는 불투명 술어가 없다) |
| `print_i64(e)` | `println!("{}", e)` |
| `0 - c`, `-> ()` | `-c`, 반환 타입 생략 |

`cargo check`(경고를 오류로)를 통과한다. 크레이트 속성 `#![allow(unused_variables)]`은 BMB 매개변수 목록을 그대로 둔 술어 때문이다.

## 과제와 정답

단위 `m1`·`m2`마다 12개(직접·전이 호출자, 전이 피호출자, 계약 변경 영향(자신 + 직접 호출자), 사전·사후 조건에 술어를 쓰는 함수, `&mut` 인자를 넘기는 함수, `&mut` 인자가 전이적으로 닿는 함수, 잎을 전이적으로 부르는 io 함수)와 전체 3개(io에 닿는 함수, 중복 2개). 질문 문장은 두 팔이 같고, 언어별 규약은 팔의 프롬프트에 있다.

정답은 BMB 맵에서 계산한다(`bmb query`; 하네스 정답 종류 `kinds`·`reasons`·`list`와 `closure`를 재사용하고 `effects`·`duplicates`만 따로). `--check-truth`는 같은 정답을 `analyze.py`로 Rust 원문에서 다시 계산해 과제마다 비교하고, 구조 전체(실행 호출·`&mut` 쓰기 간선, 계약 간선(루프 불변식 포함), io 집합, 중복 묶음)도 비교한다. 차이가 하나라도 있으면 실행이 실패한다.

과제 역할은 N=1,000 계획 그래프에서 고르고 파일로 동결한다. `m1`·`m2`의 이웃은 N ≥ 340부터 모든 크기에서 같고(gen.py 국소성), N=100에서는 뒤 단위가 없어 일부 정답이 작다. 전체 과제 3개의 정답은 N에 비례한다.

## 오라클

오라클은 팔마다 그 도구로 찾은 가장 싼 정확한 전략이다. 그래서 오라클의 토큰은 팔의 비용 하한이다(실제 에이전트는 탐색·확인에 더 쓴다).

- `bmb`: `edges DEF in|out 64`(한 번에 전이 폐포), `edges DEF in 1`, `impact DEF iface`, `contracts PRED`, `effects`, `duplicates`. `&mut` 폐포는 단계마다 `edges X out 1`(깊은 `edges`는 쓰기 아래의 호출·계약 간선까지 준다).
- `rust`: 첫 파일에서 알 수 있는 배치 규약을 쓴다(모듈 = 파일, 항목은 0열 `pub fn`으로 시작해 0열 `}`로 끝남, 모듈 단위 `use`). 호출자는 단계마다 크레이트 전체 `grep 'mU::f\('`와 자기 파일의 맨이름 `grep`, 그리고 맞은 파일의 `^pub fn ` 머리 줄 `grep`(파일마다 한 번, 기억)으로 둘러싼 함수를 찾는다. 피호출자는 함수마다 `read_file` 줄 범위. 계약은 `/// Requires:`·`/// Ensures:` 줄 `grep`. io는 `grep println!`(앞 문맥 3줄) 뒤 호출자 폐포. 중복은 대상 본문 첫 줄을 앞뒤 문맥과 함께 `grep`해 정규화 비교.

# 포팅 쌍둥이 하네스 (ADR-0104, 백로그 3P-01)

같은 기능의 BMB 프로그램과 Rust 프로그램(쌍둥이)에 같은 과제를 같은 모델·같은 호출 한도(60회)로 주고, 세션이 끝난 뒤 숨은 판정으로 잰다. 주 측정 장은 stage0과 그 BMB 포트이고(포팅 P3 통과 뒤, 3P-05), 지금 있는 `fixtures/stk`는 하네스와 판정기를 게이트에 거는 작은 고정 장치다. 과제·프롬프트는 3P-02~04에서 측정 집합을 만들 때 동결한다(아직 MANIFEST 없음).

| 팔 | 코드 | 도구 |
| --- | --- | --- |
| `bmb` | `bmb import`한 작업 공간 | `bmb mcp --write-tool`(정식 쓰기 표면 ADR-0101, `tx_*` 숨김): `query`, `map`, `verify`, `build`, `write` |
| `rust` | 크레이트 사본 | `list_dir`, `read_file`(줄 범위), `grep`, `write_file`, `edit_file`(정확히 한 곳 바꾸기), `cargo_build`, `cargo_test` |

이해 과제는 두 팔 모두 `answer`(정의 경로 `module::name`)로 끝난다.

## 판정

| 과제 종류 | 성공 |
| --- | --- |
| `comprehend` | 답한 정의 집합이 `truth`와 같다(정밀도·재현율 1) |
| `change` | 세 가지가 모두 참이다. (1) 빌드: BMB는 `bmb build`(모든 의무 증명), Rust는 `cargo build`와 `cargo test`. (2) 말뭉치: 고정 장치의 `corpus/NAME.in`마다 `NAME.out`의 바이트를 그대로 출력하고, 과제의 `cases`(새 기능의 숨은 시험)도 그렇다. (3) 범위: 바뀌거나 생기거나 없어진 정의가 과제의 `scope` 안이다(`new`가 참이면 새 정의는 허용) |

범위는 정의별 원문으로 가린다. BMB는 작업 공간의 정규형 원문(ADR-0019)을 정의 단위로 자르므로 정확하다. Rust는 파일의 최상위 항목(`fn`·`const`·`struct` 등, 문서 줄 포함)으로 자르고, 그 밖의 줄(`use`, `mod`, 매크로)은 `module::<rest>` 하나로 센다(근사).

정체성 해시(`bmb hash`, ADR-0059)를 쓰지 않는 이유: 정체성은 피호출자를 그 정체성으로 담는다. 그래서 피호출자의 본문을 바꾸면 모든 호출자의 정체성이 바뀐다(3P-01 첫 실행에서 `vm::neg`를 더하고 `main::step`을 고치자 `main::run_line`·`main::main`이 바뀐 것으로 나왔다). 그것은 호출자를 편집한 것이 아니다.

## 공정성: 문서 조건 (ADR-0106)

- **주석.** `bmb import`가 만드는 작업 공간은 정규형이라 `//`·`///` 주석을 버린다(단위의 `#[summary]`와 계약은 남는다). 그래서 Rust 팔도 주석을 지운 크레이트를 받는다(`strip_comments`: `//`·`///`·`/* */`를 지우고, `#[summary]`에 대응하는 `//!`는 남긴다). 대조 조건 `PAIR_RUST_COMMENTS=keep`은 주석을 남긴다. 같은 과제를 sonnet으로 한 번 돌리며 Rust 팔만 다시 돈다. 결과 행의 `rust_comments`가 조건을 적는다. 대조 조건에서 차이가 D 안으로 줄면 주장을 "주석 없는 Rust 대비"로 한정한다.
- **범위.** Rust 정의 원문은 주석을 지운 뒤 비교한다(주석만 고친 정의는 바뀐 것이 아니다).
- **게이트.** `check`가 두 조건의 크레이트를 모두 빌드해 말뭉치와 대조한다.

## 고정 장치 `stk`

표준 입력의 명령을 한 스택에서 실행하는 프로그램이다(`lex`·`vm`·`main` 세 단위, Rust는 같은 이름의 모듈 셋). 오류는 `error LINE: WHAT`, 끝에 `depth N`. BMB 쪽은 전부 증명된다. 말뭉치 8개(넘침, `i64::MIN`, 스택 한도 4096, 공백·CR, UTF-8이 아닌 바이트, 빈 입력)에서 두 쪽의 출력 바이트가 같다(`check`가 매번 확인).

| 과제 | 종류 | 약한 대조군(실패해야 하는 검사) |
| --- | --- | --- |
| `c01-callers-is` | `lex::is`의 전이 호출자 | 직접 호출자만(answer) |
| `r01-rename-drop` | `vm::drop`을 `vm::discard`로 이름 바꾸기 | 빈 스택에서 오류를 내지 않음(corpus), 범위 밖 `vm::swap` 편집(scope) |
| `t01-neg` | 새 명령 `neg`(커밋 재연 꼴: 문제문 + 숨은 시험) | 넘침을 감싸 버림(corpus) |

첫 게이트 실행에서 약한 대조군 "빈 스택의 `drop`이 오류를 내지 않음"이 말뭉치를 통과했다. 말뭉치에 그 경우가 없었기 때문이고, `errors.in`에 더했다. 측정 과제의 숨은 판정도 같은 방식(약한 대조군이 떨어지는지)으로 적정성을 확인한다.

```
python3 bench/twin/pair.py check                                   # 게이트(test_pair.py가 돌림)
python3 bench/twin/pair.py run --agent openai:qwen3.8-27b --out R.jsonl   # 실제 모델(ADR-0066)
```

비용은 BNB와 같다. 도구 응답 바이트 전부, 토큰 = ceil(bytes/4)(ADR-0021). 요청 바이트는 따로 적는다.

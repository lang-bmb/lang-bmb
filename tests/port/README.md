# 포트의 차등 시험 사례

`scripts/difftest.py run tests/port/cases.json`이 읽는다(백로그 4D-15, 포팅 지도 `dev-docs/measurements/selfhost-port-map.md` §3.2). 사례마다 stage0 쪽(`a`)과 포트 쪽(`b`)을 같은 입력으로 돌려 stdout·stderr·종료 코드·남긴 파일을 바이트로 견준다. 게이트 작업 "port difftest (4D-15)"가 매번 돌린다.

| 파일 | 내용 |
| --- | --- |
| `cases.json` | 사례와 빌드 목록 |
| `sha256_files.bmb` | 인자의 파일마다 `std::sha256` 값을 `sha256sum`의 줄 모양으로 |
| `json_roundtrip.bmb` | 표준 입력의 줄마다 `std::json`으로 읽어 공백 없이 다시 쓴다 |
| `json_ref.rs` | 같은 일을 stage0의 `json.rs`로(`#[path]`로 그대로 컴파일) |
| `json_edge.txt` | JSON의 경계 입력(이스케이프, 수, 중첩, 받지 않는 꼴) |
| `unicode_ref.rs` | ASCII 밖의 코드 포인트 전부를 Rust의 `char::is_alphabetic`·`is_numeric`·`is_whitespace`로 갈라 구간으로 |
| `unicode_classes.bmb` | 같은 일을 포트의 표(`selfhost/unicode.bmb`)로 |
| `lexer/` | 렉서의 경계 입력(확장자는 `.txt`: BMB 프로그램이 아닌 글이 다른 코퍼스에 섞이지 않게) |

포트(`selfhost/`)의 사례는 `a`가 `{bmb}`(stage0), `b`가 `{build:stage1}`(`selfhost/*.bmb`를 `bmb build`로)이다. 지금: `codes`, `codes-md`, `tokens`(추적되는 `.bmb` 전부를 파일 하나씩), `tokens-together`(여러 파일을 한 번에), `tokens-edges`(`lexer/`의 파일 하나씩).

`lexer/`의 파일:

| 파일 | 내용 |
| --- | --- |
| `tokens.txt` | 토큰 종류 전부를 잘 쓴 글(오류 없음): 세 진법과 128비트의 끝 값, 이스케이프, 레이블, `///` 묶음 |
| `floats.txt` | 실수 리터럴 1,494개(오류 없음): 무작위 비트 무늬의 가장 짧은 글, 2의 거듭제곱, 부정규수, 이웃한 두 `f64`의 정확한 중간값과 그 양옆, 가장 큰 유한값의 경계, 긴 자리 |
| `err-numbers.txt` | 수 리터럴의 진단(E0104·E0105·E0108·E0109·E0406) |
| `err-text.txt`·`err-bytes.txt`·`err-unterminated.txt` | 문자열·문자·바이트 리터럴의 이스케이프와 끝나지 않은 리터럴(E0101·E0102) |
| `err-quotes.txt` | 레이블, 수명(E0208), 여러 글자의 문자 리터럴 |
| `err-names.txt` | ASCII 밖의 글자: 이름에 든 글자·숫자(한 이름에 한 번 알린다)와 그 밖의 글자, 유니코드 공백 |
| `err-comments.txt` | 블록 주석(E0106) |
| `mix-NN.txt` | 토큰 조각을 무작위로 이은 줄 40개씩(씨앗 고정). 셋에 하나는 오류 없는 조각만 |

파일 하나에 오류가 하나라도 있으면 `dump tokens`는 토큰 대신 진단을 낸다. 그래서 오류 없는 파일이 토큰의 값을, 나머지가 진단의 글·위치·순서를 견준다.

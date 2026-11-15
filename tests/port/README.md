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
| `parser/` | 파서의 경계 입력(같은 이유로 `.txt`) |

포트(`selfhost/`)의 사례는 `a`가 `{bmb}`(stage0), `b`가 `{build:stage1}`(`selfhost/*.bmb`를 `bmb build`로)이다. 지금: `codes`, `codes-md`, `tokens`(추적되는 `.bmb` 전부를 파일 하나씩), `tokens-together`(여러 파일을 한 번에), `tokens-edges`(`lexer/`의 파일 하나씩), `parse`(추적되는 `.bmb` 전부를 파일 하나씩 `dump parse`로), `parse-together`(여러 파일을 한 번에: 단위의 순서와 같은 이름의 단위), `parse-edges`(`parser/`).

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

`parser/`의 파일:

| 파일 | 내용 |
| --- | --- |
| `err-items.txt` | 항목의 진단: 단위 밖의 항목(E0703), `pub`·`static`·`type` 같은 Rust의 항목(E0207·E0215·E0261), `use`(E0214), 속성의 꼴과 Verus의 `verifier`(E0216), `impl`의 속성(E0605), 튜플 구조체와 구조체 꼴 변형(E0233), 제네릭 매개변수와 `where`(E0240), trait과 `impl`의 머리(E0308), 수신자(E0228), `mut` 매개변수(E0210), 반환 타입(E0209·E0256) |
| `err-types.txt` | 타입의 진단: 함수 포인터와 `impl Fn`(E0264·E0265), `dyn`(E0236), 튜플 타입(E0233), 배열 길이(E0406), 쓰지 않는 타입 이름(E0402) |
| `err-stmts.txt` | 문장의 진단: 문장 앞의 속성(E1002), `admit()`(E0217), `assert .. by`(E0516), 패턴 `let`과 `while let`(E0226), `for`의 범위와 측도(E0252·E0253), 절의 순서와 되풀이, 몸의 처음에 쓴 절, 블록 꼴 식 문장의 끝, 복합 대입 |
| `err-exprs.txt` | 식의 진단: 비교의 사슬(E0211), `as`(E0205), 튜플(E0233), 매크로(E0202), `crate::`·`super::`·`self::`(E0242), 호출 밖의 타입 인자(E0301), 필드 줄임(E0222), 클로저(E0263), 포함 범위(E0438), 패턴(E0232), `if let` |
| `deep-KIND-N.txt` | 중첩 한도(256, E0107)의 양옆: 깊이 254·255·256·257·300의 괄호, 전위 연산자, 타입, 참조, 블록, 이항 연산의 사슬, `==>`, 색인·필드·`?`의 사슬, `else if`, 단위, `while`, `return`, `if`, 변환, 호출, `match` (21가지 × 5) |
| `order.txt` | 정규형의 순서: 상수, 안의 단위, trait, 다른 단위의 타입에 대한 trait `impl`, 타입과 그 `impl`들, 제 타입이 없는 `impl`, 함수. 속성의 순서와 `#[uses]`·`#[effects]`의 정렬 |
| `dup-units.txt` | 같은 이름의 최상위 단위와 `summary`가 문자열이 아닌 단위 |
| `empty.txt`·`only-comment.txt` | 토큰이 없는 입력 |

구문 오류가 하나라도 있으면 `dump parse`는 정규형 대신 진단을 낸다. 그래서 오류 없는 파일(코퍼스의 대부분과 `order.txt`, 한도 안의 `deep-`)이 구문 트리와 정규형을, 나머지가 진단의 글·위치·순서와 회복을 견준다. 입력 별칭의 경고(W0912~W0914)는 `dump parse`에 나오지 않는다: 그 글은 `lint`를 옮길 때 견준다.

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
| `lower/` | 함수 값과 상수를 낮추는 단계의 경계 입력(같은 이유로 `.txt`) |
| `decls/` | 검사기 1패스(선언 수집)의 경계 입력(같은 이유로 `.txt`) |
| `types/` | 검사기 2패스(몸 검사)의 경계 입력(같은 이유로 `.txt`) |
| `emit/` | 코드 생성의 입력(같은 이유로 `.txt`)과 허용 목록 `ported.txt`(포트가 IR을 내는 입력: `scripts/port_emit.py write`가 만든다) |
| `gen/` | 검사를 통과하는 생성 프로그램의 얼린 표본 40개(`4D-36`: `tests/fmt_fuzz.rs`의 생성기, `BMB_FMT_FUZZ_EXPORT`). 변형 입력과 달리 검사 뒤의 단계(해시, 정규형, 한정 글)까지 간다. 생성기가 바뀌어도 다시 만들지 않는다 |

포트(`selfhost/`)의 사례는 `a`가 `{bmb}`(stage0), `b`가 `{build:stage1}`(`selfhost/*.bmb`를 `bmb build`로)이다. 지금: `codes`, `codes-md`, `tokens`(추적되는 `.bmb` 전부를 파일 하나씩), `tokens-together`(여러 파일을 한 번에), `tokens-edges`(`lexer/`의 파일 하나씩), `parse`(추적되는 `.bmb` 전부를 파일 하나씩 `dump parse`로), `parse-together`(여러 파일을 한 번에: 단위의 순서와 같은 이름의 단위), `parse-edges`(`parser/`), `std`(추적되는 `.bmb` 전부를 파일 하나씩 `dump std`로: 이름을 든 라이브러리 단위를 더한 프로그램의 정규형), `std-together`(여러 파일을 한 번에), `lower`(추적되는 `.bmb` 전부를 파일 하나씩 `dump lower`로: 함수 값과 상수를 낮춘 프로그램의 정규형, 만든 이름, 넘긴 자리, 상수의 값), `lower-together`, `lower-edges`(`lower/`). `decls`·`decls-together`·`decls-edges`(`dump decls`: 검사기 1패스가 모은 선언). `types`(추적되는 `.bmb`를 파일 하나씩 `dump types`로: 진단, 풀린 프로그램의 정규형, 몸 검사가 계산한 표), `types-together`, `types-edges`(`types/`). `check`(추적되는 `.bmb`와 `types/`의 파일을 하나씩 `check`로: 진단의 JSONL과 종료 코드). `moves`(추적되는 `.bmb`와 `types/`의 파일을 하나씩 `dump moves`로: 함수마다 소유 값이 놓이는 자리), `moves-together`. `hash`(추적되는 `.bmb`와 `types/`의 파일을 하나씩 `hash`로: 정의와 타입 선언의 정체성 해시), `hash-together`. `fmt`(같은 파일들을 하나씩 `fmt`로: 검사한 프로그램의 정규형, 경로로 쓴 메서드 호출은 메서드 문법으로), `fmt-check`(`fmt --check`: 종료 코드와 표준 오류), `fmt-together`. `texts`(같은 파일들을 하나씩 `dump texts`로: 증분 키가 읽는 한정 글과 한정 해시), `texts-together`. `lowered`(같은 파일들을 하나씩 `dump lowered`로: 코드 생성이 읽는 낮춘 표, 글은 바이트이고 `char`는 `u32`: 시그니처, 데이터 타입, 지역 변수, 식의 타입, 글인 자리, 타입마다 참조를 벗긴 타입과 필드·변형), `lowered-together`. `layout`(같은 파일들을 하나씩 `dump layout`으로: 검사를 통과하면 타입마다 LLVM 타입·크기·정렬·술어, 모듈의 머리가 기대는 것, LLVM 타입 정의), `layout-together`. `emit`(추적되는 `.bmb`와 `emit/`·`types/`·`gen/`의 파일을 하나씩 `emit-llvm --all-checks`로: 포트 쪽은 `scripts/port_emit.py run`을 지난다. 허용 목록 `emit/ported.txt`의 입력은 포트의 IR·표준 출력·종료 코드를 stage0과 바이트로 견주고, 목록 밖의 입력은 포트가 "옮기지 않았다"로 끝나는지만 본다. 목록과 포트의 행동이 어긋나면(목록의 입력을 내지 못하거나 목록 밖의 입력을 내면) 실패한다). `emit-run`(`tests/run`과 `emit/`의 파일을 하나씩: 허용 목록에 있고 `main`이 있는 입력의 IR을 두 쪽에서 clang으로 지어 돌린 표준 출력과 종료 코드를 견주고, 옆에 `.out`이 있으면 그것과도 같아야 한다).

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
| `tree-deep.txt` | 중첩 한도 안에서 깊이 1만에 이르는 식 트리: 괄호 50단계마다 연산자 8종의 사슬 25개씩(트리 순회의 깊이, ADR-0140) |
| `order.txt` | 정규형의 순서: 상수, 안의 단위, trait, 다른 단위의 타입에 대한 trait `impl`, 타입과 그 `impl`들, 제 타입이 없는 `impl`, 함수. 속성의 순서와 `#[uses]`·`#[effects]`의 정렬 |
| `dup-units.txt` | 같은 이름의 최상위 단위와 `summary`가 문자열이 아닌 단위 |
| `empty.txt`·`only-comment.txt` | 토큰이 없는 입력 |

구문 오류가 하나라도 있으면 `dump parse`는 정규형 대신 진단을 낸다. 그래서 오류 없는 파일(코퍼스의 대부분과 `order.txt`, 한도 안의 `deep-`)이 구문 트리와 정규형을, 나머지가 진단의 글·위치·순서와 회복을 견준다. 입력 별칭의 경고(W0912~W0914)는 `dump parse`에 나오지 않는다: 그 글은 `lint`를 옮길 때 견준다.

`lower/`의 파일:

| 파일 | 내용 |
| --- | --- |
| `consts-values.txt` | 상수 식의 꼴마다 하나: 리터럴과 연산, `MIN`·`MAX`, 시프트, wrapping·saturating 메서드, `as`, 배열과 반복, 구조체·변형·`Option`·`Result`, 필드·원소·`match`, 이름으로 쓴 길이와 횟수, 안 단위의 상수 |
| `consts-errors.txt` | 상수의 진단: 넘침과 0으로 나눔(E0260), 상수 식이 아닌 것(E0259), 서로를 가리키는 상수(E0318), 같은 이름(E0302), 값이 아닌 타입(E0437), 길이의 진단(E0301·E0406·E0401), 타입이 다른 피연산자(E0401) |
| `closures.txt` | 함수 값을 낮추는 꼴: `Fn`·`FnMut`·`FnOnce` 경계, 값과 참조로 받는 매개변수, 넘긴 함수(같은 함수를 두 번, 경로로), 값을 잡는 `move` 클로저와 그 계약, 클로저 안의 클로저, `impl`과 trait `impl`의 함수 안, 안 단위 |
| `closures-errors.txt` | 함수 값의 진단: `move` 없이 잡음·타입을 쓰지 않은 값을 잡음·바깥 클로저가 잡은 값을 읽음(E0262), 받는 쪽에 없는 효과(E0608), 경계의 인자가 아닌 클로저(E0264) |

`decls`·`decls-together`·`decls-edges`는 `dump decls`(검사기의 1패스가 모은 선언의 표와 그때까지의 진단)를 같은 입력 묶음으로 견준다. `decls/`의 파일:

| 파일 | 내용 |
| --- | --- |
| `types.txt` | 필드와 `impl` 머리에 쓴 타입의 진단: `Self`·모르는 타입(E0305), 값이 아닌 타입 인자(E0403), `Vec`·`Box`·`String`·`Option`·`Result`의 인자 수와 원소(E0420·E0402), 경계(E0428), 소유자 하나(E0238), 포괄 `impl`(E0235), 머리가 부르지 않는 매개변수와 겹치는 `impl`(E0420·E0311), trait이 아닌 이름 |
| `attrs.txt` | 속성과 시그니처의 진단: 불변식의 꼴(E0254·E0255), 효과(E0601·E0602·E0604·E0605·E0607·E0256), `#[external_body]`와 근거(E1001·E1002·E1004), 요약(E0704), 수신자의 이름(E0310), trait 메서드의 `decreases`(E0207)와 겹친 이름(E0302) |
| `dup_units.txt` | 같은 이름의 단위 둘과 함수 값: 낮춘 함수가 제 단위로 돌아간다(`4D-29`) |
| `dup_fns.txt` | 같은 이름의 함수 둘, 하나만 `Fn` 경계(`4D-29`: 고치기 전에는 stage0이 멈췄다) |

`types/`의 파일:

| 파일 | 내용 |
| --- | --- |
| `methods.txt` | 메서드 호출: 타입의 `impl`과 trait의 메서드, 수신자를 넘기는 방식, 메서드로 부를 수 없는 함수(E0314), 두 trait의 같은 이름(E0313), 모르는 메서드(E0404), 정수·비트·`f64`·`Vec`의 내장 메서드와 그 인자 수(E0405), `unwrap`(E0229) |
| `text.txt` | 글: `String::new` 따위 내장 호출, `&str` 인자의 꼴(리터럴, `as_str()`, `&s[a..b]`, E0218), `as_bytes()`, 글과 `char`의 메서드, 명세 안의 글 |
| `calls.txt` | 호출: 모르는 함수(E0304), 부르는 곳과 함수의 종류(E0258·E0504·E0505), 효과(E0606), 쓴 타입 인자와 추론(E0237·E0429), trait 메서드의 `Self`, 조각·구조체의 참조 인자(E0401·E0408·E0409·E0410), 정수 변환(E0412·E0213) |
| `whole.txt` | 프로그램 전체의 검사: `decreases` 없는 재귀(E0507), 경계 있는 `Self`를 지나는 재귀(E0207), 타입 매개변수를 싸서 넘기는 재귀(E0430), `main`의 꼴과 수(E0701·E0702) |

| `moves.txt` | 이동의 진단: 옮긴 뒤의 사용(E0413), 일부가 옮겨진 값의 사용(E0433), 한 길에서만 옮김(E0414: `else` 없는 `if`, `&&`의 오른쪽, 루프의 몸), `&mut` 뒤나 참조에서 꺼냄(E0239·E0413), 참조를 `match`해서 소유 페이로드를 묶음 |
| `moves-ok.txt` | 오류 없는 이동: 놓는 자리의 종류마다(`drops`, `arm_drops`, `branch_drops`, `consumed`, `exits`, `replaces`) |
| `identity.txt` | 정체성 해시: 이름으로 가르는 타입(구조가 같은 두 구조체와 그 trait `impl`이 한 성분), 되부름 성분의 순서(지운 글로 같은 두 함수의 경로 순서, 색 정제로 갈리는 두 함수), trait 메서드의 순서와 기본 몸, 일반화 `impl`, 변형·필드·라벨의 자리 |
| `by-path.txt` | 경로로 쓴 메서드 호출의 정규형(`compiler/tests/fmt.rs`의 `BY_PATH`): 메서드 문법이 되는 것(타고난 함수, 하나뿐인 trait 메서드)과 경로로 남는 것(타고난 함수가 가리는 trait 메서드, 두 trait이 가진 이름, 제 타입 매개변수가 있는 함수, 수신자가 없는 함수) |

`emit/`의 파일(이름의 머리가 그것을 연 백로그 조각이다):

| 파일 | 내용 |
| --- | --- |
| `c1-empty-main.txt` | 몸이 빈 `main`: 모듈의 머리와 `@main` |
| `c1-literals.txt` | 리터럴을 돌려주는 함수: 정수의 폭과 진법, 부호를 바꾼 리터럴, `bool`, `f64`(`-0.0`), `char`, 바이트, 단위 값 |
| `c1-params.txt` | 스칼라 매개변수의 자리, 읽기, 넘기지 않는 단위 값의 매개변수 |
| `c1-units.txt` | 여러 단위와 안 단위의 정의가 경로 순서로, 내지 않는 명세·증명·제네릭 함수 |
| `c1-no-main.txt` | `main`이 없는 프로그램 |
| `c2-ints.txt` | 정수 연산을 폭과 부호마다(10가지): 산술과 넘침 검사, 나눗셈·나머지(0, `MIN / -1`), 비트 연산, 시프트(리터럴 양, 변수 양, 폭 이상의 양), 비교, 부호 바꿈, 비트 뒤집기 |
| `c2-scalars.txt` | `bool`과 `f64`의 연산, 짧은 회로, 지역 변수와 대입, 단위 값의 지역 변수, 블록과 `if`의 값, 코드가 없는 증명 문장 |
| `c2-calls.txt` | 제네릭 아닌 함수의 호출: 스칼라와 단위 값의 인자·결과, 다른 단위와 안 단위와 `impl`의 함수, `print_i64`·`print_u64`, `u32::from` |
| `c3-convs.txt` | 정수 변환 `T::try_from(x).unwrap()`을 원천과 대상의 쌍마다(100가지): 넓어지는 변환은 검사 없이, 나머지는 범위 검사 |
| `c3-methods.txt` | 정수 메서드를 폭과 부호마다: `wrapping_*`·`saturating_*`, 비트 메서드(회전, 세기, 뒤집기, 감기는 시프트, `is_power_of_two`), `MIN`·`MAX`, `f64::from` |
| `c3-floats.txt` | `f64`의 메서드와 상수, `print_f64`(자릿수 검사와 런타임 조각) |
| `c4-loops.txt` | 제어 흐름: `while`, `for`(닫힌 구간과 열린 구간, 부호와 폭), `loop`와 `break`의 값, 레이블로 떠나는 `break`·`continue`, 가운데의 `return` |
| `d1-arrays.txt` | 스칼라 원소의 배열: 상수 리터럴과 셈하는 리터럴, 반복, 원소 읽기와 쓰기(좁은 정수, `bool`, `f64`), 배열 전체의 대입, 값으로 넘기는 배열과 배열 결과, 배열 값의 `if`, `len` |
| `d1-slices.txt` | 스칼라 원소의 슬라이스: 읽기와 쓰기의 매개변수, `len`, 범위로 자른 조각, 바이트열과 글 리터럴, `print_str`·`print_bytes` |
| `d1-refs.txt` | 스칼라의 참조: 지역 변수의 자리, 참조 매개변수를 그대로 넘기기, 원소의 임시 자리와 되돌려 쓰기, 빌려준 값, `*x`의 읽기와 쓰기 |
| `d1-bytes.txt` | 정수와 바이트들: `to_be_bytes`·`to_le_bytes`, `from_be_bytes`·`from_le_bytes`(부호와 폭) |
| `*.out` | `main`이 있는 입력의 기대 출력(`c2-calls`, `c3-floats`, `c4-loops`, `d1-*`): 사례 `emit-run`이 견준다 |

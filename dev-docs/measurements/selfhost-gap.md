# 셀프호스팅 격차 분석 (백로그 4A-01)

> 상태: 분석 결과. 결정이 아니다. 2단계 순서는 1E-07이, 언어 범위는 제안 ADR의 승인이 정한다(ADR-0037 §6).
> 근거: 커밋 `1e1f24f`의 `compiler/src/*.rs`. 수치는 `dev-docs/measurements/selfhost-gap-stats.py`가 낸다(재현: `python3 -I dev-docs/measurements/selfhost-gap-stats.py compiler/src`).
> 짝 문서: 구성 요소 지도·차등 시험·위험은 `selfhost-port-map.md`(4A-02).

## 0. 요약

1. **규모.** stage0은 22개 파일, 13,970줄(테스트 모듈 제외 13,847줄), 493KB다. 외부 크레이트가 하나도 없다(`Cargo.toml`의 `[dependencies]`가 비어 있음). SHA-256·JSON·S-식 파서도 직접 구현했다. 포팅 대상은 Rust 표준 라이브러리 위의 코드뿐이다.
2. **M1 BMB로는 컴파일러의 어떤 구성 요소도 쓸 수 없다.** 입력을 읽을 수 없고(파일·표준 입력 없음), 크기가 실행 중 정해지는 값이 없고(`Vec` 없음), 바이트·문자열 타입이 없다. 렉서도 못 쓴다.
3. **현재 페이즈가 덮지 않는 필수 기능이 넷이다.** (a) 바이트 타입 `u8`과 바이트 문자열(리터럴, 버퍼, 비교) (b) 정렬된 맵·집합(4B에 이름만 있고 설계가 없음) (c) 시스템 인터페이스(파일·디렉터리·이름 바꾸기, 파이프가 있는 프로세스 실행, 인자·환경 변수·종료 코드, 표준 입출력) (d) `Vec`을 필드로 갖는 구조체(ADR-0031이 M2 밖으로 둠). 비트 연산(SHA-256)도 덮이지 않지만 탈출구로 우회할 수 있다.
4. **피할 수 있는 것이 더 많다.** 사용자 제네릭·trait·클로저·반복자·메서드 구문·`for`·`return`·`?`·수명·`Rc`/`RefCell`·스레드·`f64`·`u128`은 단형 코드와 `while`, 인덱스(아레나)로 바꿀 수 있다. 비용은 줄 수다(추정 2~3배, §5.4).
5. **권고 순서(셀프호스팅 관점):** 2B → 2C → 2G(+2F의 `alloc`) → 바이트·문자열(신규) → 2E → 2F 나머지, 그리고 4B·4C를 2단계 뒤가 아니라 2G 직후로 당긴다. 잠정 순서(2B→2C→2E→2F→2G)와 가장 큰 차이는 **2G가 임계 경로**라는 점이다(§5.2).

## 1. 방법

- 대상: `compiler/src/*.rs`에서 첫 `#[cfg(test)]` 앞부분(단위 테스트 제외), `//` 주석 줄 제외. `build.rs`(컴파일러 정체성 해시, 51줄)는 따로 적는다(§3.4).
- 셈: 정규식 근사다. 구문 트리 분석이 아니다. 예를 들어 "클로저"는 `(`·`,`·`=` 뒤의 `|..|`를 센다. 오차는 수 % 수준으로 보고 크기 비교에만 쓴다.
- 판정 열(§4)의 뜻:
  - **M1:** 지금 BMB에 있는가(`compiler/README.md`, `ast.rs`의 `TypeKind`·`BinOp`). M1 타입은 `i64`·`u64`·`usize`·`bool`·`()`·`[T; N]`, 매개변수 위치의 `&[T]`·`&mut [T]`. 연산은 `+ - * / %`, 비교, `&& || !`, `wrapping_add/sub/mul`, `len`. 내장 입출력은 `print_i64` 하나.
  - **제공:** 그 기능을 넣는 페이즈와 제안 ADR. "없음"은 2B~2G·4B·4C 어디에도 명시되지 않았다는 뜻이다(4B·4C는 ROADMAP 표의 한 줄 범위만 있다).
  - **대체:** 더 단순한 구성으로 바꿀 수 있는가. "필수"는 대체가 없다는 뜻이다.
  - **포팅 비용:** 하(기계적, 줄 수 거의 그대로) / 중(구조 변경, 줄 수 1.5~3배) / 상(설계가 필요하거나 증명 부담이 큼).

## 2. 규모

| 파일 | 전체 줄 | 테스트 제외 줄 | 바이트 | fn | 자기 재귀 fn |
| --- | --- | --- | --- | --- | --- |
| ast | 243 | 243 | 4,481 | 4 | 0 |
| cache | 92 | 92 | 3,054 | 6 | 0 |
| check | 1,805 | 1,805 | 65,913 | 44 | 12 |
| codegen | 826 | 826 | 30,650 | 27 | 2 |
| diag | 306 | 306 | 10,290 | 12 | 0 |
| fmt | 514 | 514 | 17,729 | 27 | 2 |
| incremental | 676 | 676 | 26,080 | 33 | 1 |
| json | 313 | 267 | 10,431 | 12 | 1 |
| lexer | 196 | 196 | 6,168 | 1 | 0 |
| lib | 849 | 849 | 29,866 | 32 | 0 |
| lint | 752 | 752 | 27,924 | 18 | 4 |
| main | 652 | 652 | 25,118 | 6 | 0 |
| map | 956 | 956 | 33,437 | 38 | 5 |
| mcp | 448 | 448 | 16,121 | 13 | 1 |
| parser | 1,485 | 1,485 | 47,072 | 61 | 1 |
| sha256 | 107 | 82 | 3,878 | 2 | 0 |
| smt | 366 | 337 | 11,079 | 20 | 2 |
| source | 67 | 67 | 1,640 | 5 | 0 |
| toolchain | 139 | 116 | 4,210 | 10 | 0 |
| uses_fill | 592 | 592 | 21,874 | 13 | 1 |
| verify | 2,203 | 2,203 | 81,874 | 66 | 11 |
| workspace | 383 | 383 | 14,151 | 12 | 2 |
| **합계** | **13,970** | **13,847** | **493,040** | **462** | **45** |

- `fn` 수는 본문이 있는 함수(메서드 포함)다. "자기 재귀"는 본문이 자기 이름을 부르는 것만 센다. 파서의 재귀 하강(`expr`→`primary`→`expr`)처럼 서로 부르는 재귀는 빠져 있다.
- 테스트: `compiler/tests/*.rs` 10,152줄, `#[test]` 113개(+ `src` 단위 테스트 10개). BMB 코퍼스·벤치 `.bmb` 51개(3,459줄)와 기대 진단 `.jsonl` 17개.

## 3. 사용 현황

### 3.1 구성별 사용 수

| 구성 | 합계 | 많이 쓰는 파일(상위 5) |
| --- | --- | --- |
| `String` | 651 | verify 112, lib 89, incremental 61, uses_fill 57, map 57 |
| `&str` | 294 | verify 51, incremental 45, codegen 26, map 21, uses_fill 16 |
| `char` 타입 | 4 | incremental 2, lexer 1, json 1 |
| `.chars()`·`char_indices` | 14 | smt 5, lexer 2, verify 1, uses_fill 1, source 1 |
| `u8`·바이트 | 89 | json 45, lexer 25, incremental 7, fmt 4, sha256 3 |
| `format!` | 505 | verify 142, codegen 84, check 68, fmt 46, workspace 21 |
| `write!`·`writeln!` | 5 | codegen 4, mcp 1 |
| `to_string`·`to_owned` | 104 | verify 21, incremental 14, mcp 9, workspace 8, map 7 |
| `push_str` | 64 | diag 17, verify 9, lib 9, fmt 6, codegen 5 |
| 문자열 메서드(`starts_with`·`split`·`trim`·`parse` 등) | 125 | incremental 24, verify 22, smt 9, lexer 9, toolchain 7 |
| `Vec<` | 229 | verify 37, map 34, lib 21, fmt 18, uses_fill 16 |
| `vec!` | 47 | mcp 19, map 17, workspace 2, verify 2, parser 2 |
| `.push(` | 275 | verify 83, parser 36, check 26, lexer 18, map 17 |
| 슬라이스 `&[`·`&mut [` | 125 | mcp 26, lib 22, verify 19, check 8, incremental 7 |
| `BTreeMap` | 147 | verify 36, check 24, map 20, uses_fill 17, incremental 13 |
| `BTreeSet` | 82 | uses_fill 19, verify 16, incremental 11, map 9, check 9 |
| `HashMap`·`HashSet` | 0 | (결정론 때문에 쓰지 않음) |
| `VecDeque` | 1 | map 1 |
| 맵 연산(`get`·`insert`·`entry`·`contains_key`·`remove`·`range`) | 222 | verify 41, check 36, map 30, incremental 26, uses_fill 13 |
| `Option<` | 109 | lib 19, verify 18, incremental 14, lint 10, check 9 |
| `Some(` | 417 | check 76, verify 71, main 40, incremental 32, lib 25 |
| `None` | 203 | check 51, verify 28, lint 24, parser 19, lib 13 |
| `Result<` | 50 | smt 8, lib 7, json 7, workspace 6, toolchain 5 |
| `Ok(`·`Err(` | 277 | main 70, parser 69, mcp 28, verify 25, workspace 18 |
| `?` 연산자 | 214 | parser 85, incremental 29, mcp 27, smt 21, workspace 18 |
| `unwrap`·`expect` | 33 | lint 5, check 5, verify 4, lib 4, workspace 3 |
| `unwrap_or*`·`map_or`·`ok_or`·`and_then`·`ok` | 111 | verify 21, mcp 14, map 11, smt 10, lib 9 |
| `Box<` / `Box::new` | 19 / 25 | ast 17 / parser 21 |
| `Rc`·`RefCell`·`Cell`·`Arc`·`Mutex` | 13 | incremental 10, toolchain 3 |
| `struct` 선언 | 69 | ast 10, lib 9, verify 8, uses_fill 6, map 6 |
| `enum` 선언 | 22 | ast 7, uses_fill 2, smt 2, codegen 2, check 2 |
| `impl` 블록 | 35 | map 5, verify 3, ast 3, uses_fill 2, source 2 |
| `trait` 선언 / `impl Trait for` | 0 / 5 | map 2, ast 2, smt 1 |
| `#[derive]` | 49 | ast 17, map 4, check 4, verify 3, uses_fill 3 |
| 제네릭 `fn f<..>` | 29 | map 9, lint 3, incremental 3, workspace 2, verify 2 |
| 수명 표기 | 152 | map 51, verify 20, check 20, lint 11, parser 8 |
| `dyn`/`impl` `Fn`·`Iterator` | 14 | verify 3, map 3, parser 2, lint 2, incremental 2 |
| 클로저 | 393 | verify 62, map 53, workspace 34, uses_fill 34, check 33 |
| `match` | 278 | verify 50, check 44, main 37, parser 29, codegen 24 |
| `if let`·`while let`·`let else` | 239 | verify 61, check 33, codegen 24, parser 16, main 16 |
| `for` | 231 | verify 50, check 35, map 27, lint 18, uses_fill 16 |
| `while` / `loop` | 34 / 19 | parser 16·8, lexer 7, json 3·3 |
| `return` | 220 | parser 46, main 37, check 29, verify 20, json 15 |
| `break`·`continue` | 84 | parser 18, verify 16, lexer 10, workspace 7, uses_fill 6 |
| `.iter()`·`into_iter`·`values`·`keys`·`drain` | 240 | verify 40, map 35, check 24, lib 23, uses_fill 22 |
| 반복자 어댑터(`map`·`filter`·`any`·`join` 등) | 469 | verify 74, map 63, lib 53, uses_fill 38, main 33 |
| `collect` | 100 | uses_fill 17, fmt 15, map 14, verify 12, lib 10 |
| `sort*`·`dedup*`·`binary_search` | 26 | incremental 7, map 5, fmt 3, uses_fill 2, lib 2 |
| `.clone()` | 290 | verify 89, map 52, check 32, uses_fill 21, codegen 20 |
| `as` 변환 | 95 | map 22, lint 19, uses_fill 15, source 9, lib 7 |
| `i128`·`u128` | 19 | verify 5, check 4, parser 3, lexer 3, lint 2 |
| `u32` | 79 | source 15, incremental 10, verify 9, lib 8, uses_fill 6 |
| `f64` | 2 | lib 2 |
| 비트 연산(`<< >> ^ & !`, `rotate_*`, `wrapping_*`) | 58 | sha256 40, check 6, verify 4, smt 3, json 2 |
| `checked_*`·`saturating_*` | 3 | lexer 2, parser 1 |
| `std::fs` | 27 | workspace 9, main 7, incremental 4, cache 4, toolchain 2 |
| 프로세스(`Command`·`Stdio`·`ExitCode`) | 77 | main 68(대부분 `ExitCode`), smt 6, toolchain 3 |
| 환경(`env::args`·`var`·`var_os`) | 3 | toolchain 1, smt 1, main 1 |
| `stdin()`·`stdout()` | 3 | mcp 2, main 1 |
| `println!`·`eprintln!` | 57 | main 53 |
| 스레드 | 2 | lib 2 |
| 시각(`Instant`·`Duration`) | 5 | smt 5 |
| `Path`·`PathBuf` | 58 | lib 13, main 11, incremental 9, workspace 8, mcp 6 |
| `BufRead`·`BufReader` 등 | 6 | smt 4, mcp 1, main 1 |

### 3.2 파생 수치

| 항목 | 값 | 뜻 |
| --- | --- | --- |
| 문자열 리터럴 | 2,212개, 45,173바이트 | 진단 메시지·SMT 키워드·JSON 키·IR 조각 |
| 서식 자리 | `{}` 1,044, `{:04x}`·`{:02x}`·`{:.2}`·`{:?}` 각 1 | 사실상 전부 기본 `Display`(정수·문자열) |
| 메서드 호출 `.f(..)` | 3,977 | 그중 `self.f(..)` 968. `&self`·`&mut self` 메서드 정의 208 |
| `let` / 타입 명시 `let` | 1,542 / 146 | 1,396곳에 타입을 써야 한다(E0201) |
| 와일드카드 갈래 `_ =>` | 114 | ADR-0026은 와일드카드를 금지한다(변형마다 갈래) |
| `matches!` | 57 | 매크로(E0202). `match`로 |
| 실행 코드의 열거형 `==`·`!=` | 76 | ADR-0026 열린 질문 1(실행 `==` 없음) |
| 16진 리터럴 | 74 | 대부분 SHA-256 상수표. BMB 리터럴은 10진수만 |
| 문자 / 바이트 리터럴 | 65 / 43 | 렉서·JSON·이스케이프 |
| `const`·`static` 항목 | 46 | 진단 코드표 `CODES`(63쌍), 상수, 메모용 `static` |
| `assert!`·`panic!`·`unreachable!` | 28 | 증명 의무 또는 탈출구로 |
| `std::mem::take/replace/swap` | 11 | ADR-0031이 거부(E0219) |

### 3.3 컬렉션의 원소 타입

| 컨테이너 | 서로 다른 인스턴스 | 사용 | 많은 것 |
| --- | --- | --- | --- |
| `BTreeMap` | 38 | 82 | `<String, String>` 13, `<String, Val>` 9, `<String, FnSig>` 7, `<String, BTreeSet<String>>` 5 |
| `BTreeSet` | 6 | 56 | `<String>` 45, `<ObKey>` 6 |
| `Vec` | 74 | 222 | `<String>` 61, `<(String, String)>` 13, `<Expr>` 11, `<J>` 9 |
| `Option` | 48 | 107 | `<String>` 15, `<&Path>` 13, `<&TypeKind>` 6, `<&mut ProofCache>` 5 |

- 맵·집합의 키는 거의 모두 `String`이다(`BTreeSet` 56개 중 45개, `BTreeMap` 상위 인스턴스 전부). 순회 순서(키의 바이트 사전순)가 출력 바이트를 정한다. 그래서 정렬된 맵은 "바이트 문자열 키 하나"를 잘 지원하면 대부분이 풀린다.
- `Vec` 222곳 중 약 163곳은 원소가 `Copy`가 아니다(`String`, 구조체, 참조). ADR-0031은 `Copy` 원소만 허용한다.
- `Option<&T>`·`Option<&mut T>`가 48곳이다(`Option<&Path>` 13, `Option<&mut ProofCache>` 5 등). 2차 참조(ADR-0015)는 참조를 값 안에 담을 수 없으므로 "있을 때만 넘기는 참조"는 플래그 + 별도 매개변수로 바꿔야 한다.

### 3.4 시스템 인터페이스(전수)

| 용도 | 위치 | 쓰는 것 |
| --- | --- | --- |
| 원자적 쓰기(임시 파일 + 이름 바꾸기) | `cache.rs:83-90`, `incremental.rs:259-270`, `workspace.rs:318-345` | `create_dir_all`, `write`, `rename`, `remove_file`, `read_dir` |
| 소스 읽기 | `workspace.rs:44-62`(`read_dir` 후 정렬, `.bmb` 확장자 거름), `main.rs` | `read_dir`, `read_to_string`(UTF-8 검증), 경로 `display()` |
| Z3 | `smt.rs:59-140, 237` | `Command::new("z3").args(["-in","-smt2","-T:360"])`, 파이프 stdin/stdout, 줄 단위 읽기, `flush`, `wait`. 프로세스 재사용 창 60초(`Instant`) |
| 도구 버전 확인 | `toolchain.rs:41-90` | `z3 --version`, `llc --version` 출력 캡처, `PATH`(`var_os`)를 키로 한 `static Mutex` 메모 |
| 실행 파일 생성 | `toolchain.rs:98-118` | `.ll` 쓰기, `clang -O2` 실행, 종료 상태·stderr 캡처, 파일 삭제 |
| MCP | `mcp.rs:428-448` | stdin 줄 단위 읽기, stdout 쓰기 + `flush` |
| CLI | `main.rs` | `env::args`, `ExitCode` 0·1·2(2가 22곳), `eprintln!` 53곳, `apply -`의 stdin 전체 읽기 |
| 환경 변수 | `smt.rs:51` | `BMB_Z3_RLIMIT`(낮추기만 허용) |
| 스택 | `lib.rs:36-45` | 256MiB 스택 스레드 하나(`STACK_BYTES`, 재귀 패스가 중첩 256단계를 견디도록) |
| 컴파일 시점 값 | `build.rs`, `incremental.rs:80-81`, `mcp.rs:373` | `env!("BMB_SOURCE_DIGEST")`(src와 `toolchain.env`의 SHA-256), `env!("CARGO_PKG_VERSION")`, `toolchain.env` 내용 |
| 오류 문구 | 36곳의 `{e}` | `io::Error`의 `Display`(`strerror` 문구 + ` (os error N)`)가 출력 바이트에 들어간다 |

## 4. 기능별 격차

### 4.1 데이터와 타입

| 필요 기능 | 사용량 | M1 | 제공 | 대체 | 포팅 비용 |
| --- | --- | --- | --- | --- | --- |
| 구조체 | 선언 69 | 없음 | 2B(ADR-0026) | 필수 | 하 |
| 데이터를 가진 열거형 + `match` | 선언 22, `match` 278 | 없음 | 2B | 필수. 단 구조체형 변형(`Stmt::Let { .. }`, `ExprKind::Call { .. }`)은 ADR-0026이 금지 → 구조체를 싼 튜플 변형 | 중 |
| 와일드카드 없는 `match` | `_ =>` 114 | — | 2B(금지) | 빠진 변형마다 갈래를 쓴다. `ExprKind`가 18변형이라 갈래 수가 크게 는다 | 중 |
| `Option`·`Result` | 약 1,000곳(`Some` 417, `Ok/Err` 277 …) | 없음 | 2B | 필수(사용자 열거형으로도 가능) | 하 |
| `?`·조기 `return`·`break` | 214 + 220 + 84 | 없음(E0206 등) | 없음(2R 대상) | 중첩 `match`와 플래그로 재구성 | **상**(파서가 `?` 85, `return` 46) |
| 재귀 데이터(트리) | `Box` 19, 재귀 열거형 4종(`ExprKind`, `json::Value`, `map::J`, `smt::Sexp`) | 없음 | **없음**(ADR-0026이 재귀 타입 금지, ADR-0031이 `Vec`을 담는 구조체 금지) | 아레나: `Vec<Node>` + `usize` 인덱스(DESIGN.md §5의 핸들·인덱스 패턴) | **상**(구문 트리 전체의 표현이 바뀐다) |
| 참조를 담는 구조체 | 수명 표기 152, 수명 매개변수를 가진 구조체·열거형 13개(`Checker<'a>`, `Enc<'p,'q>`, `Model<'p>` …) | 없음(ADR-0015) | 금지(설계) | 문맥을 매개변수로 넘기거나 인덱스로 | 중 |
| `Vec<T>`(`Copy` 원소) | `Vec<` 229, `push` 275 | 없음 | 2G(ADR-0031) | 필수 | 중 |
| 비-`Copy` 원소 `Vec`, `Vec`을 담는 구조체 | 약 163곳 | 없음 | **없음**(ADR-0031 열린 질문 2) | 아레나로 원소를 `Copy` 핸들로 만들 수 있다. 그러나 문맥 구조체(`Enc` 필드 약 20개, `Checker`·`Gen`·`Session`)가 `Vec` 여러 개를 가진다. 금지하면 `&mut Vec` 매개변수가 함수당 5~15개가 된다 | **상** |
| 정렬된 맵·집합 | `BTreeMap` 147, `BTreeSet` 82, 연산 222 | 없음 | 4B("정렬된 맵", 범위 한 줄) | 정렬 배열 + 이분 탐색(대부분 "한 번 만들고 조회"), 또는 BMB로 쓴 균형 트리. 키는 거의 바이트 문자열 | **상** |
| 해시 맵 | 0 | — | — | 불필요 | — |
| 바이트 `u8` | 89 | 없음 | **없음** | `u64` 원소로 흉내 가능(메모리 8배, 원문은 수백 KB라 감당 가능). 파일·파이프 경계에서는 바이트가 필요 | 중 |
| 문자열(소유·조각) | `String` 651, `&str` 294 | 없음 | **없음**(4B "문자열·바이트" 한 줄) | `Vec<u8>` + (시작, 길이) 핸들, 이름은 인턴 표 | **상** |
| 문자열 리터럴 | 2,212개, 45KB | 없음(속성 안에서만) | **없음** | 바이트 문자열 리터럴을 인자 위치의 `&[u8]`로만 허용하면 2차 참조와 맞는다. 반환은 "버퍼에 덧붙이는 함수"로 | 중 |
| `&'static str` 표(`CODES` 63쌍, 코드 문자열) | `const`·`static` 46 | 없음 | 없음 | 정수 코드 + `match`로 버퍼에 쓰는 함수 | 하 |
| `char`, UTF-8 | `chars()` 14, 열 번호 = 문자 수(`source.rs:64`) | 없음 | 없음 | 바이트로 처리(연속 바이트 `10xxxxxx`가 아닌 것을 센다). 문자 타입은 불필요 | 하 |
| `u32` | 79(스팬, SHA-256) | 없음 | 없음 | 스팬은 `u64`/`usize`로. SHA-256만 32비트 래핑이 필요 | 하 |
| `i128`·`u128` | 19(리터럴 값, 상수 부정) | 없음 | 없음 | `u64` + 넘침 검사(2C `checked_*`) 또는 숫자 문자열 그대로 | 하 |
| `f64` | 1곳(`stability`의 `{:.2}`, `lib.rs:346-376`) | 없음 | 없음 | 정수 고정소수점. 동결 전에 stage0을 바꾸는 것을 권고(`selfhost-port-map.md` §6) | 하 |
| 정수 변환 | `as` 95 | 없음(E0205) | 2C(ADR-0028) | 필수(`usize`↔`u64`↔`i64`, 바이트↔정수) | 하 |
| 비트 연산 | 58(SHA-256 40) | 없음 | **없음** | 산술로 흉내(`x / 2^k`, `x % 2^k`)는 가능하나 xor·and는 비트 루프가 된다. 2E `external_body`로 SHA-256 압축 함수를 경계로 두는 것이 현실적 | 중(구현) / 상(증명) |
| 16진 리터럴 | 74 | 없음(10진만) | 없음(2R) | 10진으로 옮긴다(상수표 72개) | 하 |
| 제네릭 함수 | 타입 매개변수 8개(`with_stack`, `chain`, `nested`, `in_unit`, `with_compiler_id`, `with_audit`, `json::err`, `map::strs`) | 없음 | 제외(ADR-0026) | 단형 함수로. 클로저를 받는 고차 함수가 대부분이라 클로저 제거와 함께 사라진다 | 하 |
| trait, `impl Trait for` | 0 / 5(`PartialEq for Type` 등) | 없음 | 제외 | 이름 붙은 비교 함수 | 하 |
| 컨테이너 제네릭 | `Vec` 인스턴스 74, `BTreeMap` 38 | — | `Vec<T>`·`Option<T>`는 내장(2B·2G) | 맵이 라이브러리라면 사용자 제네릭 없이 원소 타입마다 단형 복제가 필요하다. 맵을 `Vec`처럼 내장하거나, 값을 아레나 인덱스로 통일해 `u64→u64`·`bytes→u64` 맵 두 종류로 줄인다 | 중 |
| 실행 코드의 구조적 `==` | 76 | — | ADR-0026 열린 질문 1 | 필드별 비교 함수 | 하 |
| 메서드 구문 | 호출 3,977, 정의 208 | 없음 | 금지(ADR-0026) | `T::f(&x, ..)`. 기계적 | 하(줄 수 불변, 토큰 증가) |
| 클로저 | 393 | 없음 | 없음(설계상 미정) | 이름 붙은 함수 + 명시적 문맥 인자 | 중 |
| 반복자·`for` | `for` 231, 어댑터 469, `collect` 100 | 없음 | 없음 | `while` + `decreases`(+ 범위 불변식) | 중(줄 수 2~4배, 루프마다 `decreases`) |
| `dyn FnMut` 방문자 | `lint.rs:586-607`(`walk_block`·`walk_expr`) | 없음 | 없음 | 변이 종류를 열거형으로 받아 `match` | 하 |
| `Rc`·`RefCell`·`Cell`·`Mutex`·`static` 메모 | 13 + 원자 카운터 1 | 없음 | 없음 | 명시적 문맥. `thread_local` 두 개는 테스트 고리 | 하 |
| `Clone` | 290 | 값 복사 | 2B 값 의미, 2G `v.clone()` | 아레나에서는 핸들 복사가 대부분 | 하 |
| `assert!`·`panic!`·`unreachable!` | 28 | 없음(E0202) | 증명 의무 / 2E·2F(`panic`) | 도달 불가를 전제로 증명하거나 탈출구 | 중 |

### 4.2 알고리즘

| 필요 기능 | 위치·사용량 | M1 | 제공 | 대체 | 포팅 비용 |
| --- | --- | --- | --- | --- | --- |
| 정렬·중복 제거 | 26곳 + 정렬된 맵의 순서 | 없음 | 4B(명시 없음) | 키 타입별 단형 정렬(바이트 문자열, `u64` 튜플). Rust `sort`는 안정 정렬이므로 같은 키의 순서까지 맞춰야 한다 | 중 |
| SHA-256 | `sha256.rs` 82줄. `fmt`(정의 해시), `cache`·`incremental`(줄 해시·키), `build.rs`(정체성) | 없음 | 없음 | 피할 수 없다(해시가 맵·캐시 형식의 일부, ADR-0019) | 중(구현) / 상(증명, `selfhost-port-map.md` §5) |
| 정수 서식 | `{}` 1,044의 정수 부분, `{:02x}`·`{:04x}` | `print_i64`만 | 2C `print_u64`(출력만) | 라이브러리 `push_u64`·`push_i64`·`push_hex` | 하 |
| 정수 해석 | `parse` 17곳(`u32`·`u64`·`usize`) | 없음 | 없음 | 라이브러리(넘침 검사 포함) | 하 |
| JSON 읽기·쓰기 | `json.rs`(파서, `\uXXXX`·대리 쌍 → UTF-8), `map::J`·`diag::json_str`(쓰기) | 없음 | 없음 | BMB로 직접. 재귀는 깊이 256 제한이 이미 있다 | 중 |
| S-식 파서 | `smt.rs:250-366`(모델·응답 읽기) | 없음 | 없음 | BMB로 직접 | 하 |
| 텍스트 조립 | `format!` 505, `push_str` 64, `join` 등 | 없음 | 없음 | 출력 버퍼에 덧붙이는 함수 호출 열. 자리 1,044개가 덧붙임 호출 약 2,000개가 된다 | 중(양이 큼) |
| 재귀 패스 | 자기 재귀 45 + 상호 재귀(파서·검사기·증명기·서식기) | 재귀 + `decreases` | M1 | 아레나 위에서는 연료 매개변수(`MAX_NESTING` = 256)로 `decreases`를 준다 | 중 |

### 4.3 시스템 인터페이스

| 필요 기능 | 사용량 | M1 | 제공 | 대체 | 포팅 비용 |
| --- | --- | --- | --- | --- | --- |
| 파일 읽기·쓰기 | `std::fs` 27 | 없음 | 4C(ADR 미작성) | 필수 | 중 |
| 디렉터리 나열·생성·이름 바꾸기·삭제 | 위 표 | 없음 | 4C | 필수(작업 공간·캐시의 원자적 쓰기) | 중 |
| 프로세스 실행 + 양방향 파이프 | Z3(대화형), `--version` 캡처, clang | 없음 | 4C | 필수. 대화형 파이프가 가장 어렵다(줄 읽기, `flush`) | **상** |
| 명령줄 인자, 종료 코드 | `env::args`, `ExitCode` 0·1·2 | 없음(`main`은 0만) | 4C | 필수 | 하 |
| 환경 변수 | 2곳(`BMB_Z3_RLIMIT`, `PATH`) | 없음 | 4C, 효과 `nondet`(2F) | `PATH`는 메모 키일 뿐이라 뺄 수 있다 | 하 |
| 표준 입력 줄 읽기, 표준 출력·오류 쓰기 | MCP, `apply -`, `eprintln!` 53 | `print_i64`만 | 4C | 필수 | 중 |
| 단조 시계 | `smt.rs` 재사용 창 | 없음 | 2F `nondet` + 4C | 프로세스를 정의마다 새로 띄우면 불필요(느려짐, 1D-18 이전 방식) | 하 |
| 큰 스택 | 256MiB 스레드 | 없음 | 없음 | 링크 옵션·`setrlimit` 또는 연료로 깊이를 묶고 기본 스택으로 측정 | 하 |
| 스레드 | 1곳(스택 용도) | 없음 | 없음 | 불필요 | — |
| 컴파일 시점 상수(`env!`, `toolchain.env`) | 3곳 | 없음 | 없음 | 빌드 절차가 생성한 단위(예: `toolchain` 단위의 상수 함수) | 하 |
| `io::Error` 문구 | 36곳 | — | 4C | 런타임이 `strerror`와 `os error N`을 같은 형식으로 낸다 | 하 |

## 5. 결론

### 5.1 셀프호스팅에 필요한 최소 기능 집합

**언어(대체 불가):**

| # | 기능 | 페이즈 | 비고 |
| --- | --- | --- | --- |
| L1 | 구조체, 튜플 변형 열거형, 망라적 `match` | 2B | ADR-0026 범위 그대로 충분 |
| L2 | `Option`·`Result` | 2B | |
| L3 | 정수 변환(`usize`·`u64`·`i64`, 바이트↔정수) | 2C | ADR-0028 + 바이트 쌍 |
| L4 | 힙 `Vec<T>`(원소는 `Copy` 구조체·열거형) + 이동 검사 | 2G | ADR-0031 범위 |
| L5 | `Vec`을 필드로 갖는 구조체(소유, 이동) | **없음** | ADR-0031 열린 질문 2를 "허용"으로. 아니면 문맥을 매개변수로 펼친다(L5 없이도 가능하지만 함수 시그니처가 커진다) |
| L6 | `alloc` 효과 | 2F(ADR-0030) | 2G와 함께여야 한다(ADR-0030·0031이 이미 그렇게 적음) |
| L7 | 바이트 타입 `u8`과 바이트 문자열 리터럴(인자 위치 `&[u8]`) | **없음** | 신규 페이즈 후보 |
| L8 | 외부 함수 경계(런타임이 제공하는 파일·프로세스·인자·종료 코드·표준 입출력) | **없음**(4C) | `external_body`(2E) 또는 내장 함수 목록. 효과 `io`·`nondet` |
| L9 | 탈출구와 근거 | 2E(ADR-0029) | L8의 신뢰 경계, SHA-256 비트 연산. 4F 미증명 면적 집계의 바탕 |

있으면 크게 줄어드는 것: 비트 연산(SHA-256 증명), `for i in lo..hi`(루프 231곳의 `decreases` 자동화), 조기 탈출(`?`·`return`).

**라이브러리(4B, BMB로 작성·증명):** 바이트 버퍼(덧붙이기, 비교, 조각 핸들, 찾기·접두사·나누기), 정수 서식·해석(10진·16진), 바이트 문자열 키의 정렬된 맵·집합(+ `u64` 키), 키 타입별 정렬, SHA-256, JSON 쓰기(이스케이프)·읽기, UTF-8 문자 수 세기, 인턴 표(이름 → 핸들).

**필요 없는 것(대체로 해결):** 사용자 제네릭·trait, 클로저·반복자, 메서드 구문, 수명·저장된 참조, `Rc`/`RefCell`/전역 상태, 해시 맵, 스레드, `f64`, `u128`, `char`, `u32`(SHA-256 제외), 매크로. 셀프호스팅을 이유로 이것들을 언어에 넣을 필요는 없다.

### 5.2 2단계 페이즈 순서(셀프호스팅 관점)

| 순서 | 잠정안(ROADMAP 2단계) | 셀프호스팅 관점 권고 | 이유 |
| --- | --- | --- | --- |
| 1 | 2B 데이터 타입 | 2B | 모든 구성 요소의 바탕(토큰, 구문 노드, 진단). `Vec::pop`·`checked_*`도 2B에 의존 |
| 2 | 2C 산술과 변환 | 2C | 작고, 바이트·크기 계산(`usize`↔`u64`)에 바로 필요 |
| 3 | 2E 탈출구 | **2G + 2F(`alloc`)** | 2G가 없으면 입력 크기에 비례하는 자료가 없어 어떤 구성 요소도 못 쓴다. 임계 경로이자 가장 큰 페이즈이므로 먼저 시작해야 전체가 짧아진다 |
| 4 | 2F 효과 | **바이트·문자열(신규)** | 렉서부터 필요. 2G의 `Vec<u8>` 위에 얹힌다 |
| 5 | 2G 메모리 | **2E 탈출구** | 4C 외부 경계와 SHA-256에 필요. 2G 뒤여도 포팅 착수 전이면 된다 |
| 6 | — | 2F 나머지(`panic`·`nondet`) | `nondet`의 첫 원천이 4C(환경 변수·시계)다. ADR-0030 열린 질문 2의 답 |
| (병행) | 2R 문법 | 포팅 착수 전 결정(ADR-0037 §6) | §5.4의 수치를 입력으로 |
| 2단계 직후 | 4B·4C | 2G 직후 당김 | 표준 라이브러리·시스템 인터페이스 없이는 4D 포팅이 시작되지 않는다. 2H 측정과 병행 가능 |

- 이 표는 셀프호스팅만의 관점이다. 1E-07은 실제 모델 측정(마찰 빈도)과 함께 정한다. 측정이 2E를 먼저 가리켜도 셀프호스팅 임계 경로는 2G다.
- 2G를 먼저 하려면 ADR-0031의 `pop`이 2B를 기다린다는 점(열린 질문 1)만 남는다. 권고 순서는 2B가 앞이라 충돌이 없다.

### 5.3 어느 페이즈도 덮지 않는 필요 기능

1. **바이트와 문자열:** `u8` 타입, 바이트 문자열 리터럴, 버퍼 연산과 서식. 4B 한 줄("문자열·바이트")만 있고 언어 쪽 페이즈(타입·리터럴·인코딩)가 없다. 2단계에 페이즈를 신설하거나 2G 범위에 넣어야 한다. 증명 인코딩은 `Vec<u8>`과 같은 (배열, 길이) 쌍으로 충분할 것으로 본다(검증 필요).
2. **정렬된 맵·집합:** 4B에 이름만 있다. 사용자 제네릭이 없으므로(ADR-0026) "라이브러리"로 두려면 키·값 타입을 고정해야 한다. 내장할지, `bytes → u64`·`u64 → u64` 단형 맵으로 줄일지 4B-01에서 정해야 한다.
3. **`Vec`을 담는 구조체**(ADR-0031 열린 질문 2)와 **재귀 데이터의 표현**(ADR-0026이 재귀 타입 금지): 아레나로 우회하되, 아레나 묶음을 구조체로 들고 다닐 수 있어야 한다.
4. **시스템 인터페이스:** 파일·디렉터리·이름 바꾸기, 대화형 파이프가 있는 프로세스, 인자·환경 변수·종료 코드, 표준 입력 줄 읽기, 단조 시계. 4C에 범위만 있고 ADR(4C-01)이 없다. 내장 대 FFI, 런타임(C) 크기, 효과 대응을 정해야 한다.
5. **비트 연산과 16진 리터럴:** SHA-256 한 곳뿐이지만 해시가 맵·캐시 형식의 일부라 피할 수 없다. 언어에 넣을지(정수 인코딩 ADR-0017과 충돌: Int 이론에서 비트 연산은 비선형), 탈출구 모듈로 둘지 정해야 한다.
6. **정적 데이터:** 진단 코드표 같은 상수 문자열 표. `match`로 쓰는 함수로 대체할 수 있어 언어 기능은 아니어도 된다.
7. **스택 크기 제어:** 재귀 패스의 깊이 상한(256)이 큰 스택을 전제한다. 런타임 또는 링크 설정의 문제다.

### 5.4 2R(문법)에의 함의

컴파일러 코드가 크게 기대는 Rust 편의 중 BMB가 거부하는 것과 그 양이다. 셀프호스팅은 이 비용을 BMB 쪽에서 한 번에 치른다. 2R-02의 평가 저울에 "자기 코드 작성 비용"으로 넣을 수 있다.

| Rust 편의 | 사용량 | BMB 현행·제안 | 포팅 시 비용 | 2R 검토 후보 |
| --- | --- | --- | --- | --- |
| `for x in ..` + 반복자 어댑터 + `collect` | 231 + 469 + 100 | `while` + `decreases`만 | 루프마다 인덱스 변수·`decreases`·범위 불변식. 해당 줄 2~4배 | **강함.** 범위 `for i in lo..hi`는 `decreases`를 자동으로 줄 수 있어 증명 친화적이다 |
| `?`, 조기 `return`, `break`·`continue` | 214, 220, 84 | 모두 없음(E0206 등) | 파서·`main`·`json`이 특히 깊은 중첩이 된다 | **강함.** `Result` 위의 `?`(2B 뒤)는 정규형 하나와 충돌하지 않는다 |
| 클로저 | 393 | 없음 | 이름 붙은 함수 + 문맥 매개변수 | 중간. 반복자를 빼면 대부분 사라진다 |
| 메서드 호출 구문 | 3,977 | 금지(ADR-0026) | `T::f(&x)`로 기계적. 토큰 증가 | 낮음(호출 해석 금지 원칙) |
| 지역 타입 추론 | `let` 1,396곳 타입 생략 | 금지 | 기계적(`bmb fix`가 채움) | 낮음 |
| `format!` 문자열 보간 | 505(자리 1,044) | 매크로 금지 | 덧붙임 호출 약 2,000개 | 중간. 매크로 없는 보간(예: 라이브러리 빌더 정규형)을 검토할 만하다 |
| `if let`·`matches!`·`_ =>` | 239, 57, 114 | `match` 전부 나열(ADR-0026) | 갈래 수 증가 | 중간. 와일드카드 금지의 비용이 가장 크게 드러나는 곳 |
| 구조체형 변형·튜플 타입 | AST·진단 전반 | 금지(ADR-0026) | 구조체를 싸는 변형 | 낮음 |
| 16진 리터럴 | 74 | 10진만 | 상수표 변환 | 중간(비트 연산과 함께) |
| 셰도잉 | 측정 안 함 | 금지 | 이름 바꾸기 | 낮음 |
| 저장된 참조(수명) | 152 | 금지(설계 핵심) | 아레나·문맥 매개변수 | 없음(원칙) |

**포팅 후 규모 추정:** Rust 13.8K줄 → BMB 약 30~40K줄(라이브러리 별도 3~5K줄). 근거는 위 표의 확장 요인(루프·조기 탈출·서식)과 계약(루프 약 280개의 불변식·`decreases`, 함수마다 명세)이다. 측정이 아니라 추정이다. 4D의 첫 구성 요소(렉서)를 옮긴 뒤 실제 배율로 고친다. 참고로 v0의 셀프호스팅 컴파일러는 약 88K줄이었다(genesis §3.1).

# ADR-0091: 시스템 인터페이스와 런타임

- 상태: 제안 (백로그 4C-03 초안, 채택은 4C-01 승인)
- 날짜: 2026-11-05
- 관련: ADR-0029(탈출구와 신뢰 수준), ADR-0030(효과 `io`·`nondet`), ADR-0037(자립), ADR-0050(문자열), `dev-docs/measurements/selfhost-gap.md` §3.4·§5.3, `selfhost-port-map.md` §4, ROADMAP "언어 전체 수순" ③

## 맥락

- 생성 바이너리는 출력만 한다(`print_i64`·`print_u64`·`print_bytes`·`print_str`·`print_f64`, libc의 `printf`·`fwrite`·`fflush`, 힙은 `malloc`·`realloc`·`free`). **입력이 없다.** 모든 실행 프로그램은 데이터를 소스에 담는다.
- 컴파일러(stage0)가 쓰는 외부 접촉은 격차 분석 §3.4에 다 있다: 소스 읽기(디렉터리 목록·UTF-8 검증), 원자적 쓰기(임시 파일 + 이름 바꾸기), 대화형 파이프가 있는 Z3, 출력을 받는 clang·`--version`, 명령줄 인자·종료 코드 0·1·2, 표준 입력 한 줄·전체, 표준 오류, 환경 변수 하나(`BMB_Z3_RLIMIT`), 단조 시계(Z3 재사용 창), 256MiB 스택, 그리고 출력 바이트에 들어가는 `io::Error` 문구(`strerror` + ` (os error N)`). 4D 포팅(P0)과 부트스트랩 1단은 이것 없이는 시작하지 못한다.

## 결정 (제안)

1. **닫힌 내장 목록, FFI 없음.** 시스템 접촉은 컴파일러가 아는 내장 함수뿐이다(지금의 `print_*`와 같은 방식). 사용자가 선언하는 `extern` 함수는 없다(바인딩은 5E, 원칙 4·6 입증 뒤). 내장의 구현은 생성 IR에 함께 들어가는 런타임(LLVM IR 텍스트, libc 위)이다: C 파일도 별도 라이브러리도 없다(S3 "libc 밖 런타임 0" 유지).
2. **Rust의 이름, BMB에 맞는 시그니처.** 경로와 이름은 Rust 표준 라이브러리를 따른다(S1, ADR-0050 §2와 같은 원칙). Rust가 반복자·빌더·트레이트 객체를 쓰는 곳만 BMB 꼴로 바꾸고 그 차이를 습관 진단으로 안내한다.

   | 영역 | 내장 (제안) | Rust와의 차이 |
   | --- | --- | --- |
   | 파일 | `std::fs::read(p: &str) -> Result<Vec<u8>, io::Error>`, `read_to_string`, `write(p, data: &[u8])`, `rename`, `remove_file`, `create_dir_all`, `read_dir(p) -> Result<Vec<String>, io::Error>`(이름, 바이트순 정렬) | `read_dir`은 반복자 대신 정렬된 이름(결정론), 경로는 `&str` |
   | 표준 입출력 | `std::io::read_line(buf: &mut String) -> Result<usize, io::Error>`, `std::io::read_to_end(buf: &mut Vec<u8>)`, `eprint_str(s: &str)`, `flush()` | `stdin().read_line` 사슬 대신 함수 |
   | 프로세스 | `std::process::exit(code: i32) -> !`, `std::env::args() -> Vec<String>`, `std::env::var(name: &str) -> Option<String>`, `std::process::run(prog, args: &[String], input: &[u8]) -> Result<Output, io::Error>`(끝날 때까지, `Output { status, stdout, stderr }`), `std::process::spawn(prog, args) -> Result<Child, io::Error>`와 `Child`의 `write_all`·`read_line`·`wait`(이동 타입, 해제는 `wait`) | `Command` 빌더 대신 두 함수, `args`는 반복자 대신 `Vec<String>` |
   | 시간 | `std::time::monotonic_ms() -> u64` | `Instant` 대신 수 |
   | 오류 | 내장 구조체 `io::Error { code: i32, message: String }`, `message`는 Rust `Display`와 같은 바이트(`strerror` 문구 + ` (os error N)`) | 트레이트 대신 필드 |

3. **효과와 신뢰.** 모든 시스템 내장은 `io`, 값이 실행마다 다를 수 있는 것(`env::var`·`args`·시계·파일과 프로세스의 내용)은 `nondet`도 쓴다(ADR-0030). 증명에서 결과는 타입 사실(길이 상한, UTF-8은 ADR-0050의 구성 규칙: `read_to_string`은 검증 뒤 `Ok`)만 갖는다: 계약이 없는 바깥 세계다. 신뢰 수준은 `print_*`와 같은 내장(TCB, `dev-docs/HEALTH.md` §3 표에 런타임 함수 목록을 더한다)이고, 미증명 면적(ADR-0029)에는 세지 않는다. 4F의 자기 증명에서 런타임 경계 정의 15~25개(포팅 지도 §5.4)는 이 목록이 된다.
4. **런타임 구현.** libc의 POSIX 함수(`open`·`read`·`write`·`close`·`rename`·`unlink`·`mkdir`·`opendir`/`readdir`·`posix_spawnp`·`pipe`·`waitpid`·`getenv`·`clock_gettime`·`exit`)를 감싼 IR 함수. 쓰는 것만 모듈에 들어간다(지금의 `print_f64`·텍스트 런타임과 같은 규칙). 실패는 `errno`를 `io::Error`로 옮긴다. 할당 실패는 지금처럼 `abort`.
5. **스택.** 런타임이 BMB `main`을 큰 스택의 스레드에서 돌린다(`pthread_create`, 기본 256MiB: stage0의 `STACK_BYTES`, 재귀 중첩 256). 값은 이 ADR의 채택 때 측정으로 정한다.
6. **결정론.** 도구 출력에 들어가는 것은 모두 정해진 순서다: `read_dir`은 정렬, 환경은 이름으로 읽기만, 시계는 출력에 쓰지 않는다(stage0과 같은 규칙: 차등 시험에서 시간 의존 경로는 판정에 쓰지 않는다, 포팅 지도 §4).

## 근거(검토한 대안)

| 안 | 판정 |
| --- | --- |
| **닫힌 내장 목록 + IR 런타임(제안)** | 컴파일러 하나·도구 결정론·작은 TCB. 셀프호스팅에 필요한 접촉 전부(§3.4)를 덮고, 바인딩 없는 단계와 맞는다 |
| 일반 FFI(`extern "C"` + `external_body`) | 포인터·수명·ABI 표면이 언어에 들어온다. 생태계 단계(5E)의 결정이고 지금 수요는 고정된 목록이다 |
| C로 쓴 런타임 라이브러리 | 두 번째 구현 언어와 빌드 단계가 생긴다(S3). IR 텍스트면 같은 `clang` 한 번으로 링크된다 |
| Rust 시그니처 그대로(`Command`, `impl Iterator`) | 빌더·반복자·트레이트 객체가 없는 언어라 표면만 비슷하고 쓸 수 없다. 이름만 따르고 꼴은 바꾼다 |

## 결과

- 구현(4C-02)을 조각으로: (a) `args`·`exit`·표준 오류·표준 입력, (b) 파일, (c) 디렉터리와 원자적 쓰기, (d) 환경·시계, (e) 파이프 프로세스, (f) 스택. 조각마다 실행 프로그램(임시 디렉터리에서 돈다), 오류 경로 시험, `io::Error` 문구를 Rust와 대조.
- HEALTH §3 TCB 표에 런타임 함수가 들어가고, S3는 그대로 `[]`다.
- 4D-02(P0)와 부트스트랩 1단(4E-02)의 막힘이 풀린다.

## 열린 질문

1. `io::Error`를 필드 구조체로 둘지 Rust처럼 메서드(`raw_os_error()`, `kind()`)로 둘지: 메서드 구문(2M)과 함께 정한다.
2. `Child`의 해제(`wait` 없이 블록 끝): 기다릴지(Rust의 `Drop`은 기다리지 않는다) 거부할지(E04xx).
3. 경로를 `&str`로 둘지 바이트(`&[u8]`, 유닉스 경로는 UTF-8이 아닐 수 있다): stage0은 `&str`만 쓴다.
4. 다른 플랫폼(5C): 이 목록은 POSIX다.

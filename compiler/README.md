# bmb 컴파일러

M1(ADR-0016) 최소 관통 경로: 파싱 → 이름·타입·소유권·명세·효과 검사 → 증명 게이트(Z3) → LLVM IR → 실행 파일.
의미론과 인코딩은 ADR-0017·0018.

```
cargo run -- check a.bmb b.bmb   # 진단을 JSONL로 출력. 오류가 있으면 종료 코드 1
cargo run -- verify a.bmb        # 검사 + 증명. 정의별 상태 줄 포함. --emit-smt DIR로 질의 저장
cargo run -- build a.bmb -o app  # 모든 의무가 증명된 경우에만 실행 파일 생성
cargo run -- emit-llvm a.bmb -o a.ll  # IR. 증명되지 않은 의무는 런타임 검사(trap)로 남음
cargo run -- emit-llvm --all-checks a.bmb -o a.ll  # 증명 없이 IR. 모든 의무가 런타임 검사로 남음(계약을 믿지 않으므로 증명되지 않는 변이체도 정직하게 실행, 2A-10)
cargo run -- codes               # 진단 코드 목록(JSONL)
cargo run -- fix a.bmb           # 기계적 수정(replacement)을 적용·재검사 반복. 기본은 보고만(JSONL), --write로 제자리 수정, --print로 고친 텍스트 출력
cargo run -- verify --cache .bmb-cache a.bmb  # 증명 캐시: 해법기 입력이 같은 증명된 의무는 다시 풀지 않음(2D-01). 바뀌지 않은 정의는 인코딩·IR 생성도 건너뜀(2D-10)
cargo run -- stability a.bmb     # Z3 시드 0..5로 증명해 판정이 시드에 따라 바뀌는 정의를 보고(1B-19)
cargo run -- uses a.bmb          # 증명되지 않는 정의마다 최소 #[uses(..)] 목록을 찾아 보고. --write면 파일 수정(2D-05). --cache DIR은 verify와 같은 캐시(2D-15)
cargo run -- lint a.bmb          # 계약 조정 린트: 불가능한 pre, 증명에 쓰이지 않는 pre, 호출자가 쓰지 않는 post, 변이가 살아남는 약한 명세(경고 W09xx, 2D-08·2D-09). --cache DIR은 verify와 같은 캐시(2D-15)
cargo run -- mutate --def u::f a.bmb  # W0904 변이 전부(상한 없음, 증명 안 함)를 프로그램으로: 행마다 바뀐 단위의 정규형(BSB 생성기용, 2A-10)
cargo run -- doctor              # 고정된 Z3·LLVM 버전 확인(scripts/toolchain.env)
../scripts/setup.sh              # 고정 버전 설치(새 클라우드 세션은 시작 훅이 자동 실행)
../scripts/check.sh              # 검증 게이트: fmt·clippy·빌드·백로그·문서, 그다음 테스트 실행 파일과 벤치 검사를 동시에(INF-08, scripts/gate_jobs.py). 종료 코드로 판단
cargo run -- map all a.bmb        # 표준 맵 L0~L2(JSONL, ADR-0020)
cargo run -- query slice u::f -- a.bmb  # 읽기 질의: units surface edges slice find contracts status impact duplicates effects tests
cargo run -- query --cache .bmb-cache status -- ws  # map·query도 --cache DIR을 받는다(증명하는 L1·surface·status만 영향, 출력은 같은 바이트, INF-08)
cargo run -- import ws a.bmb     # 작업 공간 ws/src/에 단위당 정규형 파일 하나
cargo run -- apply ws ops.json   # 쓰기 트랜잭션(unit/put/remove/remove_unit). 구조·타입 오류면 전체 거부
cargo run -- mcp ws              # MCP 서버(stdio): query map verify build tx_begin/put/commit 등
BMB_SOUND_ITERS=60 cargo test --release --test soundness  # 건전성 퍼징(1B-17): 증명·반례·빌드를 독립 해석기와 대조
BMB_FMT_FUZZ_ITERS=2000 cargo test --release --test fmt_fuzz  # 정규형·해시 퍼징(1D-22): 배치·주석·파일 분할에 정규형·해시 불변, fmt 멱등, fmt 전후 상태 줄·IR 동일(앞 BMB_FMT_FUZZ_PROVE개만 증명)
BMB_MCP_FUZZ_SESSIONS=500 cargo test --release --test mcp_fuzz  # MCP 세션 퍼징(1D-23): 임의 도구 호출 순서에서 응답 형식·트랜잭션 모형·커밋만 작업 공간을 바꿈·커밋 뒤 검사 통과와 정규형
cargo test                       # 회귀 코퍼스(tests/check, tests/verify)와 게이트·보조 사실 테스트
BMB_BLESS=1 cargo test           # 기대 진단(.jsonl) 갱신
```

표준 출력이 닫히면(`bmb hash ws | head -1`) 모든 명령은 패닉 없이 종료 코드 141로 끝난다(SIGPIPE로 끝난 프로세스와 같음, 백로그 1D-24, `tests/cli.rs`).


## 구현하며 정한 표면 규칙

ADR-0016의 부분집합을 구체화한 것이다. 모두 "선택을 필수로"(ADR-0013)와 정규형 하나(원칙 8)에서 나왔다.

| 규칙 | 내용 |
| --- | --- |
| 구성 단위 | 최상위에는 `#[summary("..")] mod 이름 { .. }`만 온다. 요약 필수. 다른 단위는 전체 경로(`unit::f`)로 부른다. `use`·`pub` 없음 |
| 함수 | 반환 타입 필수(`-> ()` 포함). 실행 함수는 `#[effects(pure)]` 또는 `#[effects(io)]` 필수. `spec fn`은 효과 속성 없음 |
| 절 순서 | `requires`, `ensures`, `decreases` 순서 고정 |
| 계약 | `requires`·`ensures`의 각 절은 이름 붙은 술어(`spec fn .. -> bool`) 호출. 본문이 `true`인 술어는 거부 |
| 결과 이름 | `ensures`에서 결과를 쓰려면 `-> (r: T)`로 이름을 붙인다 |
| 참조 | `&[T]`·`&mut [T]`는 매개변수에만. 인자로는 `&arr`·`&mut arr` 또는 받은 참조 매개변수를 그대로 넘긴다. `&mut` 인자는 같은 호출의 다른 인자와 겹칠 수 없다. 단, 값으로 넘기는 스칼라 인자가 그 배열을 읽기만 하면 허용(`f(&mut a, a.len())`, ADR-0024) |
| 이름 | 한 함수 안에서 같은 이름을 두 번 묶을 수 없다(블록이 달라도) |
| 리터럴 | 접미사(`5i64`) 없음. 타입은 문맥에서 정해진다. 10진수만 |
| 제어 | `return`·`for`·`loop`·`match`·`break`·`continue` 없음. 블록의 마지막 식이 값. 모든 `while`에 `decreases` 필수, 재귀 함수(상호 재귀 포함)에 `decreases` 필수 |
| 명세 전용 | `forall`, `==>`, `old(..)`, `#[trigger]`는 명세 문맥에서만. `old`는 `ensures`·루프 불변식·`assert`에서 `&mut` 매개변수에만(함수 진입 시점 값) |
| 중첩 한도 | 구문 중첩(괄호·단항·블록·`if`·`else if`·`while`·`==>`·이항 연산과 후위 연산의 사슬·타입·단위)은 256단계까지. 넘으면 E0107. JSON 입력(트랜잭션, MCP)의 배열·객체 중첩도 256까지. CLI와 MCP 서버는 큰 스택(`STACK_BYTES`)의 스레드에서 돈다(백로그 1A-11) |
| 주석 | `//`만. 맵에는 들어가지 않는다. 구조화 근거(ADR-0006)는 M1 이후 |

- 데이터 타입 표면(ADR-0026, 2B 진행 중): 파서와 정규형은 `struct`·`enum`·`impl T`·`Option<T>`·`Result<T, E>`·수신자 `self: &T`·구조체 생성·필드 읽기·`*p`·`match`를 읽고 쓴다(2B-03). 정규형은 단위 안에서 하위 단위 → 타입(이름순, 각 타입의 `impl`이 바로 뒤, 그 안 함수 이름순) → 함수이고, `match`는 갈래마다 한 줄이며, 구조체 생성을 담은 조건은 괄호로 감싼다. 구조체 생성은 `T {` 다음에 `이름 :`이 올 때만이라 `while k < n { .. }`의 블록과 섞이지 않는다. 검사기는 2B-04 전까지 이 구성들을 E0207로 거부한다. 와일드카드 갈래(`_ =>`)와 타입 없는 수신자(`&self`)는 파서에서 E0207과 수정 안내를 낸다.

## 진단

- 한 줄에 객체 하나: `code`, `severity`(`W`로 시작하는 코드는 `warning`, 나머지는 `error`. `W` 코드는 `bmb lint`만 낸다), `message`, `file`, `line`, `col`, `end_line`, `end_col`, 선택적 `fix`(`message`, 기계적으로 고칠 수 있으면 `replacement`).
- 위치 순으로 정렬되고 중복이 제거된다. 같은 입력이면 같은 바이트가 나온다.
- 코드는 `src/diag.rs`의 `CODES`가 유일한 목록이다. 번호는 바꾸거나 재사용하지 않는다.
- `bmb fix`(백로그 2D-04)는 `replacement`가 있는 진단을 모두 적용하고 다시 검사하기를 적용할 것이 없을 때까지 반복한다(최대 16회, 같은 텍스트가 다시 나오면 멈춤). 한 회차에는 (파일, 시작, 끝, 코드, 텍스트) 순으로 보며 이미 고른 것과 겹치지 않는 것만 적용한다(같은 위치의 삽입 둘도 겹침). 출력은 적용한 수정마다 `{"round","file","line","col","code","replacement"}`(위치는 그 회차 전 텍스트 기준), 남은 진단, 요약 행 `{"summary":true,"rounds","applied","changed","remaining","ok"}`. 증명은 돌리지 않는다. 기본은 파일을 바꾸지 않고, `--write`가 바뀐 파일만 제자리에 쓴다. 공백은 정리하지 않으므로 정규형이 필요하면 `bmb fmt`를 이어 쓴다. 오류가 남으면 종료 코드 1
- E02xx는 Rust 습관(타입 생략, 매크로, `return`, `as`, `?` 등)을 따로 묶는다. Rust 습관 오류율 측정(DESIGN.md §4.1)이 이 범주를 센다.

## 증명 게이트

- 실행 함수마다 기호 실행으로 증명 의무를 만든다: 넘침, 0 나눗셈, `i64::MIN % -1`, 인덱스 범위, 호출 전제, 사후 조건, 루프 불변식(진입·유지), `decreases`(루프·재귀·재귀 `spec fn`), `assert`.
- 호출은 callee의 `requires`를 증명하고 `ensures`만 가정한다. `spec fn`은 `#[uses(..)]`에 이름(다른 단위는 `unit::name`)이 있을 때만 정의가 펼쳐지고, 펼침은 전이되지 않는다.
- 의무마다 깨끗한 해법기 상태에서 푼다. `unsat`만 성공이고, `sat`은 반례, `unknown`·자원 초과·해법기 부재는 원인과 함께 실패다.
- 질의는 그 텍스트(가정, 목표, 펼친 정의, 기저 인스턴스)가 언급하는 `spec fn`만 선언한다(백로그 2D-14, `verify::needed_decls`). 무관한 `spec fn`을 더하거나 바꿔도 다른 정의의 질의 텍스트는 바이트 단위로 같다. 측정: 코퍼스(`tests/**`, m1·m1b 시드) 전체에서 `verify` 출력이 이전과 바이트 단위로 같고(판정·`rlimit` 모두), m1b 시드의 `--emit-smt` 스크립트는 1,987,446 → 746,728바이트, `spec fn` 선언 21,294 → 707줄.
- 해법기 프로세스 하나가 여러 정의를 맡는다. 의무마다 `(reset)` 후 옵션을 포함한 기반 전체를 다시 보내므로 결과는 프로세스마다 새로 띄운 것과 같다. 시작 60초가 지난 프로세스는 다음 정의에서 새로 띄워, 정의마다 벽시계 안전장치(300초)를 온전히 받는다(백로그 1D-18).
- 맵·질의는 증명 결과를 담는 것(L1 행: `map L1`·`map all`·`surface`, 그리고 `status`)만 증명을 돌린다. 나머지는 해법기 없이 같은 결과를 낸다(백로그 1D-17, `tests/no_prove.rs`).
- `surface DEF`는 DEF 하나만 증명한다(백로그 3A-04, `verify::verify_only`). 정의의 상태는 자기 의무로만 정해지므로(질의는 자기 원문과 남의 계약·`#[uses]`로만 만들고, 의무마다 해법기를 초기화한다) 전체 증명의 L1 행과 같은 바이트다. 증분 키도 DEF가 닿는 것만 계산하고(`incremental::Session::new_for`), 캐시 파일 줄의 해시는 읽는 줄만 확인한다(저장할 때는 전부). `tests/mcp_cache.rs`가 정의마다 캐시 없음·콜드·웜에서 `map L1` 행과 비교한다. 실패한 정의도 실패 의무 수가 같다(`verify_defs`처럼 첫 실패에서 멈추지 않는다).
- 진단 코드 E08xx. `bmb verify`의 마지막 줄들은 `{"def":..,"status":"verified"|"failed","obligations":n,"failed":k}`.

## `#[uses]` 자동 채우기

`bmb uses [--write] [--cache DIR] <file.bmb|DIR>...`(백로그 2D-05, `src/uses_fill.rs`). 요청할 때만 돌고 `check`·`verify`는 그대로다.

- 증명되지 않는 실행 함수마다 후보 = 현재 `#[uses]` 항목 ∪ 그 정의의 진단 `fix`가 "이름으로만 본다"고 알린 술어(1D-13). 후보로 증명되지 않으면 새로 펼친 본문이 언급하는 술어를 더해 다시 시도한다(최대 8단계). 증명되면 항목을 정렬 순서로 하나씩 빼 보고 여전히 증명되면 뺀 채로 둔다. 결과는 한 항목만 빼서는 더 줄일 수 없는 목록이다(전역 최소는 보장하지 않음).
- 시도마다 속성 텍스트를 고쳐 쓰고 프로그램 전체를 다시 검사·증명한다. 정의의 의무는 자신의 `#[uses]`에만 달려 있으므로(펼침은 전이되지 않고 피호출자는 계약으로만 본다, ADR-0017 §4) 실패한 정의들은 한 번의 검증에서 함께 한 단계씩 나아간다. 검증들은 증분 캐시(2D-15, `incremental::Session::in_memory`와 `rekey`)를 함께 써서 앞선 검증 뒤로 키가 바뀌지 않은 정의는 다시 인코딩하지 않고, 증명 캐시가 바뀐 정의의 바뀌지 않은 의무를 다시 풀지 않게 한다. `--cache DIR`이면 두 캐시를 `DIR`에서 읽고 끝에 써 넣는다(`verify --cache`와 같은 파일). 결과는 캐시와 무관하다(`tests/uses_fill.rs`). 정의당 검증은 최대 32회다. 끝으로 찾은 목록을 모두 넣고 한 번 더 검증해 증명되지 않는 정의는 되돌린다.
- 출력(JSONL): 상태 줄이 있는 정의마다 `{"def","uses","status","verifications"}`. `status`는 `proved`(목록을 바꿔 증명됨), `unchanged`(이미 증명됨), `not_found`(힌트로 만든 어떤 목록으로도 증명되지 않음, 파일은 그대로). `proved` 줄에는 진단과 같은 위치 키(`file`,`line`,`col`,`end_line`,`end_col`)와 `fix.replacement`(속성 하나를 바꾸거나 `fn` 앞에 넣는 텍스트)가 붙는다. 마지막 줄은 `{"summary":true,"defs","proved","unchanged","not_found","verifications","changed"}`. `not_found`가 있으면 종료 코드 1.
- 한계: 진단이 이름을 대지 않는 술어(어떤 의무의 질의에도 나타나지 않는 것)는 찾지 못한다. `spec fn`의 종료 의무는 `#[uses]`를 읽지 않으므로 실패하면 `not_found`다.

## 계약 조정 린트

`bmb lint [--cache DIR] <file.bmb|DIR>...`(백로그 2D-08, DESIGN.md §8.2 "계약 조정", `src/lint.rs`). 요청할 때만 돌고 `check`·`verify`의 출력은 그대로다. 출력은 경고 진단(JSONL, `"severity":"warning"`)과 요약 행(아래). 프로그램이 검사를 통과하지 못하거나 증명 게이트가 돌 수 없을 때(E0812)만 종료 코드 1이다.

| 코드 | 보고 | 방법 |
| --- | --- | --- |
| W0901 | 함수의 `requires`가 함께 성립할 수 없다. 호출할 수 없는 함수이고 그에 관한 증명은 모두 공허하다 | 증명기와 같은 인코딩(매개변수 범위 사실, `#[uses]`로 펼친 정의, 절들)의 충족 가능성 질의 하나(`verify::preconditions_satisfiable`). `unsat`일 때만 보고하고 `unknown`이면 보고하지 않는다. 수량자 정의로 `unknown`이면 증명기처럼 기저 인스턴스로 다시 묻는다 |
| W0902 | `requires` 절 하나가 함수 자신의 증명에 필요 없다 | 그 절 하나를 구문 트리에서 빼고 그 정의만 다시 증명한다(`verify::verify_defs`). 완전히 증명되면 보고한다. pre를 빼도 호출자는 증명할 것이 줄 뿐이라 깨지지 않는다. 기준 검증에서 증명된 함수만, W0901 함수는 제외 |
| W0903 | `ensures` 절 하나를 어느 호출자의 증명도 쓰지 않는다 | 그 절 하나를 빼고 호출자(실행 본문의 `calls` 간선, 재귀면 자신 포함)만 다시 증명한다. 모두 완전히 증명되면 보고한다. 호출자가 있고 기준 검증에서 모두 증명된 함수만 |
| W0904 | 명세가 본문의 변경 하나를 구별하지 못한다: 변이된 함수가 여전히 증명된다(백로그 2D-09). 메시지는 `` `바뀌기 전` -> `바뀐 뒤` `` | 구문 트리에서 본문 노드 하나를 바꾸고 그 정의만 다시 증명한다. 변이: `+`↔`-`, `<`↔`<=`, `>`↔`>=`, `==`↔`!=`, `/`↔`%`, `&&`↔`||`, `*`→`+`, 인덱스 `a[i]`→`a[i + 1]`과 `a[i - 1]`(변이 둘, 2D-16), 정수 리터럴 `c`→`c + 1`(`c`가 `i64` 최댓값−1 이하일 때), `bool` 리터럴 뒤집기, `if` 조건 부정. 실행 코드만 바꾼다(루프 `invariant`·`decreases`, `assert`, `old`, `forall`은 제외). `ensures`가 있고 기준 검증에서 증명되었고 W0901이 아닌 함수만 |

- 절마다 따로 판정한다. 따로 보고된 두 절을 함께 뺄 수 있다는 뜻은 아니다.
- 재검증은 비싸므로 W0902·W0903 각각 32회(`MAX_PRE_TRIALS`, `MAX_POST_TRIALS`)까지만 한다. 넘는 절은 (경로, 절 순서)로 정해진 순서에서 뒤쪽부터 건너뛰고 요약의 `skipped`에 센다. 메모리 안 증명 캐시가 바뀌지 않은 의무를 다시 풀지 않게 한다.
- 변이는 실행당 64개(`MAX_MUTANTS`)까지 증명한다. 순서는 함수를 경로순으로 돌아가며 하나씩(라운드 로빈), 함수 안에서는 본문의 전위 순회 순서다. 넘는 변이는 요약의 `mutants_skipped`에 센다. 요약의 `mutants`는 증명한 변이 수, `surviving`은 살아남은 변이 수(W0904 수)이고, 변이 증명은 `verifications`에 세지 않는다.
- W0904는 정보성이다. 변이가 모든 입력에서 원래 본문과 같은지(동치 변이)는 판정하지 않는다. 예: `if x < 0 { 0 - x } else { x }`의 `x < 0` → `x <= 0`은 `x == 0`에서 두 갈래가 모두 0이라 동치인데도 보고된다. 전제 아래 죽은 코드 안의 변이도 거르지 않는다. `ensures`가 없는 함수는 모든 변이가 살아남으므로 변이하지 않는다.
- `*`는 `+`로만 바꾼다. `+`는 이미 `-`로 바뀌므로 가장 흔한 연산자에 변이를 하나 더 두면 `MAX_MUTANTS` 예산을 차지한다. 인덱스 변이는 대개 범위 증명으로 죽는데, 그것도 다른 죽음과 같다.
- 측정(`tests/run/search.bmb`): 변이 48개 중 9개가 살아남았다(2D-16 전 33개 중 7개). 예: `alg::digit_sum`의 `n / 10` → `n / 11`, `n % 10` → `n / 10`(2D-16), `alg::lower_bound_of`의 `(hi - lo) / 2` → `(hi - lo) / 3`. 마지막 것은 이분 탐색이 계속 맞으므로 실제로 명세가 구별할 필요가 없는 변경이다. 인덱스 변이 6개(`a[mid]`, `a[i]`, `a[best]`)는 모두 죽었다.
- 비용(2D-15): 제한 재검증(`verify::verify_defs`)은 처음 실패한 의무에서 멈춘다. 린트는 "완전히 증명되는가"만 묻기 때문이다. `--cache DIR`이면 첫 전체 검증이 증분 캐시를 쓰고 증명 캐시는 `DIR/proofs`에서 읽고 써 넣는다. 릴리스 빌드 CPU 시간(해법기 포함, 2D-16 전 변이 집합): `search.bmb` 11.7초 → 4.7초(따뜻한 `--cache` 3.8초), m1b 시드 40.3초 → 30.6초(따뜻한 `--cache` 29.8초). m1b에서 남는 비용은 대부분 죽는 변이의 실패 의무(실패는 캐시하지 않는다)와 W0901 충족 가능성 질의 중 `rlimit`까지 가는 수량자 질의 몇 개다.
- 결정론: 고정 시드·`rlimit`, 경로·절·변이 순서. 같은 입력이면 같은 바이트다.
- 요약 행: `{"summary":true,"defs","reports","verifications","skipped","mutants","surviving","mutants_skipped"}`(뒤 세 키는 2D-09에서 추가).
- 한계: W0903은 현재 프로그램의 호출자만 본다(공개 API로 쓸 post도 보고될 수 있어 정보성이다). 기준 검증에서 실패한 함수나 호출자가 실패한 함수는 판정하지 않는다.
- `bmb mutate --def PATH [--def PATH].. <file.bmb>..`(백로그 2A-10, `lint::mutate_sources`): W0904와 같은 변이 지점(같은 순서)마다 변이를 하나 적용한 프로그램을 만든다. 상한도 증명도 없다. 행은 `{"def","site","line","col","before","after","unit","text"}`(`text`는 바뀐 최상위 단위의 정규형, 다른 단위는 그대로)이고 마지막 줄은 `{"summary":true,"mutants"}`. 실행 함수가 아닌 경로나 검사 오류면 종료 코드 1. BSB 변이체 생성기(`bench/m1/bsb_gen.py`)가 쓴다.

## 증분 빌드

`--cache DIR`(MCP는 `.bmb-cache/`)은 증명 캐시(`DIR/proofs`) 옆에 정의별 결과(`DIR/defs`)를 둔다(백로그 2D-10, ADR-0032, `src/incremental.rs`). 키가 그대로인 정의는 기호 실행 인코딩도 IR 생성도 하지 않는다. 출력(진단, 상태 줄, IR, 맵)은 캐시 유무·콜드·웜과 무관하게 같은 바이트다(`tests/incremental.rs`).

- 증명 키: 컴파일러 정체성(`build.rs`가 `src/**`·`toolchain.env`로 만든 SHA-256), Z3 버전, 질의 머리(옵션·`rlimit`·전주곡), 정의의 원문과 한정 정규형, 부르는 실행 함수의 계약 해시, 펼치거나 같은 재귀 성분인 `spec fn`의 본문 해시, 재귀 성분 구성원, 그리고 정의의 텍스트에서 피호출자 계약과 `spec fn` 텍스트를 따라 닿는 `spec fn`마다 그 SMT 선언(2D-14, ADR-0033). `impact` 질의의 간선(`calls`·`uses`·`names`)을 모두 포함하고, 선언은 그보다 넓게(피호출자 계약이 부르는 `spec fn`까지) 잡는다.
- IR 키: 정의의 원문·한정 정규형, 부르는 실행 함수의 호출자 해시, 그 정의 안에서 쓸 수 있는 증명된 의무(1B-31: 실패가 있는 호출 성분은 검사를 유지하므로 그 성분의 IR은 다시 만든다).
- 완전히 증명된 정의만 증명 결과를 저장한다. 실패는 늘 다시 돈다. 재사용된 정의의 의무는 증명 캐시 적중으로 센다.
- 파일 줄마다 SHA-256이 붙고, 맞지 않는 줄·다른 형식 버전·다른 컴파일러의 파일은 무시하고 다시 만든다.
- `bmb verify --emit-smt`는 모든 스크립트가 필요하므로 재사용을 끈다(증명 캐시는 쓴다).
- `spec fn`을 더하거나 빼도, 그것에 닿지 않는 정의는 다시 인코딩하지 않는다(`tests/incremental.rs`의 `adding_an_unrelated_spec_fn_reencodes_nothing`: 스크립트도 바이트 단위로 같다). 시그니처 변경은 그 선언에 닿는 정의만 다시 인코딩한다.
- 한계: 정의 안의 공백·주석 변경도 그 정의를 다시 인코딩한다(증명은 증명 캐시가 받는다).
- 부르는 실행 함수는 호출자 해시로 키에 들어간다: 계약 해시에서 `#[uses]`를 뺀 것(`fmt::caller_text`, ADR-0043). `#[uses]`는 그 함수 자신의 질의에서만 본문을 펼치므로, 피호출자의 `#[uses]`만 바뀌면 그 함수만 다시 인코딩한다(`verify_stats_name_the_reencoded_definitions`, 감사 모드는 `edits_that_shift_spans_or_change_context_keep_outputs_exact`). `bmb hash`·맵의 `iface`는 그대로다.
- `--stats FILE`(`verify`·`emit-llvm`·`build`)은 캐시가 한 일을 JSON 한 줄로 쓴다: `{"encoded":[..],"reused":[..],"lowered":[..],"ir_reused":[..],"proof_cache_hits","proof_cache_misses"}`(정의 경로는 정렬, `verify`는 IR 목록이 빈다). 표준 출력은 그대로다. BCB(2A-14)가 커밋당 재증명 집합을 잰다.

## MCP 서버의 전단 캐시

`bmb mcp`는 호출 사이에 전단 결과를 메모리에 둔다(백로그 3A-03, `FrontCache` in `src/lib.rs`). 호출마다 `src/*.bmb`는 다시 읽는다(다른 쓰는 쪽·직접 편집도 보인다). 이름·위치·원문이 지난 호출과 같은 파일은 다시 렉싱·파싱하지 않고 구문 트리와 진단을 다시 쓴다(스팬이 파일 위치를 담으므로 위치가 바뀐 파일은 다시 파싱). 무엇이든 바뀌면 검사기는 프로그램 전체에 다시 돈다(표가 전역이다). 맵 색인(`map::Index`: 단위, 정의, 간선, 역간선)은 바뀐 뒤 첫 맵 요청에서 다시 만들고, 정의 해시는 처음 쓸 때 계산한다. 아무것도 바뀌지 않으면 모두 다시 쓴다. 정의 단위 질의는 색인의 범위 조회(나가는 간선은 정렬된 간선 집합의 구간, 들어오는 간선은 역간선 표)만 한다. 응답은 호출마다 새 서버를 띄운 것과 같은 바이트다(`tests/mcp_cache.rs`: 질의·맵·`verify`·`build`와 커밋, 다른 서버의 커밋, 직접 편집, 구문 오류, 파일 위치 이동이 섞인 세션). 측정은 `bench/m3/results/2026-10-24-front-cache.md`.

## 코드 생성

- 증명된 의무는 검사 없이 LLVM에 사실로 전달된다: 넘침 없음 → `nsw`/`nuw`, 범위 안 → 검사 없는 `getelementptr inbounds`, 0 아닌 제수 → 검사 없는 `sdiv`/`udiv`. 각 줄에 `; proved Exxxx` 주석이 붙는다.
- 증명되지 않은 의무는 `llvm.trap`으로 가는 검사로 남는다(`; unproved: ..`). `bmb build`는 그런 프로그램을 거부한다.
- `&mut` 슬라이스 매개변수는 `noalias`, `&` 슬라이스는 `readonly`(ADR-0015).
- 닫힌 세계(백로그 2D-07): 모듈 하나가 프로그램 전체이므로 모든 함수(BMB `main`, `bmb_print_i64` 포함)가 `internal`이고, C 진입점 `@main`만 외부에 보인다. LLVM이 자유롭게 인라인하고 쓰지 않는 함수를 지운다. 기준선 벤치 7개에서 실행 파일에 BMB 함수 기호가 0개가 되었고(전에는 2~4개) 크기가 100~180바이트 줄었다. 실행 시간 차이는 측정 잡음(±10%) 안이었다.

## 맵 스키마에 추가된 키

ADR-0020 §1의 하위 호환 규칙(키 추가 허용)에 따라 추가한 키다.

| 키 | 위치 | 뜻 |
| --- | --- | --- |
| `rlimit_max` | L1 행, `status` 질의 행 | 가장 비싼 의무 하나의 해법기 자원 사용량. 고정 `rlimit`(scripts/toolchain.env)에 가까울수록 증명이 깨지기 쉽다(백로그 1B-15) |

`bmb verify`의 상태 줄에는 `rlimit`(정의 전체 합)과 `rlimit_max`가 붙는다.

## 추가된 질의

ADR-0020 §5의 7개 질의 외에 추가한 질의다. 기존 질의와 행 형식은 바뀌지 않는다.

| 질의 | 결과 |
| --- | --- |
| `impact DEF [body\|iface]` | DEF의 본문(`body`) 또는 계약·시그니처(`iface`, 기본값)를 바꾸면 다시 증명할 정의마다 `{"path","reason"}` 행, 그리고 요약 행 `{"summary":true,"changed","change","reprove","defs"}`. `reason`: `changed`(자신), `calls`(호출자: 계약이 바뀜), `uses`(`#[uses]`로 펼침), `names`(시그니처가 바뀐 `spec fn`을 이름으로 씀). 증명은 피호출자를 계약으로만, `spec fn`을 `#[uses]`로만 보므로(ADR-0017 §4) 영향은 한 단계에서 멈춘다(백로그 2D-02) |
| `duplicates` | 본문 해시(`body`)가 같은 정의 묶음마다 `{"hash","defs"}` 행(정의가 둘 이상인 묶음만, 해시순, `defs`는 경로순), 그리고 요약 행 `{"summary":true,"groups","duplicate_defs","defs"}`. 본문 해시는 정의 자신의 이름을 빼므로(ADR-0019 §2) 같은 해시는 이름이나 단위만 다른 같은 정의다(DESIGN.md §7.2, 백로그 2D-06). 자기 재귀 호출은 전체 경로로 남아 재귀 정의의 복제는 묶이지 않는다 |

| `effects [DEF]` | 효과 뷰(DESIGN.md §7.2, 백로그 2D-11). DEF가 있으면 DEF에서 실행 호출(`calls` 간선)로 닿는 `io` 정의(DEF 자신과 내장 `print_i64` 포함)마다 `{"def":DEF,"effect":"io","path"}` 행(`path`는 DEF에서 그 정의까지의 최단 호출 경로, 도달한 정의의 경로순), 그리고 요약 행 `{"summary":true,"def","io_reached","defs"}`. DEF가 없으면 내장 `io` 연산에 닿는 정의마다 `{"def","effect":"io","path"}` 행(가장 가까운 연산까지의 최단 경로, 정의 경로순), 그리고 요약 행 `{"summary":true,"io_reaching","io_defs","defs"}`. 최단 경로가 여럿이면 정의 경로를 차례로 비교해 가장 작은 것을 고른다(호출 순서·정의 순서와 무관). 맵 간선만 쓰고 증명하지 않는다(1D-17의 증명 불필요 집합). 효과 검사(E0603)가 `pure`의 `io` 호출을 막으므로 `io`에 닿는 정의는 모두 `io`로 선언되어 있다. `io`로 선언했지만 어떤 연산에도 닿지 않는 정의는 DEF 없는 결과에 나오지 않는다(`io_defs`와 행 수의 차이) |
| `tests` | 남은 테스트 면적(백로그 2D-21, ADR-0003, DESIGN.md 원칙 4). 테스트가 여전히 필요한 실행 정의마다 `{"path","test_reasons","sites","lines"}` 행(경로순), 그리고 요약 행 `{"summary":true,"exec_defs","test_needed_defs","io_defs","unproved_defs","test_sites","test_lines"}`. M1의 이유는 둘이다: `io`(실행 호출이 내장 `io` 연산에 닿음: 출력의 순서·내용은 계약이 말하지 않는다), `unproved`(실패한 의무가 있어 계약 자체가 서지 않음). `sites`는 그 정의가 직접 부르는 내장 `io` 연산 수(테스트가 겨냥할 곳), `lines`는 정규형 줄 수다. `spec fn`과 `io`로 선언만 하고 어떤 연산에도 닿지 않는 정의는 세지 않는다. 증명 상태를 쓰므로 증명한다(`status`와 같다). G8(2H-06)의 BMB 쪽 값이다. 효과가 늘면(ADR-0030 제안: `panic`·`nondet`·신뢰 등급) 이유가 는다 |

MCP `query` 도구의 설명문은 바꾸지 않았다. 실제 모델 측정(1E-10)이 끝날 때까지 에이전트가 보는 문구를 그대로 둔다. 도구는 `impact`·`duplicates`·`effects`도 받는다.

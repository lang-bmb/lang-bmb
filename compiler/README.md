# bmb 컴파일러

M1(ADR-0016) 최소 관통 경로: 파싱 → 이름·타입·소유권·명세·효과 검사 → 증명 게이트(Z3) → LLVM IR → 실행 파일.
의미론과 인코딩은 ADR-0017·0018.

```
cargo run -- check a.bmb b.bmb   # 진단을 JSONL로 출력. 오류가 있으면 종료 코드 1
cargo run -- verify a.bmb        # 검사 + 증명. 정의별 상태 줄 포함. --emit-smt DIR로 질의 저장
cargo run -- build a.bmb -o app  # 모든 의무가 증명된 경우에만 실행 파일 생성
cargo run -- emit-llvm a.bmb -o a.ll  # IR. 증명되지 않은 의무는 런타임 검사(trap)로 남음
cargo run -- codes               # 진단 코드 목록(JSONL)
cargo run -- fix a.bmb           # 기계적 수정(replacement)을 적용·재검사 반복. 기본은 보고만(JSONL), --write로 제자리 수정, --print로 고친 텍스트 출력
cargo run -- verify --cache .bmb-cache a.bmb  # 증명 캐시: 해법기 입력이 같은 증명된 의무는 다시 풀지 않음(2D-01)
cargo run -- stability a.bmb     # Z3 시드 0..5로 증명해 판정이 시드에 따라 바뀌는 정의를 보고(1B-19)
cargo run -- uses a.bmb          # 증명되지 않는 정의마다 최소 #[uses(..)] 목록을 찾아 보고. --write면 파일 수정(2D-05)
cargo run -- doctor              # 고정된 Z3·LLVM 버전 확인(scripts/toolchain.env)
../scripts/setup.sh              # 고정 버전 설치(새 클라우드 세션은 시작 훅이 자동 실행)
../scripts/check.sh              # 검증 게이트: fmt·clippy·test·백로그 검사
cargo run -- map all a.bmb        # 표준 맵 L0~L2(JSONL, ADR-0020)
cargo run -- query slice u::f -- a.bmb  # 읽기 질의: units surface edges slice find contracts status impact
cargo run -- import ws a.bmb     # 작업 공간 ws/src/에 단위당 정규형 파일 하나
cargo run -- apply ws ops.json   # 쓰기 트랜잭션(unit/put/remove/remove_unit). 구조·타입 오류면 전체 거부
cargo run -- mcp ws              # MCP 서버(stdio): query map verify build tx_begin/put/commit 등
BMB_SOUND_ITERS=60 cargo test --release --test soundness  # 건전성 퍼징(1B-17): 증명·반례·빌드를 독립 해석기와 대조
cargo test                       # 회귀 코퍼스(tests/check, tests/verify)와 게이트·보조 사실 테스트
BMB_BLESS=1 cargo test           # 기대 진단(.jsonl) 갱신
```

## 구현하며 정한 표면 규칙

ADR-0016의 부분집합을 구체화한 것이다. 모두 "선택을 필수로"(ADR-0013)와 정규형 하나(원칙 8)에서 나왔다.

| 규칙 | 내용 |
| --- | --- |
| 구성 단위 | 최상위에는 `#[summary("..")] mod 이름 { .. }`만 온다. 요약 필수. 다른 단위는 전체 경로(`unit::f`)로 부른다. `use`·`pub` 없음 |
| 함수 | 반환 타입 필수(`-> ()` 포함). 실행 함수는 `#[effects(pure)]` 또는 `#[effects(io)]` 필수. `spec fn`은 효과 속성 없음 |
| 절 순서 | `requires`, `ensures`, `decreases` 순서 고정 |
| 계약 | `requires`·`ensures`의 각 절은 이름 붙은 술어(`spec fn .. -> bool`) 호출. 본문이 `true`인 술어는 거부 |
| 결과 이름 | `ensures`에서 결과를 쓰려면 `-> (r: T)`로 이름을 붙인다 |
| 참조 | `&[T]`·`&mut [T]`는 매개변수에만. 인자로는 `&arr`·`&mut arr` 또는 받은 참조 매개변수를 그대로 넘긴다. `&mut` 인자는 같은 호출의 다른 인자와 겹칠 수 없다 |
| 이름 | 한 함수 안에서 같은 이름을 두 번 묶을 수 없다(블록이 달라도) |
| 리터럴 | 접미사(`5i64`) 없음. 타입은 문맥에서 정해진다. 10진수만 |
| 제어 | `return`·`for`·`loop`·`match`·`break`·`continue` 없음. 블록의 마지막 식이 값. 모든 `while`에 `decreases` 필수, 재귀 함수(상호 재귀 포함)에 `decreases` 필수 |
| 명세 전용 | `forall`, `==>`, `old(..)`, `#[trigger]`는 명세 문맥에서만. `old`는 `ensures`·루프 불변식·`assert`에서 `&mut` 매개변수에만(함수 진입 시점 값) |
| 중첩 한도 | 구문 중첩(괄호·단항·블록·`if`·`else if`·`while`·`==>`·이항 연산과 후위 연산의 사슬·타입·단위)은 256단계까지. 넘으면 E0107. JSON 입력(트랜잭션, MCP)의 배열·객체 중첩도 256까지. CLI와 MCP 서버는 큰 스택(`STACK_BYTES`)의 스레드에서 돈다(백로그 1A-11) |
| 주석 | `//`만. 맵에는 들어가지 않는다. 구조화 근거(ADR-0006)는 M1 이후 |

## 진단

- 한 줄에 객체 하나: `code`, `severity`, `message`, `file`, `line`, `col`, `end_line`, `end_col`, 선택적 `fix`(`message`, 기계적으로 고칠 수 있으면 `replacement`).
- 위치 순으로 정렬되고 중복이 제거된다. 같은 입력이면 같은 바이트가 나온다.
- 코드는 `src/diag.rs`의 `CODES`가 유일한 목록이다. 번호는 바꾸거나 재사용하지 않는다.
- `bmb fix`(백로그 2D-04)는 `replacement`가 있는 진단을 모두 적용하고 다시 검사하기를 적용할 것이 없을 때까지 반복한다(최대 16회, 같은 텍스트가 다시 나오면 멈춤). 한 회차에는 (파일, 시작, 끝, 코드, 텍스트) 순으로 보며 이미 고른 것과 겹치지 않는 것만 적용한다(같은 위치의 삽입 둘도 겹침). 출력은 적용한 수정마다 `{"round","file","line","col","code","replacement"}`(위치는 그 회차 전 텍스트 기준), 남은 진단, 요약 행 `{"summary":true,"rounds","applied","changed","remaining","ok"}`. 증명은 돌리지 않는다. 기본은 파일을 바꾸지 않고, `--write`가 바뀐 파일만 제자리에 쓴다. 공백은 정리하지 않으므로 정규형이 필요하면 `bmb fmt`를 이어 쓴다. 오류가 남으면 종료 코드 1
- E02xx는 Rust 습관(타입 생략, 매크로, `return`, `as`, `?` 등)을 따로 묶는다. Rust 습관 오류율 측정(DESIGN.md §4.1)이 이 범주를 센다.

## 증명 게이트

- 실행 함수마다 기호 실행으로 증명 의무를 만든다: 넘침, 0 나눗셈, `i64::MIN % -1`, 인덱스 범위, 호출 전제, 사후 조건, 루프 불변식(진입·유지), `decreases`(루프·재귀·재귀 `spec fn`), `assert`.
- 호출은 callee의 `requires`를 증명하고 `ensures`만 가정한다. `spec fn`은 `#[uses(..)]`에 이름(다른 단위는 `unit::name`)이 있을 때만 정의가 펼쳐지고, 펼침은 전이되지 않는다.
- 의무마다 깨끗한 해법기 상태에서 푼다. `unsat`만 성공이고, `sat`은 반례, `unknown`·자원 초과·해법기 부재는 원인과 함께 실패다.
- 해법기 프로세스 하나가 여러 정의를 맡는다. 의무마다 `(reset)` 후 옵션을 포함한 기반 전체를 다시 보내므로 결과는 프로세스마다 새로 띄운 것과 같다. 시작 60초가 지난 프로세스는 다음 정의에서 새로 띄워, 정의마다 벽시계 안전장치(300초)를 온전히 받는다(백로그 1D-18).
- 맵·질의는 증명 결과를 담는 것(L1 행: `map L1`·`map all`·`surface`, 그리고 `status`)만 증명을 돌린다. 나머지는 해법기 없이 같은 결과를 낸다(백로그 1D-17, `tests/no_prove.rs`).
- 진단 코드 E08xx. `bmb verify`의 마지막 줄들은 `{"def":..,"status":"verified"|"failed","obligations":n,"failed":k}`.

## `#[uses]` 자동 채우기

`bmb uses [--write] <file.bmb|DIR>...`(백로그 2D-05, `src/uses_fill.rs`). 요청할 때만 돌고 `check`·`verify`는 그대로다.

- 증명되지 않는 실행 함수마다 후보 = 현재 `#[uses]` 항목 ∪ 그 정의의 진단 `fix`가 "이름으로만 본다"고 알린 술어(1D-13). 후보로 증명되지 않으면 새로 펼친 본문이 언급하는 술어를 더해 다시 시도한다(최대 8단계). 증명되면 항목을 정렬 순서로 하나씩 빼 보고 여전히 증명되면 뺀 채로 둔다. 결과는 한 항목만 빼서는 더 줄일 수 없는 목록이다(전역 최소는 보장하지 않음).
- 시도마다 속성 텍스트를 고쳐 쓰고 프로그램 전체를 다시 검사·증명한다. 정의의 의무는 자신의 `#[uses]`에만 달려 있으므로(펼침은 전이되지 않고 피호출자는 계약으로만 본다, ADR-0017 §4) 실패한 정의들은 한 번의 검증에서 함께 한 단계씩 나아가고, 메모리 안 증명 캐시가 다른 정의의 증명된 의무를 다시 풀지 않게 한다. 정의당 검증은 최대 32회다. 끝으로 찾은 목록을 모두 넣고 한 번 더 검증해 증명되지 않는 정의는 되돌린다.
- 출력(JSONL): 상태 줄이 있는 정의마다 `{"def","uses","status","verifications"}`. `status`는 `proved`(목록을 바꿔 증명됨), `unchanged`(이미 증명됨), `not_found`(힌트로 만든 어떤 목록으로도 증명되지 않음, 파일은 그대로). `proved` 줄에는 진단과 같은 위치 키(`file`,`line`,`col`,`end_line`,`end_col`)와 `fix.replacement`(속성 하나를 바꾸거나 `fn` 앞에 넣는 텍스트)가 붙는다. 마지막 줄은 `{"summary":true,"defs","proved","unchanged","not_found","verifications","changed"}`. `not_found`가 있으면 종료 코드 1.
- 한계: 진단이 이름을 대지 않는 술어(어떤 의무의 질의에도 나타나지 않는 것)는 찾지 못한다. `spec fn`의 종료 의무는 `#[uses]`를 읽지 않으므로 실패하면 `not_found`다.

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

MCP `query` 도구의 설명문은 바꾸지 않았다. 실제 모델 측정(1E-10)이 끝날 때까지 에이전트가 보는 문구를 그대로 둔다. 도구는 `impact`·`duplicates`도 받는다.

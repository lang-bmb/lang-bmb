# bmb 컴파일러

M1(ADR-0016) 최소 관통 경로: 파싱 → 이름·타입·소유권·명세·효과 검사 → 증명 게이트(Z3) → LLVM IR → 실행 파일.
의미론과 인코딩은 ADR-0017·0018.

```
cargo run -- check a.bmb b.bmb   # 진단을 JSONL로 출력. 오류가 있으면 종료 코드 1
cargo run -- verify a.bmb        # 검사 + 증명. 정의별 상태 줄 포함. --emit-smt DIR로 질의 저장
cargo run -- build a.bmb -o app  # 모든 의무가 증명된 경우에만 실행 파일 생성
cargo run -- emit-llvm a.bmb -o a.ll  # IR. 증명되지 않은 의무는 런타임 검사(trap)로 남음
cargo run -- codes               # 진단 코드 목록(JSONL)
cargo run -- doctor              # 고정된 Z3·LLVM 버전 확인(scripts/toolchain.env)
../scripts/setup.sh              # 고정 버전 설치(새 클라우드 세션은 시작 훅이 자동 실행)
../scripts/check.sh              # 검증 게이트: fmt·clippy·test·백로그 검사
cargo run -- map all a.bmb        # 표준 맵 L0~L2(JSONL, ADR-0020)
cargo run -- query slice u::f -- a.bmb  # 읽기 질의: units surface edges slice find contracts status
cargo run -- import ws a.bmb     # 작업 공간 ws/src/에 단위당 정규형 파일 하나
cargo run -- apply ws ops.json   # 쓰기 트랜잭션(unit/put/remove/remove_unit). 구조·타입 오류면 전체 거부
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
| 명세 전용 | `forall`, `==>`, `old(..)`, `#[trigger]`는 명세 문맥에서만. `old`는 `ensures`의 `&mut` 매개변수에만 |
| 주석 | `//`만. 맵에는 들어가지 않는다. 구조화 근거(ADR-0006)는 M1 이후 |

## 진단

- 한 줄에 객체 하나: `code`, `severity`, `message`, `file`, `line`, `col`, `end_line`, `end_col`, 선택적 `fix`(`message`, 기계적으로 고칠 수 있으면 `replacement`).
- 위치 순으로 정렬되고 중복이 제거된다. 같은 입력이면 같은 바이트가 나온다.
- 코드는 `src/diag.rs`의 `CODES`가 유일한 목록이다. 번호는 바꾸거나 재사용하지 않는다.
- E02xx는 Rust 습관(타입 생략, 매크로, `return`, `as`, `?` 등)을 따로 묶는다. Rust 습관 오류율 측정(DESIGN.md §4.1)이 이 범주를 센다.

## 증명 게이트

- 실행 함수마다 기호 실행으로 증명 의무를 만든다: 넘침, 0 나눗셈, `i64::MIN % -1`, 인덱스 범위, 호출 전제, 사후 조건, 루프 불변식(진입·유지), `decreases`(루프·재귀·재귀 `spec fn`), `assert`.
- 호출은 callee의 `requires`를 증명하고 `ensures`만 가정한다. `spec fn`은 `#[uses(..)]`에 이름(다른 단위는 `unit::name`)이 있을 때만 정의가 펼쳐지고, 펼침은 전이되지 않는다.
- 의무마다 깨끗한 해법기 상태에서 푼다. `unsat`만 성공이고, `sat`은 반례, `unknown`·자원 초과·해법기 부재는 원인과 함께 실패다.
- 진단 코드 E08xx. `bmb verify`의 마지막 줄들은 `{"def":..,"status":"verified"|"failed","obligations":n,"failed":k}`.

## 코드 생성

- 증명된 의무는 검사 없이 LLVM에 사실로 전달된다: 넘침 없음 → `nsw`/`nuw`, 범위 안 → 검사 없는 `getelementptr inbounds`, 0 아닌 제수 → 검사 없는 `sdiv`/`udiv`. 각 줄에 `; proved Exxxx` 주석이 붙는다.
- 증명되지 않은 의무는 `llvm.trap`으로 가는 검사로 남는다(`; unproved: ..`). `bmb build`는 그런 프로그램을 거부한다.
- `&mut` 슬라이스 매개변수는 `noalias`, `&` 슬라이스는 `readonly`(ADR-0015).

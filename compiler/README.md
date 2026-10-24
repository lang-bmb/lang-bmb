# bmb 컴파일러

M1(ADR-0016) 최소 관통 경로의 첫 단계: 파싱 → 이름·타입·소유권·명세·효과 검사 → JSONL 진단.
증명(검증 조건 → Z3)과 LLVM IR 생성은 아직 없다.

```
cargo run -- check a.bmb b.bmb   # 진단을 JSONL로 출력. 오류가 있으면 종료 코드 1
cargo run -- codes               # 진단 코드 목록(JSONL)
cargo test                       # 회귀 코퍼스(tests/check)
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

# 포팅 원장 (백로그 4D-08)

> 구성 요소마다 포트가 무엇을 들였고 어떤 상태인지 적는다. 관문 B0(ADR-0116)와 관문 B·C(ADR-0111)가 이 표를 읽는다.
> 표는 `python3 scripts/components.py ledger --write`가 만든다. 구성 요소의 파일·고정·세션 비용은 `dev-docs/health/components.json`에 있다.

## 열의 뜻

정의는 ADR-0122에 있다.

| 열 | 뜻 | 출처 |
| --- | --- | --- |
| stage0 줄 | 그 구성 요소의 Rust 줄 수(첫 `#[cfg(test)]` 앞, 포팅 지도 §7.1의 규칙) | 파일 |
| 포트 줄, 명세 줄 | `selfhost/`의 그 구성 요소 BMB 줄 수(정규형)와 그중 증명만 읽는 줄(`spec fn`·`proof fn`·`proof { }`, 절·`assert`·증명 속성으로 시작하는 줄) | 파일 |
| 실행 배, 전체 배 | (포트 줄 − 명세 줄) / stage0 줄, 포트 줄 / stage0 줄. ADR-0116의 "계약 없는 배율"과 "증명한 배율"이다 | 파일 |
| 정의, 증명된 정의 | `bmb verify`의 상태 줄 수와 그중 신뢰 사실 없이 `verified`인 것. 조기 고정점까지는 증명 없이 돌 수 있어(`--all-checks`) 이 열이 포트의 증명 상태를 보이는 유일한 곳이다. 전체 비율이 health S7 | `bmb verify` |
| 의무, rlimit | 증명 의무의 수와 해법기 비용의 합 | `bmb verify` |
| 탈출구 | `#[external_body]`와 `assume`의 수 | 파일 |
| 세션 토큰, 게이트 실행, 미결정으로 끝난 실행 | 그 구성 요소를 쓴 세션들의 합: 하네스가 보고한 토큰, `bmb verify`를 돌린 횟수, 그중 남은 오류가 E0811뿐이던 횟수 | `components.json`의 `ledger`(세션 기록에서 옮겨 적는다) |
| 고정 | 차등 시험을 통과해 stage0 쪽이 고정됐는가(ADR-0113). 고정된 구성 요소의 stage0 파일이 바뀌면 게이트가 막는다 | `components.py pin` |

마찰은 구성 요소를 쓰다 만난 언어·도구·증명기의 걸림돌이다. 한 줄에 무엇이 걸렸는지와 처리(백로그 ID, 우회, 그대로)를 적는다.

## 원장

<!-- ledger:data begin (scripts/components.py ledger --write; do not edit by hand) -->

| ID | 구성 요소 | 국면 | stage0 줄 | 포트 줄 | 명세 줄 | 실행 배 | 전체 배 | 정의 | 증명된 정의 | 의무 | rlimit | 탈출구 | 세션 토큰 | 게이트 실행 | 미결정으로 끝난 실행 | 고정 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| C0 | 기반 라이브러리 | P0 | 1,245 | 792 | 58 | 0.8 | 0.8 | 29 | 29/29 | 198 | 4,257,174 | 0 | — | — | — | 고정 |
| C1 | 렉서 | P1 | 862 | 2,004 | 139 | 2.2 | 2.3 | 78 | 78/78 | 616 | 18,369,883 | 0 | — | — | — | 고정 |
| C2 | 파서·구문 트리 | P2 | 5,168 | 5,015 | 255 | 0.9 | 1.0 | 193 | 193/193 | 1,249 | 31,997,838 | 0 | — | 5 | 0 | 고정 |
| C3 | 정규형·해시·정체성 | P3 | 2,629 | 3,845 | 179 | 1.4 | 1.5 | 159 | 159/159 | 1,353 | 50,790,729 | 0 | — | — | — | 고정 |
| C4 | 검사기 | P3 | 17,474 | 22,274 | 720 | 5.1 | 5.2 | 912 | 912/912 | 7,847 | 412,621,347 | 0 | — | 15 | 4 | — |
| C5 | 맵·질의 | P5 | 2,654 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C6 | 증명 인코딩 | P6 | 13,734 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C7 | 해법기·도구 연결 | P7 | 1,102 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C8 | 코드 생성 | P4 | 6,664 | 1,919 | 55 | 0.4 | 0.4 | 99 | 99/99 | 656 | 24,772,608 | 0 | — | 5 | 0 | — |
| C9 | 캐시·증분 | P8 | 1,675 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C10 | 도구 | P9 | 9,641 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C11 | 작업 공간·트랜잭션 | P10 | 1,230 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C12 | MCP 서버 | P11 | 1,494 | — | — | — | — | — | — | — | — | — | — | — | — | — |
| C13 | CLI | P11 | 1,196 | — | — | — | — | — | — | — | — | — | — | — | — | — |

stage0 합계 66,768줄, 조기 고정점(E1: C0, C1, C2, C3, C4, C8) 범위 34,042줄(50%), 포트 합계 35,849줄.

마찰:
- C0: `const`가 `&str`을 담지 못해 진단 코드 표(185 + 6)는 `Vec<Code>`를 만드는 함수다(부를 때마다 만든다). 그대로 둠
- C0: 구조체 필드에 `&'static str`이 없어 `Diag.code`는 `String`이다. 그대로 둠
- C0: `std::json::push_string`·`std::num::push_u64`의 길이 전제를 부르는 쪽이 채울 수 없어 실행 때 길이를 확인하는 감싼 함수 셋(`diag::json_str`·`push_u64`·`push_i64`)을 뒀다가 `4D-20`이 전제를 지워 걷었다(해소)
- C0: `&s[a..b]`의 글자 경계 의무(E0820)를 한 바이트 글자의 앞뒤에서도 증명이 따라가지 못해 `is_char_boundary`로 실행 때 확인(`diag::push_cell`): 그대로 둠
- C0: 정렬은 색인의 삽입 정렬을 직접 썼고 색인의 범위는 실행 때 확인한다(`diag::placed`·`to_jsonl`). `std::slice::sort_indices`는 키가 `Ord` 하나라 네 겹의 순서에 쓰려면 네 번 돌려야 한다: 그대로 둠
- C0: 정규형이 주석을 버린다. 설명은 항목의 `#[summary]`로 옮겼다: 그대로 둠
- C0: 튜플이 없어 `line_col`은 구조체 `Pos`를 낸다: 그대로 둠
- C0: 이 행의 배율은 포트 파일이 있는 `diag`·`source`(stage0 852줄)에 대한 것이다. `sha256`·`json`은 `lib/std`가 대신하고(467줄·945줄 대 82줄·311줄) 포트 줄에 세지 않는다. 구동기 `selfhost/main.bmb`는 국면마다 자라므로 C13의 포트 목록에는 P11에서 넣는다
- C0: `4D-03f1`(`sha256`, `selfhost/sha256.bmb`): stage0 107줄이 174줄, 정의 6개. 상태와 낱말 열은 길이를 계약에 적은 `Vec<u32>`다(`h.len() == 8`), 상수 `K`는 함수 안의 배열 리터럴을 색인으로 읽는 `k(i)`다(전제 `i < 64`). `u32`의 `rotate_right`·`wrapping_add`·비트 연산은 내장이라 그대로다. `bmb verify` 첫 회에 1개 실패: 16진 자리를 글 `"0123456789abcdef"`의 조각 `&s[a..a + 1]`로 붙이던 것(문자 경계 E0820, 임의의 `&str`에서는 참이 아니다)을 `char::from_u32`로 바꿨다
- C1: stage0이 Rust 표준 라이브러리에서 받던 셋을 포트가 직접 든다: 정수 리터럴의 128비트 값(`wide`, `u64` 둘, `4D-17`), 10진 글과 `f64` 사이의 정확한 변환(`big`·`float`, 큰 정수로 읽기와 가장 짧게 쓰기, `4D-18`), 유니코드 글자 분류표(`unicode`, `4D-19`). 이 행의 포트 줄에 넷을 모두 센다: `lexer` 혼자는 1,159줄(stage0 862줄의 1.34배)
- C1: 튜플과 `Result<(char, usize), usize>`가 없어 이스케이프의 결과는 구조체 `Esc`, 부동소수의 부분은 `FloatParts`다: 그대로 둠
- C1: 클로저(`sp`, `doc`, `letter`, `digit`)는 함수가 됐고, 한 루프이던 `lex_as`는 토큰 종류마다 함수 하나로 나뉘었다(`continue 'outer`가 `return`). 각 함수가 앞으로 나아간다는 것은 증명하지 않고 부르는 한곳(`past`, 그리고 길이를 받아 나아가는 `fwd`)이 실행 때 보장한다: 그대로 둠(닿지 않는 갈래)
- C1: `b.get(i + k)`는 `at(text, i, k)`다: 끝을 넘으면 256. 자리 계산의 넘침 의무가 사라진다: 그대로 둠
- C1: `&text[a..b]`의 글자 경계 의무(E0820)는 `cut`이 실행 때 확인한다(C0의 `push_cell`과 같다): 그대로 둠
- C1: 구조체·열거형에 `&'static str`이 없어 `Tok::P`는 `String`이다(구두점마다 할당 하나): 그대로 둠, 속도가 문제가 되면 구두점 번호로
- C1: `match`가 소유한 페이로드를 꺼내지 못하게 막고(E0413) `match &t.tok`을 안내했다: 안내대로
- C1: 한 함수 안에서 이름 하나는 타입 하나다(E0303): `match` 갈래의 묶음 이름을 갈래마다 달리 지었다: 그대로 둠
- C1: 포팅이 stage0의 코드 생성 결함을 찾았다: 구조체의 `Vec` 필드에서 64비트보다 좁은 정수 원소를 8바이트로 읽었다(전부 증명된 프로그램이 틀린 값을 냈다). stage0을 고치고 `tests/run/vec_field_narrow`로 고정
- C1: `ast.bmb`(진법)와 `lib.bmb`(`dump_tokens`)는 조각이라 C2·C10의 포트 목록에 그 국면에서 넣는다. 구동기가 읽지 못한 파일에 내는 표준 오류는 stage0의 것(운영체제의 오류 글)과 다르다: P11에서 맞춘다
- C2: `ast`의 식 순회(`exprs_mut`·`expr_mut`·`block_mut`·`const_exprs_mut`)와 깊은 사본은 `4D-03b`에서 옮겼다. 타입의 순회(`types_mut`·`unit_types_mut`·`fn_types`)는 그것을 쓰는 조각과 함께 옮긴다
- C2: 구조체 꼴 변형이 없어(E0233) `ExprKind::If { .. }` 같은 변형은 구조체 하나를 담는 변형이다(`If(If)`, `Call(Call)`): 그대로 둠. 필드 이름 `requires`·`ensures`·`decreases`는 예약어라 `pre`·`post`·`measure`다
- C2: 재귀 하강의 측도: stage0의 `depth`(중첩 한도 256)를 매개변수 `d`로 넘기고, 측도는 `(256 - d) * 64 + 함수의 차례`다. stage0의 모든 재귀 고리가 `enter`를 지난다는 성질이 그대로 종료의 증명이 됐다(재귀 함수 43개): 그대로 둠
- C2: 토큰을 읽는 루프마다 걸음 수의 상한(토큰 수)을 측도로 두었다. 한 걸음에 토큰 하나 이상을 읽는다는 것은 증명하지 않는다(stage0이 끝나는 입력에서는 상한에 닿지 않는다): 그대로 둠, 렉서의 `past`와 같은 선택
- C2: 토큰 자리의 범위(`pos < len`)는 타입 불변식(`#[invariant(inside)] struct Toks`)이다. 파서의 함수들이 범위의 계약 없이 토큰을 읽는다: 그대로 둠(좋았던 것)
- C2: 참조를 돌려줄 수 없어(E0403) `peek() -> &Tok`가 없다: 토큰을 묻는 함수(`is_p`·`is_kw`·`kind`)와 값을 복사해 주는 함수(`ident`·`literal`)로 나눴다. 구문 트리의 가지를 꺼내는 자리마다 `match`가 필요하다: 그대로 둠
- C2: 와일드카드 갈래가 없어 `ExprKind`(변형 35개)·`TypeKind`(17개)·`Tok`(10개)의 `match`는 언제나 전부를 쓴다. 상수만 내는 갈래가 `parser` 201줄, `fmt` 330줄, `ast` 35줄이다(새 포트 줄의 9%): 그대로 둠, 언어의 선택(ADR-0026)
- C2: 복합 대입 `t op= e`는 자리 `t`의 트리가 둘 필요해 자리를 한 번 더 읽는다(두 번째 읽기의 진단과 길이 이름은 버린다)(쓸 때는 사용자 타입에 `clone`이 없었다. ADR-0141 뒤로는 `.clone()`으로 쓸 수 있으나 고정된 파서를 그것 때문에 다시 열지 않는다): 그대로 둠
- C2: `&&`·`||`의 오른쪽이나 그 안에서 `&mut self`를 부르면 그 뒤에 타입 불변식이 따라오지 않는다(E0823, 건너뛴 길의 반례). `while n > 0 && self.eat_p(..)`를 조건은 묻기만 하고 몸에서 읽도록 바꿨다(10곳): 우회, 증명기의 빈틈 `4D-24`
- C2: 구조적 측도는 구조체·열거형·`Vec`를 건너는 상호 재귀 타입(`Expr`·`ExprKind`·`Block`·`Stmt`)에 쓸 수 없다(E0401). `plain_place`·`type_text`와 `fmt` 전체가 남은 깊이 `f`를 받는다: 우회, `4D-25`
- C2: 포팅이 stage0의 코드 생성 결함을 찾았다: `else if` 조건의 임시값을 `if` 전체가 끝난 자리에서 해제해, 전부 증명된 프로그램이 이중 해제로 죽었다. stage0을 고치고 `tests/run/else_if_temp`를 추가했다(`4D-23`)
- C2: 입력 별칭의 경고(W0912~W0914)와 그 수정 글은 `dump parse`에 나오지 않아 견주지 못했다: `lint`를 옮길 때(P3 뒤) 견준다
- C3: `fmt`는 처음에 `dump parse`가 쓰는 부분만 옮겼다(쓴 그대로의 정규형, 1,890줄). 정체성 글과 한정 글의 방식은 `4D-03f1`, `identity`는 `4D-03f2`, 한정 글의 함수는 `4D-03f4`에서 옮겨 이 행을 닫고 고정했다. `sha256`은 C0, `check::methodize`는 C4의 행에 적는다
- C3: `String`에 `pop`·`truncate`가 없다. 줄 끝을 고치는 두 곳(`}`와 `else`를 잇기, trait 메서드의 `;`)은 그 조각을 따로 받아 잘라 붙인다: 그대로 둠
- C3: `sort_by`가 없어 이름 순서는 `std::slice::sort_indices`(키의 안정 정렬 순서)로 얻는다. 그 길이 전제를 실행 때 확인하던 것은 `4D-22`가 전제를 지워 걷었다(해소). 돌려받은 자리의 범위는 쓰는 곳에서 확인한다: 그대로 둠
- C3: `BTreeMap<(u32, u32, u32), String>`(길이 이름)은 구조체의 `Vec`와 뒤에서부터의 선형 탐색이다: 그대로 둠, 단위마다 몇 개뿐
- C3: `4D-03f1`(출력기의 이름 방식): 구조체가 참조를 들 수 없어 `Printer<'a>`의 `Option<IdMode<'a>>`(검사기의 표와 `&dyn Resolve`)는 출력기가 가진 값 `Names`가 됐다: 방식(`Mode`), 검사기의 표의 사본, 묶인 이름의 표. `dyn Resolve`의 두 구현(참조를 모으는 `Collect`, 성분을 해시하며 쓰는 `Final`)은 방식 둘(`Mode::Collect`·`Mode::Identity`)과 참조의 글을 담은 표 `refs`다: 정체성이 글을 찍기 전에 그 글이 가리키는 것들(참조 그래프의 간선)의 글을 표에 놓는다. `RefCell`로 모으던 열쇠는 출력기의 `seen`이다. 메서드 36개를 더했고 기존 자리 23곳을 바꿨다. `bmb check`의 첫 진단 2개(E0414, E0303), `bmb verify`는 첫 회에 전부 통과(1,451개)
- C3: `4D-03f2`(`identity`, `selfhost/identity.bmb`): stage0 791줄이 778줄, 정의 32개. 구문을 가리키던 참조(`Ent<'p>`의 `&'p FnDef` 등)는 사본이다(프로그램 하나를 한 번 더 든다). `Key`(열거형)는 종류 숫자와 경로를 이은 글이다: 글의 순서가 `Key`의 순서가 되도록 숫자를 골랐다(0 정의, 1 타입, 2 trait). `BTreeMap<Key, _>`은 `table::Table`, `BTreeSet<Key>`는 줄 세운 `Vec<String>`이다. Tarjan의 `while let Some(..) = frames.last_mut()`은 색인으로 읽는 `while`과 걸음의 한도(`fuel`, `decreases`)다: 끝남을 구조로 보이지 못한다. `&mut self.pr`와 `self`의 다른 필드를 함께 넘길 수 없어(E0410) 출력기는 `Ids` 밖에 두고 메서드마다 인자로 넘긴다. `bmb check`의 첫 진단 9개(E0414 넷, E0303 넷, E0426 하나: `match`의 갈래 순서), `bmb verify`는 첫 회에 전부 통과(1,485개). stage0과의 첫 대조에서 차이 하나(정의의 자리 열쇠: `body::DefRow`의 자리는 이름의 자리다), 고친 뒤 0
- C3: `4D-03f4`(한정 글, `selfhost/fmt.bmb`): `qualified_text`·`caller_text`·`qualified_hashes`·`type_hashes`·`trait_hashes`·`trait_impl_address`·`qualify_type`이 정의 14개, 363줄. 쓰는 곳(증분 키, 작업 공간, `escape`)이 아직 옮겨지지 않아 명령의 출력이 없었다: stage0에 `bmb dump texts`를 두어 견줬다. 정의마다 새로 짓던 한정 출력기는 하나를 지어 단위와 글을 바꿔 가며 쓴다(검사기의 표를 정의마다 복사하지 않도록). 함수의 자리로 그 시그니처를 찾는 `fns.values().find(..)`는 자리에서 `impl` 머리로 가는 표다. `defs_of`가 주던 `&FnDef`를 줄에 담을 수 없어 같은 순서의 걷기(`qual_walk`)를 하나 더 썼다: 걷는 순서가 `body::defs_of`와 `identity::collect`까지 세 곳에 있다(그대로 둠, 증분 키를 옮길 때 하나로 모은다). `bmb check`의 첫 진단 0, `bmb verify` 첫 회에 전부 통과(1,515개), stage0과의 첫 대조에서 차이 0
- C4: 닫히지 않은 행이다: 검사기 앞의 단계(`refs`·`stdlib`·`closures`·`consts`와 도우미 `std_units`·`table`)뿐이고 `dump std`·`dump lower`를 견준다(`4D-03b`·`4D-03c`). `consts::literal_tree`는 검사기의 표가 필요해 검사기와 함께 옮긴다. 배율은 옮긴 부분이 C4 전체와 견줘진 것이라 낮게 나온다. `gate_runs`는 지금까지의 `bmb verify` 실행 수다(조각마다 더한다: `4D-03b` 3번, `4D-03c` 3번 중 미결정이 있던 것 2번)
- C4: `&mut dyn FnMut(&mut Expr)`가 없다(E0236·E0263): 순회의 손님은 trait(`ast::ExprFn`, `refs::Rewrite`)의 `impl`을 가진 구조체이고 순회는 그 타입에 제네릭이다. 클로저가 잡던 값은 손님의 필드가 된다: 그대로 둠
- C4: 참조를 돌려줄 수 없어(E0403) 식의 가지를 꺼내는 도우미를 쓸 수 없다. 와일드카드 갈래도 없어, 손님과 식을 가르는 함수마다 `ExprKind`의 갈래 35개를 다 쓴다: `closures` 12곳, `consts` 5곳, `ast` 5곳, `stdlib`·`refs` 1곳씩(24곳 × 35갈래, 대부분 빈 갈래), `TypeKind`(17갈래)는 17곳: 그대로 둠, 언어의 선택(ADR-0026). 결정 기준(사람 지시 2026-11-15)으로는 고칠 후보다
- C4: 갈래가 묶은 값을 다시 `match`하면 E0410(`match`가 빌린 동안 바깥 값을 쓴다고 본다): `match &mut sw.measure { .. }`를 갈래 안에 쓸 수 없어 `Option<Expr>`·`Option<Box<Expr>>`를 받는 도우미 함수로 뺐다(순회마다 둘): 우회, 빌림 검사의 보수성
- C4: 사용자 타입에 `clone`이 없었다(E0404). 구문 트리의 깊은 사본 `ast::clone_*` 86개(약 890줄)를 타입 선언에서 기계로 만들었다가, 결정 기준에 따라 언어에 `clone`을 넣고(ADR-0141, `4D-28`) 지웠다: `ast.bmb` 1,950줄 → 1,063줄, 정의 87개와 그 증명이 사라졌다. 조사 중에 제네릭 `Vec<T>.clone()`의 이중 해제를 찾았다(`4D-27`)
- C4: `include_str!`이 없다: 표준 라이브러리의 글은 `scripts/std_units.py`가 만든 `selfhost/std_units.bmb`(단위마다 문자열 리터럴 하나)이고 게이트가 신선도를 본다: 그대로 둠
- C4: 원소가 옮겨 다니는 `Vec`를 다른 `Vec` 뒤에 붙이는 수단이 없다(`append`·`extend`): 두 번 뒤집어 옮기는 `ast::append<T>`를 썼다: 우회, 표준 라이브러리의 빈칸
- C4: 값이 복사되지 않는 맵을 읽을 수 없다(`BTreeMap::get`은 `V: Copy`, ADR-0097). 포트의 `table::Table<V>`가 열쇠에서 `Vec`의 자리로 가는 맵을 싸고, 자리가 범위 안이라는 것은 실행 때 확인해 계약으로 내준다(`within`): 우회
- C4: stage0의 `stdlib::named`는 프로그램을 복사해 걷는다(걷는 함수가 `&mut`를 받아서). 포트는 복사하지 않고 그 프로그램을 걷는다: 손님이 아무것도 고치지 않아 결과가 같다
- C4: `i128`이 없다. stage0의 상수 평가는 모든 정수를 `i128`로 계산한다. 포트는 부호와 크기(`consts::Num { neg, mag: u64 }`)다: 모든 정수 타입의 값이 거기 들고, 거기 들지 않는 결과는 어느 타입에서도 넘침이라 진단이 같다. 비트 연산과 시프트·wrapping은 64비트 2의 보수 꼴을 거친다(`pattern`·`from_pattern`): 그대로 둠, 경계 입력 `tests/port/lower/consts-*.txt`가 연산마다 견준다
- C4: `Option<&T>`가 없다(참조는 인자로만, ADR-0015): 평가기의 `want: Option<&TypeKind>`는 `&Option<TypeKind>`이고, 문맥의 타입을 주는 자리마다 지역 변수에 `Some(..)`을 담아 그 참조를 넘긴다: 그대로 둠
- C4: `Option`을 돌려주는 함수에서 값을 꺼내 없으면 돌아가는 자리(Rust의 `?`)마다 임시 변수와 `match` 넉 줄이 된다(`let mut x = 기본값; match f() { None => { return None; }, Some(v) => { x = v; } }`): `consts`에 41곳. 갈래의 값 자리에 `return`을 쓰는 꼴이 타입이 맞지 않아 안 된다: 우회. 결정 기준으로는 후보다
- C4: 한 경로에서만 옮긴 값은 E0414다: 조건에 따라 값을 넘기거나 버리는 자리는 버리는 쪽에 `let dropped: T = v;`를 쓴다(`consts::keep`, `array_lens`): 그대로 둠, 해제 자리가 글에 고정된다는 언어의 선택(ADR-0031 §2)
- C4: stage0의 `closures::lower`는 클로저 안의 식을 주소(`*const Expr`)의 집합으로 가려낸다. 포트는 순회에 `deep` 인자를 두어 클로저의 안으로 들어가지 않는다: 같은 식들을 같은 차례로 본다(차등 시험 `lower`). stage0은 함수마다 사본을 고쳐 쓰고 한꺼번에 되돌려 넣는데, 포트는 제자리에서 고친다: 한 함수의 고쳐 쓰기가 다른 함수를 읽지 않아 결과가 같다
- C4: 클로저가 잡던 낮추기의 상태(표 열 개와 셈 둘)는 손님 구조체 `closures::State`의 필드가 됐다. `&mut self`와 `self`의 다른 필드를 한 호출에 넘길 수 없어(E0410) 메서드 대신 `st: &mut State`를 받는 자유 함수다: 그대로 둠
- C4: 구조체의 필드를 도는 루프에서 원소를 `&mut`로 넘기면 증명 인코딩이 비싸다: `refs::module`은 루프 다섯에 88M rlimit(빈 캐시에서 30초), `ast::mod_types`와 `closures::lower`·`plan`은 한도에 걸려 미결정이었다. 루프를 `&mut Vec<T>`를 받는 도우미로 빼면 증명된다(쪼갠 뒤 포트 전체 141M rlimit): 우회. 인코딩 시간 자체는 `T2-20`으로 남겼다(`closures::round_mod` 하나가 66초, 그중 약 55초가 해법기 밖)
- C4: 색인이 범위 안임을 한도 안에 풀지 못한 자리(`st.generics.items[gi].bounded[b]`를 읽으며 `st`의 다른 표를 고치는 루프)는 읽기를 먼저 지역 `Vec`에 모으고 쓰기를 나중에 하도록 갈랐다(`closures::init_names`·`init_traits`·`targets`): 우회
- C4: 포팅이 stage0의 결함 둘을 찾았다. (1) 바깥 단위의 이름으로 쓴 반복 횟수가 뒤에 오는 안 단위의 것으로 읽혀 옳은 프로그램이 E0301로 거부됐다: 파서를 고치고 포트에 같은 커밋에서 반영, `tests/conformance/02/named_length_before_nested_unit`(`4D-26`). JSON 구문 트리 읽기(`ast_json.rs`)에도 같은 결함이 있어 함께 고쳤다. (2) 상수 평가의 진단이 `char`를 Rust의 `Debug`로 썼다(이스케이프가 Rust의 출력 가능 문자 표를 따른다): 정규형이 쓰는 꼴로 바꿨다
- C4: `4D-03d`(1패스): stage0 2,260줄(`collect`에서 닿는 것)에 포트 4,076줄(`types` 1,210 + `decls` 2,866), 배율 1.8. 첫 빌드에서 추적되는 `.bmb` 912개가 전부 바이트 동일했고, 증명 실패는 882개 가운데 둘(루프 불변식 하나씩)이었다. `bmb verify` 전체 실행 2회
- C4: 타입의 같음: stage0의 `TypeKind ==`(자리는 보지 않는다)는 두 값을 함께 가르는 `match`인데 포트에는 그 꼴이 없다(갈래의 묶음을 다시 가르면 E0410). 타입마다 다른 글(`types::key`)을 만들어 글로 견준다: 우회. 짝을 걷는 `overlaps`·`unify`는 갈래마다 도우미 함수 하나씩이다
- C4: 참조를 지역 변수에 둘 수 없어(`let b: &str = ..`) 경계의 이름을 `String`으로 복사해 든다(7곳): 그대로 둠, 언어의 선택(ADR-0015)
- C4: `TypeKind`(17갈래)의 `match`는 전처리(갈래를 채우는 스크립트)로 썼다: 쓰는 글은 갈래 2~6개인데 포트의 글은 17개다(`types`·`decls`에 42곳). 와일드카드 갈래는 넣지 않기로 했다(ADR-0141 근거)
- C4: 내장 선언(함수 28개, 시스템 타입 5개)은 stage0의 `dump decls`에서 만든다(`scripts/builtins.py`): 손으로 옮기면 두 표가 갈라진다
- C4: `clone`(ADR-0141)이 쓰인 자리: 표의 값을 읽고 나서 `self`를 고치는 곳마다(`self.adts.items[i].generics.clone()` 등 90여 곳). 사본 함수는 하나도 쓰지 않았다
- C4: `4D-03e2`(함수의 머리와 문맥): stage0 약 810줄(`check_mod`·`check_fn`·`check_effects`·매개변수의 꼴·`check_contract`·`declare`·`check_const`·`expect_ty`·`block`)에 포트 `body.bmb` 1,345줄(정규형), 배율 1.7. 처음 쓴 글의 `bmb check`가 진단 63개를 냈다: 와일드카드 갈래(E0224) 8, 갈래의 선언 순서(E0426) 4, 참조를 담은 지역 변수(E0403) 4, 실행 코드의 `== None`(E0425) 1, 변수 이름 `spec`(예약어, E0103) 1, 한 경로에서만 옮긴 값(E0414) 1, 한 함수에서 타입이 다른 같은 이름(E0303) 1, 나머지는 그 문법 오류 하나가 뒤의 함수를 모두 가려 난 E0304였다. 세 번째 `check`에서 0개. `bmb verify` 2회: 정의 69개 가운데 실패 하나(진행 중에 길이가 바뀌는 `Vec`의 중간에서 시작하는 `for`의 불변식, 미결정 E0811), 처음부터 돌며 색인을 견주는 꼴로 고쳤다. 갈래를 다 쓰는 `match`는 열거형 선언에서 갈래를 채우는 스크립트로 썼다(`TypeKind` 11곳, `ExprKind` 2곳, `Stmt` 1곳). 문장과 식은 빈 몸이고(`stmt`·`expr`·`measure_type`) 다음 조각이 채운다: 그때까지 `body`는 구동기에 묶이지 않는다
- C4: `4D-03e3a`(문장): stage0 약 420줄(`stmt`, `loop_clauses`, `struct_fields`, `deref_fix`, `untyped_literal`)에 포트 740줄, 배율 1.8. 앞 조각에서 걸린 규칙(갈래를 다 쓰기, 갈래의 순서, 참조 지역 변수, 한 이름에 한 타입)을 알고 쓴 글이라 `bmb check`의 언어 진단은 0개였고, 정의 33개가 첫 `bmb verify`에서 전부 증명됐다. 대신 stage0의 증명기가 멈췄다(`4D-31`: 한쪽이 `return`한 `match`의 갈래가 묶은 이름이 남아 뒤의 `for` 변수와 부딪친다). 한 이름에 한 타입 규칙 때문에 `i`·`k` 같은 이름을 함수 안에서 되풀이 쓰게 되고, 그 꼴이 stage0이 시험하지 않은 길이었다. 문장의 갈래가 묶는 이름은 문장마다 다르게 지었다(`sl`, `sa`, `sw`, ..): `a` 하나로 쓰면 E0303이다
- C4: `4D-03e3b`(자리와 let의 마감): `place` 계열, `try_expr`, `measure_type`(`components`·`reaches_itself`·`struct_cycle`), `settle_lets`, `jump_target`, `field_type_of`, `unknown_local`. `bmb check`의 진단은 처음부터 0개. `bmb verify` 2회: 정의 45개 가운데 `settle_lets` 하나가 미결정 둘(E0811)이었다. `self`의 `Vec` 원소를 도는 루프 안에서 그 원소에 대입하면 길이가 그대로라는 것을 한도 안에 풀지 못한다: 찾기와 쓰기를 `&mut Vec`를 받는 도우미(`settle_diag`)로 뺐다(`4D-03b`의 마찰과 같은 꼴, 우회). stage0의 튜플 반환 `(TypeKind, bool)`은 구조체 `Place`, `Option<(&str, &str)>`인 `try_variants`는 `u8`(0·1·2)다. 클로저가 표를 읽던 `settle_lets`의 순회는 손님 `Uses`가 자리만 모으고(`Fact`·`CallUse`) 표는 순회 뒤에 읽는다: 손님이 `self`의 표를 빌릴 수 없다. 편집 도구의 마찰: 정규형은 한 줄로 긴 몸을 쓰고 닫는 중괄호의 들여쓰기가 깊이마다 달라, 글 조각으로 자리를 찾는 편집이 세 번 틀린 자리를 잘랐다(검사가 곧바로 짚었다). 정의 단위로 바꾸는 도구(`bmb` 쓰기 도구)를 쓰는 것이 맞다
- C4: `4D-03e4`(`match`·변형·구조체 리터럴): `variant_value`, `builtin_variant`, `match_expr`(+`match_variants`·`arm_variant`·`arm_binds`·`match_cover`·`wildcard_arms`·`match_arm`), `struct_lit`(+`expected_args`·`check_slots`·`instance`), `mentions` 계열. 정의 53개. `bmb check`의 진단은 첫 회 5개(E0303 넷: 생성한 갈래의 이름이 두 타입, E0414 하나: 한쪽 길에서만 옮긴 값), 둘째 회 0개. `bmb verify`는 첫 회에 전부 증명(실패 0). 표면이 바꾼 꼴: 조각 `&segs[..n - 1]`이 없어 앞머리를 사본(`head`)으로 만든다. `Vec<(&Expr, TypeKind)>`를 둘 수 없어 `check_slots`는 값을 차례(`Slot.at`)로 가리키고 인자 목록과 필드 목록을 함께 받아 `by_field`로 고른다. 튜플 반환은 구조체(`VariantAt`, `Variants`), 갈래의 지킴이(`Some(t) if ..`)는 술어 함수와 `if`다. stage0의 한 함수 `match_expr`(260줄)는 증명 단위를 줄이려 넷으로 나눴다. 편집: 새 정의를 줄 전체가 맞는 닻(`mod body {`, `    impl Body {`) 뒤에 끼우고 정규형이 자리를 잡게 했다. 잘못 자른 자리 0
- C4: `4D-03e5a`(연산과 리터럴): `binary`(셋으로 나눔: 피연산자를 보는 `binary`, 타입이 다를 때의 `binary_types`, 같을 때의 `binary_same`), `shift`, `int_lit`, `f64_from`, `bit_vector_terms`, `const_ref`, `literal_elem`, `recursive_with_vec`, `text_operand`. 정의 35개. `bmb check`의 진단 0개, `bmb verify` 첫 회에 전부 증명. 표면이 바꾼 꼴: stage0의 `u128` 리터럴 값과 `i128` 경계는 `wide::U128`과 `consts::Num`(부호와 크기)으로 견준다. 지킴이가 겹친 `match`(`bit_vector_terms`의 갈래 15개)는 갈래마다 `if`로 풀고 결과를 수(`what: u8`)로 든다: 글을 만드는 갈래와 되부르는 갈래가 한 `match`에 섞여 값으로 내기 어렵다. 재귀 측도는 조각마다 `f - 1`로 내려 부른다: 같은 깊이에서 서로 부르는 함수의 층(3f, 3f+1, 3f+2)이 셋뿐이라 넷째 층이 필요한 자리는 깊이를 하나 쓴다
- C4: `4D-03e5b`(식의 갈래): `expr`, `expr_inner`, 갈래마다의 검사 23개, `collect_triggers`, `first_index_on`, `settled`. 정의 68개. `bmb check`의 진단 0개. `bmb verify` 2회: `call_expr`와 `path_expr`가 미결정 셋(E0811, 색인의 범위). `builtin_path(p)`나 `vec_builtin(p)`가 참이면 길이가 2(또는 1 이상)라는 것을 증명기는 모른다: 술어 함수의 몸은 호출하는 쪽에 보이지 않는다. 길이 조건을 `if`에 함께 적었다(뜻은 같다). stage0의 `match`가 지킴이 23개로 고르던 호출의 갈림(`call_expr`)은 `if`의 사슬이고, `expr_inner`의 갈래는 메서드 하나씩이다: 한 정의가 커지면 증명이 느려진다. 옮기다 stage0의 결함을 찾았다(`4D-32`: `u32::from()`이 `args[0]`을 길이 확인 없이 읽는다). BMB에서는 그 색인에 증명 의무가 붙어 같은 글을 쓸 수 없다
- C4: `4D-03e6a`(내장 호출과 내장 메서드): `box_new`, `vec_call`, `mem_call`, `vec_method`, `f64_method`, `copy_within`, `clone_method`. 정의 20개. `bmb check`의 진단 0개, `bmb verify` 첫 회에 전부 증명. 표면이 바꾼 꼴: stage0의 `Option<&'static str>`(까닭의 글이 있거나 없다)은 빈 글이 없음을 뜻하는 `String`이다(`clone_blocked`, `vec_missing`). `match m { "push" => (1, true, Unit), .. }`처럼 이름으로 튜플을 고르던 표는 변수 셋에 `if`의 사슬로 대입한다. 클로저 `place_ty`(두 번 부르고 `self`와 `cx`를 잡는다)는 메서드 `mem_place`다
- C4: `4D-03e6b`(메서드 호출): `method`, `method_target`, `bind_method`, `int_method`, `shared_receiver`, `unknown_method`, `not_a_method`, `method_call`. 정의 22개. `bmb check`의 첫 진단 2개(E0303: 한 함수에서 `t`가 타입 둘, E0504: 루프 불변식에서 실행 함수 `Table::len`을 부름 — 길이를 `let`으로 먼저 읽는다), `bmb verify` 첫 회에 전부 증명. 표면이 바꾼 꼴: stage0의 `Option<Option<MethodTarget>>`은 변형 셋의 enum `Found`다. 수신자의 타입을 가리던 클로저 `own`(두 가지)은 `owns(t, owner, by_self)`이고, 그래서 타입의 impl과 trait의 메서드를 한 메서드 `bind_method`가 묶는다. `BTreeMap`을 돌며 `strip_prefix`로 가리던 것은 표의 열쇠를 마디로 나눠 견준다. `sort` + `dedup`은 `sorted_set`이다. `m.starts_with("checked_")`는 이름을 늘어놓은 `checked_arith`다
- C4: `4D-03e6c`(글, `check/text.rs`): `text_arg`, `text_call`, `text_method`, `char_method`, `as_bytes_arg`. 정의 15개. `bmb check`의 첫 진단 0개, `bmb verify` 첫 회에 전부 증명. 표면이 바꾼 꼴: 가드 붙은 `match` 갈래(`Method { .. } if name == "as_str"`, `_ if str_slice_of(a).is_some()`)의 사슬은 `Option`을 돌려주는 풀이 함수(`as_str_of`, `slice_of`, `ref_var`)마다의 `match`와 이른 `return`이다. `&s[a..b]`의 세 겹 `let .. else`는 함수 셋(`slice_of`·`slice_index`·`range_of`)이다. `args[0]`을 읽는 자리는 인자 수를 견준 뒤에도 `args.len() > 0`을 다시 적는다(이름의 글 견줌에서 인자 수가 따라 나오지 않는다)
- C4: `4D-03e7a`(변환과 참조 인자): `conversion`, `ref_arg`, `slice_of_vec`, `e0218`, `data_ref_arg`, `sub_slice_arg`. 정의 13개. `bmb check`의 첫 진단 2개(둘 다 E0303: 한 함수에서 이름 하나가 타입 둘), `bmb verify` 첫 회에 전부 증명. 표면이 바꾼 꼴: stage0의 `Option<Option<String>>`(`slice_of_vec`)은 구조체 `VecSlice { about, root }`다. 튜플을 묶는 `let (var, explicit_ref, am) = match .. { 갈래마다 이른 return }`은 변수 둘에 대입하는 `match`와 이름을 푸는 `match`로 나뉜다. 클로저 `vec_var`는 자유 함수다. 별칭 진단 `self.diags.aliases`는 `Body`의 필드 `aliases`다
- C4: `4D-03e7b`(호출과 타입 인자): `call`, `call_arg`, `written_targs`, `targs_fix`, `arg_peek`, `place_peek`, `dispatch`, `reads_only`. 정의 31개. `bmb check`의 첫 진단 8개(E0303 넷: 이름 하나에 타입 둘, E0414 둘: 한 길에서만 옮긴 값, E0103: 변수 이름으로 쓴 예약어 `proof`와 그 여파 하나). `bmb verify` 첫 회에 2개 실패, 둘 다 솔버의 자원 한도였다: `call`(stage0 357줄을 한 함수로 옮긴 것, 의무 102개, z3가 메모리로 끝남)은 `call_kind`·`call_arg`·`call_deferred`·`call_bound`·`call_record`·`alias_check`로 나눠 풀렸고, `call_arg`는 `&&`의 오른쪽에서 `&mut self` 메서드를 부른 꼴(`.. && !self.deref_fix(..)`)이 원인이었다(백로그 4D-24, 호출을 조건 밖으로 빼서 풀림). 표면이 바꾼 꼴: `(mode, sig.kind)` 튜플의 `match` 아홉 갈래는 `if`의 사슬이다. 루프 안의 `continue`는 인자 하나를 맡는 메서드의 이른 `return`이다. 슬라이스 무늬(`[]`, `[t]`, `[t, ..]`)는 세어 둔 수의 비교다. `BTreeMap<String, TypeKind>`는 `types::Subst`와 `sub_put`이다
- C4: `4D-03e8a`(전역 검사): `check_recursion`, `trait_cycles`, `check_main`, `sccs_with_cycles`, `defs_of`. 정의 29개. `bmb check`의 첫 진단 7개(E0606 셋: `Table::find`를 부르는 함수의 `pure` 선언, E0410 둘: `t.index.insert(v, t.next)`처럼 `&mut` 수신자의 필드를 인자로 읽음, E0414 둘). `bmb verify` 첫 회에 전부 증명. 표면이 바꾼 꼴: 함수 안의 지역 구조체와 `impl`(Tarjan의 `T`)은 단위의 구조체 `Scc`와 자유 함수 `scc_visit`다. `BTreeSet`의 순서로 이웃을 보던 것은 볼 때마다 `decls::sorted`로 정렬한다. `while let Some(x) = work.pop()`은 끝남을 보일 연료가 든 `while`과 한 걸음을 맡는 메서드(`instance_edges`)다. `Vec<Vec<String>>`의 `sort`는 성분이 겹치지 않으므로 첫 원소의 순서(`by_first`)다. `type_depth`는 넘침 의무 때문에 결과가 연료를 넘지 않는다는 `ensures`를 갖는다
- C4: `4D-03e8b`(구동·표·대조): `check_program`, `check_resolved`, `ConstTrees`(stage0 `consts::literal_tree`), `Methodize`, `may_untrusted`, `lib`의 `push_tables`·`check`. 정의 24개. `bmb check`의 첫 진단 6개(E0303, E0401 넷: `for i in 0..10`의 `i`가 `i64`라 `usize`와 섞임, E0414). `bmb verify` 첫 회에 3개 실패(`key_part`와 `last_of`의 넘침·사후조건, `may_untrusted`의 루프 불변식: `&mut`로 넘긴 `Vec`의 길이가 줄지 않음을 호출 뒤에 알 수 없어 더할 것을 따로 모았다). stage0과의 첫 출력 대조(추적되는 `.bmb` 936개의 `dump types`)는 715개가 같았고 195개에서 stage1이 멈췄다: stage0의 결함 `4D-34`(갈래가 묶은 값의 부분 이동 뒤 이중 해제). 고친 뒤 남은 차이는 앞 조각의 누락 둘이었다: `#[uses(<T as Tr>::m)]`의 풀이(`decls::uses_target`), 상수 검사가 평가기의 진단을 빼는 자리(stage0은 검사기가 앞 단계의 진단 목록 위에서 돈다). 증명은 이 둘을 볼 수 없었다(명세가 없는 동작). 표면이 바꾼 꼴: 클로저 셋(`fresh`, `id`, `segs`)과 `&mut` 둘을 받는 `literal_tree`는 상태를 든 구조체 `ConstTrees`의 메서드다(`&self.adts`와 `&mut self.types`를 한 호출에 넘기면 E0410). `loop { .. return .. }`은 연료가 든 `while`이다. `rsplit_once`는 자리를 찾는 `last_of`와 `lexer::cut`이다
- C4: `4D-03e9`(`moves`, `selfhost/moves.bmb`): stage0 `moves.rs` 1,164줄이 1,491줄, 정의 66개. `bmb check`의 첫 진단 9개(E0103 둘: 필드 이름으로 쓴 예약어 `spec`, E0414 여섯: 조건 안에서만 표에 넣는 값, E0303 하나). `bmb verify` 첫 회에 2개 실패: `moves::jump`(루프의 색인 범위, 상태의 사본을 돌게 해서 풀림), 그리고 고치지 않은 `body::Body::expr_inner`가 솔버 한도를 넘었다(`Body`에 필드 하나가 늘자 서른다섯 갈래의 `match` 끝에서 타입 불변식의 의무가 한도 500만의 73%에서 102%로). 놓는 자리를 `Body`의 새 필드 대신 `Needs` 안에 담아도 넘어서, `expr_inner`를 식의 종류로 가른 두 메서드(`expr_early`·`expr_late`)로 나눴다(각각 한도의 42%, 50%). 표면이 바꾼 꼴: 참조를 든 구조체 `Mv<'a>`는 값을 든 `Mv`와 읽기만 하는 `Env`로 나뉘고, 식의 타입 표와 데이터 타입 표는 `std::mem::replace`로 빌려줬다가 돌려받는다(복사하지 않는다). `BTreeMap<String, St>`는 이름의 순서로 둔 두 `Vec`(`State`)이다: 열쇠를 지울 수 있어야 해서 `table::Table`을 쓰지 못했다. `enum St`와 `Option<St>`는 `u8` 0·1·2다. 가드 붙은 갈래(`Field { .. } if 지역 필드`)는 `local_field`가 돌려주는 `Option`의 `match`다. stage0과의 첫 대조에서 차이 0
- C4: `4D-03f3`(`check::methodize`, `selfhost/methodize.bmb`): stage0 약 190줄이 정의 16개. 클로저 `&mut |e| {..}`가 잡던 것(단위의 경로, 정의의 자리, 남길 호출, 고쳐 쓴 호출의 표)은 손님 구조체 `Rewrite`의 필드다: 검사기의 표도 참조로 들 수 없어 되풀이마다 사본을 든다. `loop`는 걸음의 한도(`fuel`)를 둔 `while`이다(남기는 집합이 커지는 동안만 돌지만 그것을 측도로 쓰지 못했다). `std::mem::take(args)`로 옮기던 인자는 사본이다. 가드 붙은 갈래 둘(`Ref { expr, .. } if by_ref`, `k if own(k)`)은 서른다섯·열일곱 갈래의 `match`와 `Option`이다. `bmb check`의 첫 진단 5개(E0303, E0410: `self.conv.insert(.., self.fk.clone())`, E0414 둘, E0413: `*x.clone()`으로 상자에서 꺼내기), `bmb verify`는 첫 회에 전부 통과(1,501개). stage0과의 첫 대조에서 차이 0
- C8: 닫히지 않은 행이다: 낮춘 표, 타입 배치, 모듈의 뼈대, 스칼라 식과 문장(`4D-05a`~`4D-05c2`)뿐이고 `dump lowered`·`dump layout`과 허용 목록의 `emit-llvm --all-checks`를 견준다. 옮긴 것은 stage0의 `check/text.rs`의 `lower`와 `verify.rs`의 `Tables::lowered`·`struct_fields`·`enum_variants`·`value_ty`라서(C4·C6의 파일) 배율은 C8과 견줘지지 않는다. `gate_runs`는 포트 전체의 `bmb verify` 실행 수다(`4D-05a`·`4D-05b`·`4D-38`·`4D-05c1`·`4D-05c2` 1번씩, 모두 첫 회에 전부 증명)
- C8: 상수 IR 글을 들 수단이 없다(`include_str!`·`const`의 긴 글): `scripts/codegen_text.py`가 stage0의 `bmb dump runtime`에서 `selfhost/codegen_text.bmb`를 만들고 게이트가 신선도를 본다: 우회, `std_units`와 같은 까닭
- C8: `Gen`이 표를 든 참조를 필드로 가질 수 없어(참조는 2급) 표(`&lowered::Tables`, `&body::Checked`)를 메서드마다 매개변수로 넘긴다: 그대로 둠
- C8: 한 함수 안에서 이름 하나는 타입 하나라(E0303) 갈래가 35개인 `expr`의 묶는 이름을 갈래마다 달리 지었다(`uop`·`ux`, `bop`·`lhs`·`rhs`, `ifx`): 같은 마찰이 세 번째다(`4D-05a`, `4D-05c1`). 그대로 둠, 언어의 선택이지만 큰 `match`에서 비용이 는다
- C8: `old`는 예약어라 묶는 이름으로 쓸 수 없다(E0103 `expected binding or `_``): 진단이 예약어라고 말하지 않는다. 이름을 바꿨다(`prev`): 우회, 진단의 글은 고칠 후보
- C8: `u64`의 배치 산술(`size.div_ceil(a) * a + s`)은 넘침 의무가 생긴다: stage0의 릴리스 빌드처럼 감기는 연산(`wrapping_add`·`wrapping_mul`·`wrapping_sub`)으로 쓰고 0으로 나눔은 갈래로 막았다(`round_up`): 그대로 둠
- C8: 와일드카드 갈래가 없어 `TypeKind`의 17갈래를 세 곳에 더 쓴다(`lower_in`·`struct_fields`·`enum_variants`): 그대로 둠, 언어의 선택(ADR-0026)
- C8: 한 함수 안에서 이름 하나는 타입 하나다(E0303): `match`의 두 갈래가 같은 이름으로 다른 타입을 묶을 수 없어 이름을 나눴다(`fs`·`sf`, `vs`·`ev`): 그대로 둠

<!-- ledger:data end -->

## 읽는 법

- 관문 B0은 C0~C2 행으로 전체 추정을 다시 낸다(ADR-0116 결정 2): 배율이 포팅 지도 §7.3의 범위(3.5~5.7)를 넘는지, C2의 증명된 정의 비율이 절반에 못 미치는지, 미결정으로 끝난 실행이 게이트 실행의 과반인지(ADR-0122).
- 관문 C는 E1 범위의 증명된 정의 비율을 함께 보고한다(ADR-0111).
- 4D 태스크는 그 구성 요소의 행이 채워져야 완료다.

# ADR-0074: 트랜잭션의 `put`·`remove`는 단위의 모든 항목을 주소로 쓴다(타입, `impl` 함수, trait, trait impl)

- 상태: 제안 (초안 작성 승인 2026-10-31 "권장안 채택", 백로그 T0-23의 §10 권장안. 채택은 따로 승인: AUTONOMY §4, 동결 집합이 보는 도구 설명과 측정 조건에 닿는다)
- 날짜: 2026-10-31
- 관련: DESIGN.md §6(쓰기는 트랜잭션), ADR-0009(트랜잭션), ADR-0011(배치는 도구가), ADR-0021(동결), ADR-0026 §3·§6(`impl` 블록은 타입당 하나, 정규형), ADR-0046 §2(trait impl의 일관성과 단위), ADR-0059 §1·§5(정체성, `rename`·`expect`), ADR-0060(타입 불변식), ADR-0070(같은 조건), ADR-0073(편집 표면), 백로그 T0-20·T0-23·T0-09·T0-21·T1-20, JUDGMENT §10

## 맥락

- 트랜잭션은 함수만 쓴다. `Op::Put`(`compiler/src/workspace.rs`)은 `code`를 `mod`로 싸서 파싱한 뒤 `m.fns`만 단위에 넣고, 그 밖의 항목이 있으면 `non_fn_items`가 이름을 대어 트랜잭션 전체를 거부한다(T0-20 (1)). `Op::Remove`도 `m.fns`에서만 찾는다. 함수는 `fn`·`spec fn`·`proof fn`(모두 `FnDef`)이다.
- 그래서 에이전트는 타입을 더하거나 필드·변형·`#[invariant(p)]`(ADR-0060)를 고칠 수 없고, `impl` 메서드·trait·trait impl을 쓸 수 없다. MCP만으로는 데이터 타입이 있는 단위를 만들 수 없다. T0-09(`bench/brb-encap`)는 시드를 파일로 깔고, 새 연산을 메서드가 아닌 단위의 함수로 받는다(`prompt-bmb.md`의 "it cannot write types or `impl` blocks, so a new operation is a function of the unit"). Rust 팔은 `impl` 메서드다.
- 같은 트랜잭션의 다른 연산은 이 항목들을 이미 주소로 다룬다. `rename`은 `unit::f`·`unit::T::f`·`unit::T`·`unit::Trait`를 받는다(`TargetKind`의 `Fn`·`ImplFn`·`Type`·`Trait`, trait 메서드는 거부). `expect`는 `identities`의 모든 열쇠를 받는다(`identity::Key`의 `Def`·`Type`·`Trait`). `Def`에는 `impl` 함수 `unit::T::f`, trait 기본 본문 `unit::Trait::m`, trait impl 함수 `<unit::T as unit::Trait>::f`(`check::TraitImplInfo::fn_path`)가 있다. 맵과 `verify`의 상태 줄도 같은 경로를 쓴다.
- 언어와 컴파일러는 이 항목들을 이미 검사하고 증명한다(ADR-0026·0046·0060). 편집 표면(ADR-0073)의 `edit_file`·`edit_ast`도 파일 전체를 바꾸므로 타입·`impl`·trait를 쓸 수 있다. 정의 단위 표면만 함수에 머물렀다.

## 결정

### 1. 항목과 주소

단위의 항목 다섯 종류를 주소로 쓰고 지운다. 주소는 맵·`expect`·`rename`이 이미 쓰는 경로다. 단위 안에서 타입·trait·함수·하위 단위는 한 이름공간이라(검사기 E0302) 경로 하나는 항목 하나다.

| 항목 | 주소 | `put` | `remove` |
| --- | --- | --- | --- |
| 함수(`fn`·`spec fn`·`proof fn`) | `unit::f` | 같은 이름의 함수를 바꾸거나 더한다(지금 규칙) | 그 함수 |
| 타입(`struct`·`enum`, 속성 포함) | `unit::T` | 같은 이름의 타입 선언 전체(필드·변형·타입 매개변수·속성)를 바꾸거나 더한다. `impl` 블록은 건드리지 않는다 | 타입과 그 고유 `impl` 블록(`unit::T::*`) |
| 고유 `impl` 함수 | `unit::T::f` | `impl T { fn f .. }`의 함수마다 T의 `impl` 블록에 병합(아래 2) | 그 함수. 블록의 마지막 함수면 블록도 |
| trait(메서드 시그니처·계약·기본 본문) | `unit::Trait` | 같은 이름의 trait 선언 전체를 바꾸거나 더한다 | trait와 그 기본 본문(`unit::Trait::*`) |
| trait impl | `<unit::T as unit::Trait>` | 같은 (trait, 타입) 쌍의 impl 블록 전체를 바꾸거나 더한다 | 그 impl 블록 |

- trait impl의 주소는 그 함수 경로의 앞부분이다(`TraitImplInfo::fn_path`가 `::f` 앞에 쓰는 것). 병합 때는 검사 전이므로 같은 규칙을 구문으로 계산한다. trait 경로가 한 마디이고 대상 단위에 그 이름의 trait가 있으면 `unit::Trait`, 아니면 쓴 그대로(전체 경로, 내장 `Ord`·`Eq`·`Copy`)다(`identity::Ids::trait_path`와 같은 규칙). 타입은 대상 단위가 선언한 이름을 전체 경로로 쓴다(`fmt::type_hashes`의 `qualify`와 같은 규칙). 타입 매개변수는 쓴 이름 그대로다. 그래서 매개변수 이름만 다른 impl은 다른 주소가 되고, 둘이 함께 남아 검사기가 겹침(E0311)으로 거부한다.
- 항목의 위치 규칙은 검사기가 지킨다. `impl T`는 T의 단위에만, 타입당 하나(E0308). trait impl은 trait나 타입의 단위에만(E0312), 한 쌍에 하나(E0311). 다른 단위의 trait impl을 옮기려면 같은 트랜잭션에서 `remove`하고 `put`한다.

### 2. `put`의 병합 규칙

- `code`의 항목을 원문 순서로 하나씩 적용한다. 같은 주소가 두 번 나오면 뒤의 것이 남는다(지금 함수 규칙과 같다: `put` 둘을 이어 쓴 것과 같은 뜻).
- 함수·타입·trait·trait impl은 **같은 주소의 항목을 통째로 바꾼다**. trait impl을 함수 단위로 병합하지 않는 이유: impl은 trait의 메서드를 빠짐없이 가져야 하는 한 항목이고(E0315), 정체성에서도 trait와 타입의 텍스트에 함께 들어간다(`identity.rs` 머리말 "Nominal types and traits").
- 고유 `impl`은 항목이 아니라 이름공간이다(ADR-0026 §3: 경로는 `unit::T::f`). 그래서 `impl T { fn f .. }`는 **함수마다** 병합한다. T의 블록에 `f`가 있으면 바꾸고 없으면 더한다. 블록의 다른 함수는 그대로다. 블록이 없으면 만든다. 블록 머리(타입 매개변수와 경계)는 `put`의 머리가 된다. 바뀐 머리에서 다른 함수가 검사를 통과하지 못하면 그 함수의 진단으로 거부된다(타입 선언을 바꿔 메서드가 깨지는 것과 같은 길).
- 다른 종류의 같은 이름(함수 `foo`가 있는 단위에 `struct foo`)은 바꾸지 않고 더한다. 검사기가 E0302로 거부한다. 종류가 다른 항목을 말없이 지우지 않는다.
- 커밋 규칙은 그대로다(ADR-0009). 적용이 끝난 프로그램에 구조·타입 오류가 하나라도 있으면 전체를 거부하고 아무것도 쓰지 않는다. 증명은 요건이 아니다. 저장은 정규형이라(ADR-0026 §6: 타입 뒤에 그 `impl` 블록, `impl` 안의 함수는 이름순) `code` 안의 순서는 결과에 남지 않는다.

### 3. `remove`가 받는 것

- 1의 표의 주소 다섯 꼴. 주소 P를 지우면 P의 항목과 경로가 P로 시작하는 항목(`unit::T`면 `unit::T::*`)이 함께 간다. trait impl `<unit::T as ..>`는 `unit::T`로 시작하지 않으므로 남는다. 남은 impl이 없는 타입이나 trait를 가리키면 검사기가 거부한다(E0305). 지우려면 같은 트랜잭션에서 그 주소도 지운다.
- 받지 않는 것: trait 메서드 `unit::Trait::m`과 trait impl 함수 `<..>::f`(trait나 impl의 일부다. 고친 항목 전체를 `put`한다. `rename`이 trait 메서드를 받지 않는 것과 같다), 단위 경로(`remove_unit`). 문제 문자열로 거부하고 무엇을 대신 쓰는지 말한다.
- `write`의 `remove` 배열도 같은 주소를 받는다.

### 4. 거부로 남는 것

- **`use`.** 언어 밖이다(E0214). `put`에 별도 거부를 두지 않고, 항목과 함께 단위에 넣어 검사기가 파일에서와 같은 E0214로 거부하게 한다. 편집 표면(`plan_files`)과 같은 규칙, 같은 진단이다. 그 밖에 `non_fn_items`의 T0-20 거부는 없어진다.
- **하위 단위(`mod`).** 단위는 `unit` 연산이 만든다. 지금처럼 거부한다("create units with a `unit` op").
- 빈 `impl T { }`(함수 없는 고유 블록)는 쓸 것이 없으므로 "`code` contains no definition"과 같은 꼴로 거부한다.

### 5. 커밋 줄과 불변식 변경

- `changed`·`removed`는 `fmt::qualified_hashes`의 행을 비교한다. 지금 행은 정의(`check::defs_of`: 함수, `impl` 함수, trait impl 함수, trait 기본 본문)와 타입 선언(`type_hashes`, 속성 포함: `type_def_as`가 속성 줄을 먼저 찍는다)이다. 여기에 trait 선언 `unit::Trait`와 trait impl 머리 `<unit::T as unit::Trait>`의 행을 더한다. 그래야 `put`이 쓰는 모든 항목이 커밋 줄에 나온다. 동결 집합의 과제·시드·해법에는 trait·trait impl이 없다(`bench/` 아래 `.bmb`·`.json`·`.py`의 `trait`·`impl .. for` grep 0건, 2026-10-31). 그래서 새 행은 세션이 스스로 trait를 쓸 때만 보인다.
- **불변식 변경.** 타입의 `#[invariant(p)]`를 더하거나 빼거나 다른 술어로 바꾸면 타입 선언의 텍스트가 바뀌어 `changed`에 `unit::T`가 나온다. 술어 `p`의 본문만 바꾸면 `changed`에는 `p`만 나오지만, 타입의 `id`가 바뀐다(ADR-0060 §1: 속성은 술어를 `id`로 적는다). 이것은 `expect unit::T`(옛 `id`면 거부)와 맵 L1의 `id`에 드러난다. 전이적 영향은 지금처럼 `impact`가 답한다. trait impl을 바꾸면 그 trait와 타입의 `id`도 바뀐다(정체성 텍스트가 impl을 담는다). H-ENCAP 판정의 "불변식 변경 = 타입 `id`가 시드와 다름"(`bench/brb-encap/README.md`)은 이 정의와 같다.

### 6. 동결 집합 보호: 동작은 모두에게, 새 설명은 플래그로

- **동작.** 위의 `put`·`remove`는 모든 서버와 CLI(`bmb apply`)에서 같다. 플래그에 따라 뜻이 갈리는 `put`을 두지 않는다. 지금 성공하는 입력의 응답은 그대로다(함수만 담은 `put`, 함수 주소의 `remove`). 응답이 바뀌는 것은 오늘 거부되는 입력뿐이다: 함수 밖의 항목을 담은 `put`(T0-20의 거부 → 커밋)과 새 주소 꼴의 `remove`("no definition" → 삭제). T0-20의 훑기에서 동결 집합의 기준·약한·게이밍 해법은 그런 `put`을 보내지 않았다(`code` 1,136개, BRB 모듈 59개, 0건). 구현 때 새 `remove` 주소에도 같은 훑기를 한다.
- **설명.** `tools/list`의 바이트는 플래그가 없으면 지금과 같다. 새 플래그 `--item-docs`가 있을 때만 `tx_put`·`tx_remove`·`write`의 설명(`--tool-docs`의 두 꼴 모두)이 다섯 항목과 주소를 말한다. 동결 판은 자기 `set.json`의 `mcp_flags`(없으면 플래그 없음)로 서버를 띄우므로 설명 바이트가 그대로다. 새 판은 `mcp_flags`에 `--item-docs`를 적는다. `initialize`의 `instructions`는 함수를 말하지 않으므로 바꾸지 않는다. 문구 초안: `tx_put` "In a transaction: add or replace items in UNIT. A function, type (with its attributes), trait or trait impl replaces the one at the same path; the functions of an `impl T { .. }` block are added to or replace those of T's `impl` block.", `tx_remove` "In a transaction: remove the item at PATH: `unit::f`, `unit::T` (with its `impl` block), `unit::T::f`, `unit::Trait` or `<unit::T as unit::Trait>`." 확정은 구현 때이고 시험이 고정한다.
- **보호를 시험으로.** 동결 판이 쓰는 플래그 조합(없음, `--write-tool`, `--write-tool --tool-docs`, `--file-tools`, `--ast-tools`)마다 `tools/list` 결과의 SHA-256을 구현 전 커밋에서 재어 고정하는 시험을 더한다. 지금은 `write`·`--tool-docs` 시험이 도구 이름과 일부 문구만 본다(`tests/mcp.rs`).
- **동결 판의 프롬프트.** m1·m1b·BVB·BRB와 쓰기 판의 프롬프트는 `tx_put`·`write`가 "definitions"를 받는다고만 해서 거짓이 되지 않는다. `bench/brb-encap/prompt-bmb.md`만 "it cannot write types or `impl` blocks"라고 한다. 그래서 T0-21은 이 구현 전의 컴파일러 트리에서 돈다(ADR-0070 결정 1 (a). `run_models.py`에 넣으면 `--same-as`로).
- **기본값 전환**(플래그 없이 새 설명)은 이 ADR 밖이다. 옛 설명으로 동결된 판을 계획이 더 쓰지 않게 되면 승인 안건으로 올린다.

### 7. 시험

- `tests/workspace.rs`: `impl` 함수의 `put`(더함, 바꿈, 블록의 다른 함수 보존, 블록 생성, 머리 교체와 그로 깨지는 함수의 진단), 타입 `put`(필드 변경과 메서드의 진단, `#[invariant(p)]` 추가·교체 → `changed`에 타입, 술어 본문 교체 → 타입 `id` 변경과 옛 `id`의 `expect` 거부), trait·trait impl `put`(쌍 단위 교체, 메서드 누락 E0315, 다른 단위로 옮기기는 `remove`와 함께, 매개변수 이름만 다른 impl은 E0311), 한 `code`에 다섯 종류를 섞은 `put`(모두 커밋, `changed`가 모두 나열), `remove`의 다섯 주소와 연쇄(`unit::T` → `unit::T::*`), 남은 trait impl의 거부(E0305), 받지 않는 주소(`unit::Trait::m`, `<..>::f`, 단위)의 문구, `use`의 E0214, 하위 단위의 거부. 지금의 `put_rejects_items_it_cannot_write`는 이것들로 바뀐다.
- 왕복: 코퍼스의 프로그램마다 빈 작업 공간에 `unit`과 항목별 `put`으로 쓴 결과가 `bmb import`의 정규형 파일과 같은 바이트다(MCP만으로 데이터 타입이 있는 단위를 만들 수 있다는 것의 시험). trait impl 주소의 구문 계산이 검사기의 `fn_path` 앞부분과 같다는 것도 이 코퍼스로 본다.
- `tests/mcp.rs`: 6의 바이트 고정 시험, `--item-docs`의 새 설명, `tx_put`·`write`로 타입과 메서드를 쓰고 증명하기. `tests/mcp_fuzz.rs`: 코드 생성기에 타입·`impl`·trait·trait impl 항목과 새 `remove` 주소를 더하고, 모형(커밋만 작업 공간을 바꾸고 커밋 뒤 검사 통과·정규형)을 그대로 지킨다.

### 8. T0-09 우회의 해제: `brb-encap`의 새 판

- 채택·구현 뒤 `bench/brb-encap-b`를 만들어 첫 실행 전에 동결한다(ADR-0021 §1). BMB 팔의 새 연산은 Rust 팔처럼 타입의 `impl` 메서드다(`Set::replace(&mut s, ..)`). 프롬프트의 "cannot write types or `impl` blocks" 문장이 빠지고, `mcp_flags`에 `--item-docs`가 들어간다. 숨은 판정 단위는 메서드를 부른다. 기준·약한·게이밍 해법도 메서드로 다시 쓴다. 게이트 작업 넷(`oracle`·`weak`·`adequacy`·`tests`)은 같은 기준으로 돈다. 반증 조건은 `brb-encap`의 것을 실행 전에 다시 등록한다.
- 동결 판 `brb-encap`은 그대로이고 T0-21이 잰다. 두 판의 결과는 섞지 않는다(ADR-0021 §1).

## 근거

- **원인과 층(JUDGMENT §3):** 결정론적으로 할 수 있는 쓰기를 도구 표면이 막는다. 층은 도구다. 언어·컴파일러는 이미 이 항목들을 검사·증명하고, 쓰기 표면만 M1 부분집합에 머물렀다.
- **정석(S1):** 정의 단위 편집(ADR-0009)은 단위의 모든 항목을 주소로 쓰고 지운다. Rust에서 모듈은 항목(`fn`·`struct`·`enum`·`impl`·`trait`·`use`)의 열이다(Rust Reference "Items"). 구조 편집 도구(rust-analyzer의 항목 단위 편집)도 항목을 단위로 다룬다. trait impl을 `<T as Trait>`로 부르는 것은 Rust의 정규 경로(qualified path) 표기다.
- **후보(T0-23의 §10 권장안):** (A) `put`·`remove`가 모든 항목을 받는다. (B) 항목 종류마다 새 연산(`put_type`, `put_impl`, ..)을 둔다. 한 가지 일에 두 방법이고 도구 수가 는다(S3·S4). (C) 현상 유지, 타입·`impl`은 편집 표면(ADR-0073)으로 쓴다. 그 표면은 측정용 변형이고 기본으로 꺼져 있다. 트랜잭션 표면의 결함을 다른 표면으로 흡수한다(§2.1). 셋 다 C1~C4를 통과한다. A를 고른다. 결정적 기준은 S3(같은 트랜잭션의 `rename`·`expect`·정체성이 쓰는 주소를 `put`·`remove`도 쓴다)와 S2(항목 종류 전체를 한 규칙으로)다.
- **고유 `impl`은 함수마다, trait impl은 통째로:** 앞의 것은 이름공간이라 함수가 각자 정의다(경로 `unit::T::f`, 정체성의 `Def`). 뒤의 것은 trait 하나를 구현하는 한 항목이다(E0315, 정체성의 impl 텍스트). 주소를 가진 것만 바꾼다는 한 규칙의 두 경우다(S3).
- **동결 보호의 대안:** (가) 동작은 모두에게, 새 설명은 플래그와 새 판에서(이 결정). (나) 플래그가 동작까지 가른다. 동결 판이 새 트리에서 돌 때 모든 입력의 응답이 같지만, 같은 연산에 뜻이 둘이 되고(S3) 기본 서버에 결함이 남는다(§2.1). 동결 판이 받는 혜택도 없다. 그 과제는 함수만 쓴다. 트리 안의 비교(원판 대 변형 판)는 두 쪽이 같은 동작을 보므로 공정하다(ADR-0070: 컴파일러 트리가 바뀌면 어차피 새 조건). (다) 모든 판을 새 설명으로 다시 동결한다. 타입을 쓰지 않는 과제의 조건(매 요청에 들어가는 설명 바이트. T0-17이 재는 고정 문맥이다)을 바꾸고, 진행 중·대기 중인 비교(T0-10·T0-11의 원판, T0-13·T0-15·T0-17·T1-41의 원판 대 변형)의 기준선을 끊는다. 얻는 것은 없다(C3). 결정적 기준은 C3(동결 조건은 모델이 보는 바이트)와 S3이다. 플래그로 설명 꼴을 고르는 것은 `--write-tool`·`--tool-docs`·`--file-tools`·`--ast-tools`와 같은 기존 장치다(S3).
- **`use`를 검사기에 맡기는 이유:** `put`만의 거부 문구를 두면 같은 규칙이 두 곳에서 두 문구로 나온다. 파일·편집 표면과 같은 E0214 하나로 둔다(S3).

## 결과

- 바뀌는 곳(정보): `workspace.rs`의 `Op::Put`·`Op::Remove`(`non_fn_items` 삭제), `fmt::qualified_hashes`(trait·trait impl 행), `mcp.rs`의 `FLAGS`와 `tools`(`--item-docs`), `compiler/README.md`의 `apply`·MCP 줄, 6·7의 시험. DESIGN.md는 바뀌지 않는다(§6 "쓰기는 트랜잭션"의 연산이 받는 항목만 넓어진다).
- 측정: 동결 판의 설명 바이트와 지금 성공하는 입력의 응답은 그대로다. 오늘 거부되는 입력만 커밋된다. T0-21은 구현 전 트리에서 돈다(6).
- 열린 것: MCP의 `rename`·`expect`(T1-20)는 따로다. 같은 새 판에서 열면 설명 바이트가 한 번에 바뀐다. trait 메서드의 `rename`도 여기서 넓히지 않는다.
- 채택 뒤 할 일: T0-23 구현(1~7), `brb-encap-b` 구축·동결(8, 새 태스크).
- 되돌리기: `Op::Put`이 함수 밖의 항목을 다시 거부하고, `Op::Remove`가 `m.fns`만 보고, `--item-docs`를 지운다. 플래그 없는 설명은 이 결정과 무관하게 같은 바이트다.

# ADR-0092: 표준 라이브러리 핵심

- 상태: 채택 (2026-11-06, 사람 승인으로 4B-01 완료. 초안은 4B-03). 점검 R12대로 제네릭(ADR-0046)·소유 타입(ADR-0047)·문자열(ADR-0050) 위에서 쓴다: 4A-01 시점의 단형 맵 전제는 철회
- 날짜: 2026-11-05
- 관련: ADR-0031(`Vec`), ADR-0046, ADR-0047, ADR-0050, ADR-0091(시스템 인터페이스), `dev-docs/measurements/selfhost-gap.md` §5.1, 백로그 2U-07, ROADMAP "언어 전체 수순" ④

## 구현 시 확인 (2026-11-08, 4B-02 (a))
- **싣기:** `lib/std/NAME.bmb`마다 단위 `mod NAME` 하나. `compiler/build.rs`가 파일 목록을 만들어 `include_str!`로 컴파일러에 넣고, 같은 파일을 컴파일러 정체성(증분 캐시 키 `BMB_SOURCE_DIGEST`)에 더한다. 설치 위치나 경로 인자가 없다.
- **더하기(`compiler/src/stdlib.rs`):** 프로그램이 경로 `std::NAME::..`로 부르는 단위(호출·타입·패턴·경계·`#[uses]`, `refs::walk`)와 그 단위가 부르는 단위를 최상위 단위 `std` 아래에 프로그램의 단위 뒤에 더한다. 파일 이름은 `std/NAME.bmb`이고 소스 지도에서 프로그램 파일 뒤라, 프로그램 파일의 위치는 호출자의 파일 목록 번호 그대로다. 부르지 않는 단위는 더하지 않는다(열린 질문 1의 첫 판: 검사도 증명도 하지 않는다). 더한 단위는 다른 정의처럼 검사·증명·코드 생성된다.
- **이름:** 프로그램의 최상위 단위 `std`는 E0309(내장 이름)다.
- **`lib/` 이전(4B-02 (d), 결과 절 셋째 항):** 다섯 파일을 `lib/std/`로 옮겼다(내용 그대로, 단위 이름도). 부르는 쪽(`tests/lib_run`, `bench/game`의 둘, `bench/lib`의 둘)은 `bytes::f` → `std::bytes::f`로 고치고 파일을 넘기지 않는다. 출력은 같다. 이름 하나는 §2 표와 다르게 했다: SHA-256은 `std::hash`가 아니라 `std::sha256`이다. Rust의 `std::hash`는 `Hash`·`Hasher` trait(해시 표용)이라 같은 이름에 다른 뜻이 되고, §2의 원칙(Rust의 이름과 뜻)과 어긋난다. `base64`·`crc32`는 표에 없던 것을 그 이름 그대로 `std::base64`·`std::crc32`로 두었다.
- **`std::str`(4B-02 (b)):** `starts_with`·`ends_with`·`find`·`strip_prefix`·`strip_suffix`·`trim_ascii_start`·`trim_ascii_end`·`decode_utf8`. 조각을 돌려줄 수 없어(ADR-0015) Rust가 조각을 돌려주는 곳은 위치를 돌려준다(`strip_prefix`는 `&s[i..]`의 `i`). 계약은 바이트 명세와 정확히 같다: `find`는 `std::bytes::found`(첫 위치, 없으면 어디에도 없음; 그 전의 `std::bytes::find`는 `Some`만 실제 위치라고 했고 처음도 `None`의 뜻도 말하지 않아 이번에 정확히 했다), `decode_utf8`은 표 3-7의 `std::utf8::seq_len`과 스칼라 값(선형 식 `is_code`), 그 밖은 바이트 술어. 돌려받은 위치에서 조각내는 쪽은 실행 검사가 없다(2U-06의 경계 사실과 계약). `trim`은 Rust의 유니코드 공백이 아니라 ASCII 공백만 하는 Rust의 `trim_ascii_*` 이름으로 둔다(유니코드 `trim`은 수요가 나올 때). `split_once`는 위치를 돌려주면 `find`와 같아 두지 않는다. `decode_utf8`의 결과는 구조체 `std::str::Decoded { ch, next }`(BMB에 튜플이 없다). 메서드 구문: 텍스트 수신자의 이름이 내장 메서드가 아니고 `std::str`의 함수면 그 함수다(`s.starts_with(p)` → `std::str::starts_with(s, p)`, `String`은 `s.as_str()`로 넘김: 조정 `Adjust::AsStr`), 문자열 리터럴 수신자도(`"a=b".find("=")`). 메서드 이름만으로도 `std::str`이 더해진다(`stdlib::METHOD_UNITS`). 모르는 텍스트 메서드의 안내(E0404)가 `std::str`의 함수를 들고 Rust의 `trim`·`contains`·`split_once`를 위 함수로 옮기는 법을 말한다. 시험: 실행 `tests/run/std_str`(Rust 번역 `std_str_ref.rs`와 출력 같음, valgrind), 차등 `tests/stdlib.rs`의 `std_str_agrees_with_rust_str`(무작위 텍스트 3,000줄, 전부 증명된 BMB 프로그램 `tests/data/std_str_diff.bmb`, `BMB_SOUND_ITERS`로 늘림). 증명 비용: 가장 비싼 정의 `decode_utf8` rlimit 460,423(최대 의무 37,521, 기본 한도 5,000,000의 1/5 안).
- **`std::num`(4B-02 (c)):** 읽기 `parse_u64`·`parse_i64`·`parse_u64_hex`(`&str`, 바이트판 `*_bytes`)는 Rust의 `from_str`·`from_str_radix(s, 16)`과 정확히 같다: 앞의 `+`(부호 있는 것은 `-`도) 하나, 숫자 하나 이상, 넘치면 `None`, `None`은 정확히 그 밖의 입력(넘침은 `dec_grows`로 "숫자가 늘면 값이 줄지 않는다"를 써서 증명). 쓰기 `push_u64`·`push_i64`·`push_u64_hex`(소문자)는 `String`에 Rust의 `Display`·`{:x}`와 같은 텍스트를 덧붙인다. 계약은 정규형 술어 `is_dec(t, a, n, x)`: 길이가 `width(x)`(상수 19개 문턱, 선형)이고 `t[k]`가 `x`의 끝에서 `n-1-k`째 자리(`dig`, 상수 나눗셈만이라 선형). 재귀를 바이트가 아니라 자릿수 번호에 두어 덧붙임 뒤에도 앞 바이트가 그대로라는 사실이 따로 필요 없다(바이트 위의 재귀 술어는 덧붙이기 전 바이트에 이름이 없어 틀 보조 정리를 쓸 수 없었다). 되읽힘은 보조 정리 `u64_round_trip`·`u64_hex_round_trip`(쓴 텍스트를 읽으면 `Some(x)`). 이 단위를 쓰다 검증기의 `String` 길이 결함(백로그 1B-55)을 찾아 고쳤다. 시험: 실행 `tests/run/std_num`(Rust 번역과 출력 같음), 차등 `tests/stdlib.rs`의 `std_num_agrees_with_rust`(경계 값·부호·앞의 0·섞인 바이트 3,000줄, 읽은 값을 다시 써서 되읽힘까지 대조).
- **`std::slice`(4B-02 (e)):** §2 표의 `sort_by_key`는 키를 꺼내는 클로저를 받는데 BMB에는 클로저가 없다(ADR-0053은 승인 대기). 그래서 키의 안정 정렬 순서를 위치의 열로 돌려주는 `sort_indices<T: Ord>(keys: &[T]) -> Vec<usize>`를 두었다: `v[r[0]], v[r[1]], ..`이 Rust의 `v.sort_by_key(..)` 결과다(같은 키는 들어온 순서). 값을 옮기지 않으므로 이동 타입의 `Vec`에도 쓰인다. 계약 `sorted_order`는 길이, 범위, 모든 쌍의 안정 순서(`before`: 작은 키, 같으면 작은 위치)이고, 안정 순서가 같은 위치를 두 번 담을 수 없으니 순열이다(보조 정리 `sorted_is_perm`). 다중 집합 대신 이 꼴을 골라 증명이 선형 사실과 쌍 정량만 쓴다(삽입 정렬, 루프 불변식은 "옮기는 위치만 빼고 안정 순서"). 클로저가 들어오면 `sort_by_key`를 그 위에 둔다. `binary_search<T: Copy + Ord>`는 Rust의 뜻(`Ok(i)`는 같은 값의 어느 위치든, `Err(i)`는 유일한 끼울 자리)이고 전제는 모든 쌍의 오름차순(`ascending`). `T: Ord`는 지금 정수와 `bool`뿐이다(ADR-0046 열린 질문 4). 시험: 실행 `tests/run/std_slice`(같은 키의 순서, 찾음·끼울 자리, Rust 번역과 같음), 차등 `tests/stdlib.rs`의 `std_slice_agrees_with_rust`(반복이 많은 무작위 키 1,500줄, Rust의 안정 `sort_by_key`와 `binary_search`와 대조). 증명 비용: `sort_indices` 438,638(최대 의무 41,437).
- **`std::collections`(4B-02 (f)):** §2 표대로 정렬된 `Vec` 위의 `BTreeSet<K: Copy + Ord>`와 `BTreeMap<K: Copy + Ord, V: Copy>`(값의 `Vec`을 나란히). 타입 불변식은 모든 쌍에서 키가 오름차순(그래서 유일)이고 두 `Vec`의 길이가 같다. 위치는 이분 탐색 `locate`(키의 자리: 앞은 작고 뒤는 작지 않음). 넣기·빼기의 계약은 두 겹이다: 배열이 어떻게 바뀌는지(`inserted_at`·`removed_at`·`replaced`: 자리 `p`에 끼움, 뺌, 바꿈)와 맵의 뜻(`maps`·`has_key`·`holds`: `get`은 `Some(v)`면 키가 `v`에 대응하고 `None`이면 키가 없다), 그리고 부르는 쪽이 싸게 쓰는 크기 사실(`len`, `map_grew` 등). §2 표의 "정렬된 쌍의 열과 그 위의 lookup 술어"다(열린 질문 2의 수학적 맵은 아직). 밀기는 집합 밖의 `Vec`에서 한다(`std::mem::replace`로 꺼내 밀고 되돌림): 루프 머리마다 타입 불변식이 서야 하는데 밀기 중간에는 키가 겹친다. 값을 복사(`V: Copy`)로 둔 것은 `get`이 참조를 돌려줄 수 없어서(ADR-0015)이고, 키가 `Ord`인 것은 지금 정수와 `bool`뿐이다(`String` 키는 ADR-0046 열린 질문 4와 함께). 이 단위를 쓰려고 타입 매개변수의 `Vec<T>`(백로그 2T-10)를 열었다. 시험: 실행 `tests/run/std_collections`(횟수 세기·서로 다른 값, Rust 번역과 같음, valgrind), 차등 `tests/stdlib.rs`의 `std_collections_agree_with_rust`(작은 키 범위의 무작위 넣기·바꾸기·빼기·찾기 2,000번과 끝의 내용을 Rust의 `BTreeMap`·`BTreeSet`과 대조). 증명 비용: 단위 2,366,602, 최대 의무 73,787(`BTreeMap::insert`).
- **텍스트 키(4B-04, 2026-11-08):** §2 표의 `K: Ord`대로 키가 이동 값이어도 된다: `BTreeSet<K: Ord>`·`BTreeMap<K: Ord, V: Copy>`, 키 `String`(ADR-0046 구현 시 확인의 내장 `Ord`). 조회는 Rust처럼 빌린 키다(`get(&k)`·`contains(&k)`·`contains_key(&k)`·`remove(&k)`). 옮기는 키는 `v[j] = v[j - 1]`로 밀 수 없어서 재귀 함수 `shift_in`(Rust의 `Vec::insert`)과 `shift_out`(`Vec::remove`, 꺼낸 것을 `Some`에)이 `std::mem::replace`로 하나씩 옮긴다. 루프가 아니라 재귀인 것은 이미 옮긴 키를 루프 불변식에서 말할 수 없기 때문이다(이동 검사는 명세의 언급도 사용으로 본다). `ensures`는 들어온 값으로 말하므로 재귀의 계약이 그 자리를 맡는다. 정렬 보조 정리에는 키 대신 그 자리의 원소(`&v[p]`)를 넘긴다. 순서대로 꺼내기는 소비하는 `into_keys`(맵·집합)와 `into_values`(맵)다: 자리의 키를 돌려주는 `key_at`·`at`은 키가 `Copy`일 때만 뜻이 있는데, 경계가 다른 두 번째 `impl` 블록은 ADR-0026이 금하고(E0308) `where` 절은 없다(E0240). 그래서 `key_at`·`at`을 뺐다. 값은 `Copy`로 남는다(`get`이 값을 복사해 돌려준다). 옮기는 값은 그 자신의 `Vec`에 두고 맵은 위치를 값으로 갖는다(Rust의 아레나 관용). 두 마찰(경계가 다른 `impl` 블록, 명세에서 옮긴 값 말하기)은 사람의 결정으로 남긴다(백로그 2T-11·2T-12). 증명 비용이 줄었다: 단위 2,366,602 → 1,354,174, 최대 의무 73,787 → 58,395(`BTreeMap::insert`). 시험: 실행 `tests/run/std_text_keys`(낱말 세기, `"B" < "a" < "ab" < "b" < "é"`, Rust 번역과 같음, valgrind), 차등 `std_collections_text_keys_agree_with_rust`(접두·대소문자·2~4바이트 글자·빈 텍스트 키 20개의 무작위 연산을 Rust와 대조). 리터럴을 빌려 넘기는 `s.contains(&7)`의 코드 생성(임시에 담음)도 이때 고쳤다.
- **`std::intern`(4B-02 (g), 2026-11-08):** §2 표의 인턴 표. rustc의 `Symbol` 표처럼 텍스트에서 번호로의 맵(`BTreeMap<String, usize>`)과 번호 순서의 이름(`Vec<String>`)이고, 타입 불변식은 맵의 모든 쌍에서 그 번호의 이름이 그 키다(`consistent`). `intern(&mut self, s: &str) -> usize`(처음이면 다음 번호, 계약은 그 번호가 `s`를 가리킴), `get`(없으면 어떤 키도 `s`가 아님), `name_is`, 이름을 빌려 줄 수 없어(ADR-0015) `push_name(i, &mut out)`(rustc의 `as_str`), `len`, `into_names`. 불변식 보존은 제네릭 보조 정리 `insert_keeps<K: Ord>` 하나다: 정렬 위에서는 키 같음이 SMT의 `=`라서 33,270에 증명되고, `String` 인스턴스로 부른다(텍스트의 같음은 길이 안 바이트의 정량식이라 `String`에 바로 쓰면 배열 이론이 미결정이었다). 이 단위를 쓰다가 `proof fn`의 매개변수 사실(E0819)이 `Vec<String>` 필드에서 바깥 길이를 안쪽 길이의 배열로 쓰던 인코더 결함(E0812, 잘못된 정렬의 질의)을 고쳤다. 시험: 실행 `tests/run/std_intern`(Rust 번역과 같음, valgrind), 차등 `std_intern_agrees_with_rust`. 증명 비용: 단위 약 44만, 최대 의무 67,419(`intern`).
- **`std::json`(4B-02 (h), T4-05 합침, 2026-11-08):** §2 표의 JSON 읽기·쓰기. 값의 트리 대신 노드 배열(simdjson의 테이프)이다: `Node { kind, start, end, next }`(종류 7가지, 텍스트의 바이트 범위, 그 노드와 그것이 담은 모든 것 다음의 노드), 배열의 원소와 객체의 키·값(키는 값 앞의 문자열 노드)이 순서대로 뒤따른다. 참조를 돌려줄 수 없어(ADR-0015) `Doc`는 텍스트를 담지 않고 읽는 함수가 텍스트를 받는다(전제 `of_text`: 같은 길이). 타입 불변식 `nodes_in`(모든 노드가 텍스트 안, `next`가 앞으로)이 읽는 쪽의 경계를 모두 증명한다. `parse`는 한 상태 기계이고 열린 배열·객체는 쌓기(`Vec<usize>`)라 중첩 깊이가 기계의 스택이 아니다(JSONTestSuite의 10만 겹 배열). 문자열은 RFC 8259 §7(32 미만 바이트 없음, 이스케이프 여덟과 `\\u` 네 자리, 나머지는 `std::utf8::width`로 UTF-8), 숫자는 §6. 읽기 `count`·`kind`·`start`·`end`·`next`·`element`·`member`(키는 따옴표 사이 바이트 그대로 비교, `raw_is`)·`i64_value`(`std::num::parse_i64_bytes`)·`string_value`(이스케이프 해독, 짝 없는 서로게이트는 `serde_json`처럼 거부). 쓰기 `push_string`은 `serde_json`의 규칙(`"`·`\\`·`\\b`·`\\f`·`\\n`·`\\r`·`\\t`, 그 밖의 제어 바이트는 `\\u00xx` 소문자)이고 계약은 길이의 범위다. 숫자 쓰기는 `std::num`의 `push_u64`·`push_i64`. 판정의 정확함은 증명이 아니라 시험이 맡는다(§3의 "전부 증명"은 모든 의무: 경계·넘침·종료): JSONTestSuite(고정 커밋 `1ef36fa`, `scripts/json_suite.py`, 저장소에 넣지 않고 받아 씀)의 `y_` 95개는 토큰이 Python `json`과 같고 `n_` 188개는 모두 거부, `i_` 35개는 21 받음·14 거부(`dev-docs/measurements/json-suite/`). 차등 `std_json_agrees_with_rust`: Rust가 만든 무작위 값을 무작위 이스케이프·공백으로 써서 토큰을 대조(기본 200개, 2,000개도 같음). 증명 비용: 단위 4,835,818, 최대 의무 485,775(`push_string`).
- **도구:** 정규형(`bmb fmt`·`import`)·작업 공간 파일·커밋의 `changed`는 프로그램 단위만 담는다. 트랜잭션의 `std` 경로 연산은 거부(`std` is the standard library). `fix`·`uses`·린트·`mutate`는 프로그램 파일·정의만 본다. 맵 L0·L1 행에 `"library":"std"`(있을 때만 붙는 키, ADR-0020 §1). 시험 `compiler/tests/stdlib.rs`, 실행 `tests/run/std_str`, 검사 `tests/check/err/std_unit`.

## 맥락

- 지금 있는 라이브러리는 `lib/`의 다섯(`bytes`·`utf8`·`sha256`·`base64`·`crc32`)이고, 쓰는 프로그램이 파일로 함께 넘긴다. 모두 전부 증명되고 공인 벡터를 통과한다(T4-01~04).
- 컴파일러 포팅(4D)에 필요한 라이브러리는 격차 분석 §5.1에 있다: 바이트 버퍼, 정수 서식·해석(10진·16진), 바이트 문자열 키의 정렬된 맵·집합(stage0의 `BTreeMap` 147곳·`BTreeSet` 82곳), 키별 안정 정렬(`sort*` 26곳), SHA-256, JSON 쓰기·읽기, UTF-8, 인턴 표. 2U-05의 실행 프로그램은 문자열 연산(`starts_with`·`find`·`split`·`trim`·`parse`, 문자 순회)이 없어 바이트와 `while`로 썼다(2U-07).

## 결정

1. **BMB로 쓰고 증명한 소스 라이브러리.** 표준 라이브러리는 컴파일러에 내장된 함수가 아니라 BMB 원문이다(`lib/std/`). 증명 게이트가 다른 코드와 똑같이 확인하고, 탈출구는 0이다(ADR-0091의 시스템 내장은 라이브러리가 아니다). 컴파일러가 프로그램에 자동으로 더한다(`std::` 경로가 이 단위들로 풀린다). 지금처럼 파일을 넘기는 방식은 사라진다.
2. **범위(첫 판).** 셀프호스팅에 필요한 것부터, Rust의 이름과 뜻으로:

   | 단위 | 내용 | 계약 스타일 |
   | --- | --- | --- |
   | `std::str` | `starts_with`·`ends_with`·`find`·`split_once`·`trim`·`strip_prefix`, 문자 순회 `decode_utf8(s, i) -> Option<(char, usize)>`(ADR-0050 열린 질문 2) | 결과가 바이트 명세와 정확히 같다(`lib/bytes.bmb`의 `eq_result`·`found` 꼴) |
   | `std::num` | `u64`·`i64`의 10진·16진 `parse`와 서식(`push_u64` 등의 일반화) | 되읽힘(`parse(format(x)) == Some(x)`)과 정규형 |
   | `std::collections` | `BTreeMap<K: Ord, V>`·`BTreeSet<K: Ord>`: `insert`·`get`·`remove`·`contains`·순서대로 꺼내기 | 명세는 정렬된 쌍의 열(키 유일, 오름차순)과 그 위의 `lookup` 술어. 구현은 정렬된 `Vec`(첫 판, 이진 탐색)이고, 측정이 요구하면 균형 트리로 |
   | `std::slice` | 안정 정렬 `sort_by_key`(삽입 → 병합), 이진 탐색 | 결과가 입력의 순열이고 정렬됨, 같은 키의 순서 보존 |
   | `std::json` | 값 타입, 쓰기(이스케이프), 읽기(JSONTestSuite `y_`·`n_`, T4-05와 같은 일) | 쓰기는 문법에 맞고 되읽힘 |
   | `std::hash` | `lib/sha256.bmb`를 옮김 | FIPS 180-4 명세(지금 그대로) |
   | `std::intern` | 이름 → 손잡이 표 | 같은 이름은 같은 손잡이, 다른 이름은 다른 손잡이 |

3. **증명 비용 예산.** 라이브러리 정의마다 가장 비싼 의무의 `rlimit`이 기본 한도의 1/5 이하(V5와 같은 여유), 단위 전체가 `stability --seeds 8`에서 불안정 0. 넘는 정의는 보조 정리(`proof fn`)로 나누거나 계약을 약하게 하지 않고 구현을 바꾼다. 쓰는 쪽의 비용은 계약만 읽으므로(계약 방화벽) 라이브러리 몸에 묶이지 않는다.
4. **동결과 바꾸기.** 라이브러리의 공개 계약은 코퍼스와 같은 규칙으로 바꾼다: 약하게 하면 `bmb weaken`의 정렬이 쓰는 쪽을 다시 증명한다(2D-19). 측정 동결 판(ADR-0021)은 그때의 라이브러리를 함께 동결한다.

## 근거(검토한 대안)

| 안 | 판정 |
| --- | --- |
| **BMB 소스 + 증명(채택)** | 미션의 실증(증명된 코드가 테스트를 대체)이 라이브러리에서 시작한다. 셀프호스팅 컴파일러가 같은 라이브러리를 쓴다 |
| 컴파일러 내장(지금의 `Vec` 메서드처럼) | 빠르지만 증명되지 않는 TCB가 커지고, 컴파일러 하나에 라이브러리 구현이 붙는다 |
| `lib/` 파일을 쓰는 쪽이 넘기는 지금 방식 | 이름 공간과 판(버전)이 없고, 도구(맵·질의)가 프로그램과 라이브러리를 구분하지 못한다 |
| 해시 맵 | 순회 순서가 결정적이지 않다(stage0도 쓰지 않는다, 격차 분석 §3.1) |

## 결과

- 4B-02 구현은 `std::str`·`std::num` → `std::slice` → `std::collections` → `std::json` 순서(2U-07, T4-05와 합침). 단위마다 공인 벡터 또는 Rust 차등 시험, 실행 프로그램, 퍼저 대상.
- 맵과 질의는 `std::` 단위를 프로그램 단위와 구분해 보여준다(쓰기 도구는 `std::`를 고치지 않는다).
- `lib/`의 다섯은 `std::`로 옮기고 시험(`tests/lib_run`)은 이름만 바뀐다.

## 열린 질문

1. 자동으로 더하기의 비용: 쓰지 않는 단위를 증명하지 않으려면 단위별 증명 캐시를 컴파일러와 함께 싣는다(5C 배포와 함께).
2. `BTreeMap`의 명세 보기: 쌍의 열 대신 수학적 맵(`Map<K, V>`, Verus의 보기)을 명세 언어에 넣을지. 지금 명세 언어에는 열(슬라이스)만 있다.
3. 메서드 구문(2M) 전에는 `std::str::starts_with(s, p)` 꼴로만 부른다.

# ADR-0020: 맵 JSON 스키마 L0~L2

- 상태: 채택 (자율 채택, AUTONOMY.md §4)
- 날짜: 2026-10-24
- 관련: DESIGN.md 원칙 2·7, §7.1, ADR-0008·0016·0019, 백로그 1D-04

## 맥락
표준 탐색 맵 L0~L2는 명세에 고정되고, 이후 변경은 하위 호환만 허용한다(ADR-0008). 서드파티 도구가 의존할 수 있도록 형식을 정한다.

## 결정

### 1. 형식
- 맵과 질의 출력은 JSONL이다. 한 줄이 행 하나다. 키는 아래 표의 순서로 쓰고, 행은 `path`(또는 `from`, `to`) 순으로 정렬한다. 같은 코드면 같은 바이트가 나온다.
- 모든 맵 출력의 첫 줄은 머리 행 `{"schema":"bmb.map","version":1}`이다.
- **하위 호환 규칙:** 키를 추가할 수 있고, 새 `kind` 값을 추가할 수 있다. 기존 키의 이름·의미·타입은 바꾸지 않고, 키를 지우지 않는다. 이를 어기는 변경은 `version`을 올리고 새 ADR로 한다.
- 정의는 전체 경로(`unit::name`)로 가리킨다. 위치는 `file`과 `line`(1부터)이고, 파일 배치가 바뀌면 함께 바뀐다. 정체성은 경로와 해시가 맡는다.

### 2. L0 목차
| 키 | 값 |
| --- | --- |
| `level` | `"L0"` |
| `kind` | `"unit"` 또는 `"def"` |
| `path` | 단위 또는 정의의 전체 경로 |
| `summary` | 단위의 한 줄 요약(`unit`만) |
| `units`, `defs` | 직속 하위 단위·정의의 경로 목록(`unit`만) |
| `def_kind` | `"fn"` 또는 `"spec fn"`(`def`만) |
| `hash` | 본문 해시 앞 12자(`def`만, ADR-0019) |

### 3. L1 표면
정의마다 한 행이다.

| 키 | 값 |
| --- | --- |
| `level`, `path`, `def_kind` | L0과 같음 |
| `signature` | 정규형 시그니처 한 줄 |
| `requires`, `ensures` | 계약 절의 정규형 텍스트 목록 |
| `decreases` | 정규형 텍스트 또는 `null` |
| `effects` | `"pure"`·`"io"`, `spec fn`은 `"spec"` |
| `uses` | 펼치는 `spec fn`의 전체 경로 목록 |
| `status` | `"verified"`, `"failed"`, `"none"`(의무 없음) |
| `obligations`, `failed` | 의무 수, 실패 수 |
| `body`, `iface` | 해시 전체(ADR-0019) |
| `file`, `line` | 정의 이름의 위치 |

### 4. L2 관계
엣지마다 한 행이다. 모든 엣지는 해석이 끝난 전체 경로를 잇는다.

| `kind` | 의미 |
| --- | --- |
| `calls` | 실행 코드에서의 호출(내장 `print_i64` 포함) |
| `contract` | `from`의 `requires`·`ensures`·`decreases`·불변식·`assert`가 술어 `to`를 씀 |
| `uses` | `from`이 `#[uses]`로 `to`를 펼침 |
| `spec_calls` | `spec fn` 본문에서의 호출 |
| `writes` | `from`이 `to`에 `&mut` 인자를 넘김(`arg`에 인자 위치) |

키: `level`(`"L2"`), `from`, `to`, `kind`, 그리고 `writes`에만 `arg`.

### 5. 질의(ADR-0016의 7개)
`bmb query <질의> [인자] <파일>...`로 실행하고, 결과는 위 행 형식을 따른다.

| 질의 | 결과 |
| --- | --- |
| `units` | L0 전체 |
| `surface DEF` | DEF의 L1 행 |
| `edges DEF [out\|in\|both] [깊이]` | DEF에서 시작하는 L2 엣지(기본 `out`, 깊이 1). 깊이만큼 전이 |
| `slice DEF` | `{"path","code","deps":[{"path","text"}],"bytes","approx_tokens"}`. `code`는 DEF의 정규형, `deps`는 DEF가 부르는 정의의 계약면과 펼치는 `spec fn`의 정의 |
| `find NAME` | 이름의 마지막 부분이나 전체 경로가 NAME인 정의의 L0 행 |
| `contracts PRED` | `{"pred","required_by","ensured_by","uses_by"}` |
| `status` | 정의별 `{"path","status","obligations","failed","effects"}` 행과 요약 행 `{"summary":true,"defs","verified","failed","io_defs"}` |

`approx_tokens`는 `ceil(bytes / 4)`의 근사다. 정확한 토큰 단위는 측정 절차 ADR(1E-01)에서 정한다.

## 근거
- 행 단위 JSONL은 부분만 읽고 버리기 쉽고, 행 추가만으로 확장된다.
- 맵에 코드를 넣지 않고(`slice`만 예외로 정규형을 돌려준다) 위치와 해시로 가리킨다(DESIGN.md §7.1).

## 결과
- L1의 `status`를 채우려면 증명을 돌려야 한다. 맵 도출 비용에 증명 비용이 포함된다. 증명 캐시(2단계)로 줄인다.

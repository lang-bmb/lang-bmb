# 셀프호스팅 포팅 구성 요소 지도 (백로그 4A-02)

> 상태: 설계 초안. 포팅 순서·동결 범위의 결정은 4D-01(ADR, 승인)이 한다.
> 근거: 커밋 `1e1f24f`의 `compiler/`, `bench/`, `tests/`. 크기는 `dev-docs/measurements/selfhost-gap-stats.py`. 기능 격차는 `selfhost-gap.md`(4A-01).
> 규칙: ADR-0037(stage0 → stage1 → stage2 = stage3, 포팅 착수 시 stage0 기능 동결, 구성 요소마다 차등 시험).

## 0. 요약

1. 구성 요소는 14개로 나눈다(§1). 가장 큰 것은 증명기 인코딩(2,203줄), 검사기(1,805줄), 파서(1,728줄)다.
2. **렉서와 파서는 지금 CLI로 관찰할 수 없다.** `fmt`·`hash`·`check`는 모두 검사기까지 돈다(`lib.rs:75-94`의 `frontend`). 구성 요소별 차등 시험을 하려면 동결 전에 stage0에 관찰 명령(`bmb dump tokens|parse`)을 넣어야 한다(§6).
3. **조기 고정점이 가능하다.** 코드 생성은 `--all-checks`(증명 없이 모든 의무를 실행 시 검사)로 증명기 없이 돈다. 기반 라이브러리·렉서·파서·정규형·검사기·코드 생성(약 5.8K줄, 42%)만 옮기면 그 부분 컴파일러가 자기 자신을 빌드해 stage2 = stage3을 잴 수 있다. 런타임·성능·결정론 위험을 증명기 포팅 전에 드러낸다(§3).
4. stage1을 만드는 데 자기 증명이 필요 없다. `bmb build`는 모든 의무가 증명되어야 하지만, `emit-llvm --all-checks` + clang 경로가 이미 있다. 그래서 4E(고정점)는 4F(자기 증명)를 기다리지 않는다.
5. 자기 증명의 미증명 면적은 주로 런타임 경계(외부 함수 15~25개)와 SHA-256 비트 연산(2~4개)으로 추정한다. 가장 큰 증명 위험은 개수가 아니라 아레나 인덱스 유효성(수량자 불변식)이다(§5).

## 1. 구성 요소

| ID | 구성 요소 | 파일(테스트 제외 줄) | 합계 | 관찰 가능한 출력 | BMB 추정 줄(배율) | 비용 |
| --- | --- | --- | --- | --- | --- | --- |
| C0 | 기반 라이브러리 | `sha256`(82), `json`(267), `diag`(306), `source`(67) + 4B 바이트·서식·맵·정렬 | 722 + 신규 | 해시 값, JSON 텍스트, JSONL 진단 형식, 줄·열 | 3~5K(라이브러리 신규 포함) | 상(맵·바이트가 새로 필요) |
| C1 | 렉서 | `lexer`(196) | 196 | 토큰 열(관찰 명령 필요), E01xx | 0.4~0.6K(2~3) | 하 |
| C2 | 파서·구문 트리 | `parser`(1,485), `ast`(243) | 1,728 | 구문 진단, 구문 트리(관찰 명령 필요) | 4~5K(2.5~3) | 상(`?` 85, `return` 46, 아레나 구문 트리) |
| C3 | 정규형·해시 | `fmt`(514) | 514 | `bmb fmt`, `bmb hash` | 1~1.5K(2~3) | 중 |
| C4 | 검사기 | `check`(1,805) | 1,805 | `bmb check` JSONL, `bmb codes`, `docs/diagnostics.md` | 4~5K(2.5) | 상 |
| C5 | 맵·질의 | `map`(956) | 956 | `bmb map`, `bmb query`(증명 없는 것: 1D-17 집합) | 2.5~3.5K(3, 반복자 63·클로저 53) | 중 |
| C6 | 증명 인코딩 | `verify`(2,203) | 2,203 | `verify --emit-smt DIR`의 스크립트 | 5~6K(2.5) | 상 |
| C7 | 해법기·도구 연결 | `smt`(337), `toolchain`(116) | 453 | `verify` 상태 줄·반례, `doctor`, `stability` | 1K + 런타임 | 상(대화형 파이프) |
| C8 | 코드 생성 | `codegen`(826) | 826 | `emit-llvm`(·`--all-checks`) IR, 실행 파일 동작 | 1.5~2K(2) | 중 |
| C9 | 캐시·증분 | `cache`(92), `incremental`(676) | 768 | `--cache` 있을 때·콜드·웜 출력(같은 바이트여야 함), 캐시 파일 | 1.5~2K(2.5) | 중 |
| C10 | 도구 | `lint`(752), `uses_fill`(592), `lib`(849: 전단·`fix`·`stability`·전단 캐시) | 2,193 | `lint`, `uses`, `mutate`, `fix`, `stability` JSONL | 5~6K(2.5) | 중 |
| C11 | 작업 공간·트랜잭션 | `workspace`(383) | 383 | `import`·`apply` 결과 디렉터리와 JSONL | 1K(2.5) | 중(파일 시스템) |
| C12 | MCP 서버 | `mcp`(448) | 448 | 요청 줄 → 응답 줄 | 1~1.5K(3) | 중 |
| C13 | CLI | `main`(652) | 652 | 모든 명령의 stdout·stderr·종료 코드 | 1.5K(2.5) | 하 |

- BMB 줄 추정은 `selfhost-gap.md` §5.4의 확장 요인을 구성 요소별 사용량에 맞춰 적용한 추정이다. 합계 약 30~40K줄. C1을 옮긴 뒤 실제 배율로 고친다.
- 구성 요소 경계는 지금 Rust 모듈 경계와 같다. 포팅 중 구조를 바꾸면 차등 시험의 대응이 흐려지므로, 경계는 그대로 두고 표현(아레나·바이트)만 바꾼다.

## 2. 의존 그래프

`compiler/src`의 `crate::모듈::` 참조에서 뽑았다(→는 "사용한다").

```
C0  source ← (없음) / sha256 ← (없음) / diag → source / json → diag
C1  lexer → diag, source
C2  parser → ast, diag, lexer, source ; ast → source
C4  check → ast, diag, source, verify(Tables 타입만: check.rs:130 into_tables)
C3  fmt → ast, check(FnSig, ty_str), sha256
C6  verify → ast, cache, check, diag, incremental, smt, source, toolchain
C9  incremental → ast, diag, fmt, sha256, source, toolchain, verify(FnStatus, ObKey, Tables) ; cache → sha256, toolchain
C7  smt → toolchain ; toolchain → (없음)
C8  codegen → ast, fmt, incremental, verify
C5  map → ast, check, diag, fmt, source, verify
C10 lint → ast, cache, diag, fmt, incremental, map, source, verify ; uses_fill → ast, cache, diag, incremental, source, verify
    lib → ast, cache, check, codegen, diag, fmt, incremental, lexer, map, parser, source, verify
C11 workspace → ast, check, diag, fmt, json, lexer, parser, source
C12 mcp → diag, json, toolchain, workspace, lib
C13 main → diag, lint, mcp, toolchain, uses_fill, workspace, lib
```

- **순환 둘:** `check ↔ verify`는 `verify::Tables`(검사기가 만든 표)라는 자료 타입 하나 때문이다. `verify ↔ incremental`은 `incremental::Session`(증명기가 받음)과 `verify::{FnStatus, ObKey, Tables}`(증분이 저장함) 때문이다. 둘 다 함수 호출이 아니라 타입 공유다. BMB에서는 공유 타입을 별도 단위(`tables`, `obkey`)로 빼면 순환이 없어진다. 출력은 바뀌지 않으므로 stage0에서 동결 전에 해 두면 대응이 쉬워진다(선택).
- `fmt → check`는 해시용 한정 정규형이 서명 표(`FnSig`)를 읽기 때문이다. 그래서 `bmb fmt`·`bmb hash`는 검사기를 통과한 프로그램에만 돈다.

## 3. 포팅 순서와 차등 시험

ADR-0037 §3의 순서(렉서→파서→검사기→증명기→코드 생성→맵·도구)를 기본으로, 관찰 가능성과 조기 고정점을 위해 둘을 바꾼다. **코드 생성을 증명기 앞으로**, **증명 없는 맵·질의를 증명기 앞으로** 둔다. 각 단계는 앞 단계의 BMB 구현 위에 쌓인다(stage0과 섞어 링크하지 않는다).

| 단계 | 옮기는 것 | 차등 시험(같은 입력 → 같은 바이트) | 시험 입력 |
| --- | --- | --- | --- |
| P0 | C0 기반 라이브러리, 4C 런타임 | SHA-256: 저장소의 모든 파일에 대해 `sha256sum`과 같은 값. JSON: MCP 세션 기록의 요청 줄을 읽고 다시 써서 stage0 `json` 왕복과 같은 텍스트. 서식: 정수 경계값(0, ±1, `i64::MIN`, `u64::MAX`)의 10진·16진 | 저장소 파일, MCP 퍼저 기록 |
| P1 | C1 렉서 | `bmb dump tokens FILE`(동결 전 추가, §6) JSONL과 종료 코드 | 코퍼스 전체 + 무작위 바이트·잘린 파일(퍼저 내보내기) |
| P2 | C2 파서 | `bmb dump parse FILE`(동결 전 추가): 구문 진단 + 구문 트리의 정규 출력 | 위와 같음 + `tests/nesting.rs` 깊이 경계(255·256·257) |
| P3 | C4 검사기, C3 정규형·해시, C13 일부(`check`·`fmt`·`hash`·`codes`) | `check` JSONL·종료 코드, `fmt --check`·`fmt` 출력, `hash` JSONL, `codes --md` = `docs/diagnostics.md` | `tests/check/**`의 기대 진단 `.jsonl` 17개는 stage와 무관한 골든이므로 그대로 쓴다. 퍼저 `fuzz.rs`·`fmt_fuzz.rs` 생성물 |
| P4 | C8 코드 생성(전체 검사 모드. 증명된 의무 집합이 비어 있으므로 `verify`의 `ObKey` 타입과 IR 캐시 없이 돈다) | `emit-llvm --all-checks` IR 바이트, 실행 파일의 stdout·종료 코드 | `tests/run`(6), `bench/baseline`(7), 건전성 퍼저 6종의 생성 프로그램 |
| **E1** | **조기 고정점** | BMB 컴파일러 소스(C0~C4·C8)를 stage0 `emit-llvm --all-checks` + clang으로 빌드(stage1) → stage1로 같은 소스(stage2) → stage2로(stage3). stage2 IR = stage3 IR. stage2가 P1~P4 시험을 통과 | 자기 소스 |
| P5 | C5 맵·질의(증명 없는 집합) | `map L0`·`map L2`, `query units|edges|slice|find|contracts|impact|duplicates|effects` | m3 생성기 작업 공간(N = 100·1,000·10,000, `bench/m3/gen.py`), m1·m1b 시드 |
| P6 | C6 증명 인코딩 | `verify --emit-smt DIR`의 정의별 스크립트 바이트 | `tests/verify/**`(12), m1b 시드, m3 N = 100·1,000, 건전성 퍼저 생성물 |
| P7 | C7 해법기 연결, 증명 게이트, `build` | `verify` 전체 출력(상태 줄의 `rlimit`·`rlimit_max` 포함, 반례 문구), `stability`, `emit-llvm`(증명된 의무 반영) IR | P6과 같음. 같은 Z3 바이너리에 같은 스크립트면 같은 답과 `rlimit`을 기대한다(의무마다 `(reset)`, ADR-0018). P7에서 확인한다 |
| P8 | C9 캐시·증분 | 캐시 없음·콜드·웜·편집 후 출력이 stage0 캐시 없음 출력과 같다. 캐시 파일은 컴파일러 정체성 줄만 다르다(정체성은 소스 해시라 stage마다 다름) | `tests/incremental.rs`·`tests/cache.rs` 시나리오를 CLI 스크립트로 |
| P9 | C10 도구 | `lint`, `uses`, `mutate`, `fix`(`--print`·`--write`) JSONL과 바뀐 파일 | `tests/run/search.bmb`, m1b 시드, `tests/lint.rs`·`uses_fill.rs`·`fix_cmd.rs` 입력 |
| P10 | C11 작업 공간 | `import`·`apply`의 출력 JSONL과 결과 디렉터리 트리(파일 이름·바이트) | `tests/workspace.rs` 연산 묶음, m1·m1b 시작 작업 공간 |
| P11 | C12 MCP, C13 나머지 | 기록된 세션을 두 서버에 넣고 응답 줄 비교 | `mcp_fuzz.rs` 세션(기록 내보내기), `mcp_cache.rs` 시나리오, 벤치 하네스의 oracle 에이전트 실행(`bench/m1`·`m1b`·`brb`) |
| **E2** | **전체 고정점(4E)** | stage2 = stage3(IR 바이트, 같은 clang이면 실행 파일도), stage2가 위 모든 시험과 `scripts/check.sh`의 벤치 작업을 통과 | 전부 |

차등 시험 하네스(4D에서 작성, 제안): `(명령, 인자, 입력 집합)` 목록마다 두 바이너리를 같은 작업 디렉터리·환경 변수로 실행하고 stdout·stderr·종료 코드·만든 파일을 바이트로 비교한다. 정규화는 캐시 파일의 컴파일러 정체성 하나만 둔다. 벤치 스크립트는 이미 바이너리를 바꿔 끼울 수 있다(`bench/m3/scale.py`의 `$BMB`).

### 3.1 시험 자산 현황

| 자산 | 양 | 차등 시험에서의 쓰임 |
| --- | --- | --- |
| BMB 코퍼스·벤치 원문 | `.bmb` 51개, 3,459줄 | 모든 단계의 기본 입력 |
| 기대 진단 | `.jsonl` 17개 | stage와 무관한 골든(P3) |
| 벤치 과제 | m1 15, m1b 37, brb 24, bnb 27 파일 | 기준 해법·약한 대조군 실행(P11) |
| 규모 생성기 | `bench/m3/gen.py`(정의 10,000개까지) | 맵·질의·증명 규모(P5~P7), 성능 비교 |
| 퍼저 | `fuzz.rs`, `fmt_fuzz.rs`, `mcp_fuzz.rs`, 건전성 6종 | 생성기가 Rust 테스트 안에 있다. 생성물을 파일로 내보내는 모드가 필요하다(테스트 코드 변경이라 동결과 무관) |
| Rust 통합 테스트 | 30개 파일, `#[test]` 113개 | 15개 파일은 CLI를 실행하고, 15개 파일(`corpus`, `fmt`, `fmt_fuzz`, `fuzz`, `gate`, `impact`, `incremental`, `map`, `mcp_cache`, `nesting`, `no_prove`, `stability`, `toolchain`, `verify`, `fixes`)은 라이브러리 API만 부른다. stage0 은퇴(4G) 뒤에도 쓰려면 CLI 기반으로 바꿔야 한다 |

## 4. 결정론 차이의 원천(차등 시험이 걸릴 곳)

| 원천 | 위치 | 대응 |
| --- | --- | --- |
| `io::Error` 문구(`strerror` + ` (os error N)`) | `{e}` 36곳 | BMB 런타임이 같은 형식으로 낸다(4C-01에 명시) |
| `f64` 서식 `{:.2}` | `lib.rs:346-376`(`stability` 요약의 `rlimit_spread_max`) | 동결 전에 stage0을 정수 계산으로 바꾼다(§6). Rust의 부동소수 반올림을 BMB로 재현하는 것보다 싸다 |
| `{:?}` 문자열 이스케이프 | `toolchain.rs:72`(Z3 버전 해석 실패 메시지) | 같은 이스케이프를 구현하거나 동결 전에 `json_str`로 바꾼다 |
| `from_utf8_lossy`(대체 문자) | 3곳(외부 도구 출력, JSON 숫자) | 라이브러리에 같은 규칙 |
| UTF-8 검증 실패(`read_to_string`) | 파일 읽기 | 같은 오류 문구 |
| 경로 정렬 | `workspace.rs:55`(`PathBuf` 정렬: 구성 요소 단위) | 같은 디렉터리의 파일 이름만 정렬하므로 바이트 사전순과 같다. 시험으로 확인 |
| 안정 정렬 | `sort*` 26곳 | BMB 정렬도 안정 정렬이어야 한다(같은 키의 순서) |
| 맵 순회 순서 | `BTreeMap`·`BTreeSet` | Rust `String`의 순서는 바이트 사전순이다. 바이트 문자열 비교로 같다 |
| 벽시계 | `smt.rs`의 재사용 창(60초) | 출력에 영향 없음(의무마다 `(reset)`). 시간 의존 경로는 차등 시험에서 판정에 쓰지 않는다 |
| 컴파일러 정체성 | `build.rs`의 `BMB_SOURCE_DIGEST` | 캐시 파일에만 들어간다. 정규화 대상 |
| 버전 문자열 | `mcp.rs:373`(`CARGO_PKG_VERSION`) | 상수로 옮긴다 |

## 5. 위험

### 5.1 성능

- 기준(stage0, 릴리스, vCPU 4): m1b 시드(실행 정의 53개, 의무 338개) `verify` 벽시계 1.6초(이 분석에서 1회 측정, 대부분 Z3). m3 N = 10,000에서 전단 캐시 없는 정의 단위 질의가 서버 CPU 약 0.5초, 콜드 `verify`가 CPU 1,297초(`bench/m3/results/2026-10-24*.md`).
- stage2가 느려질 원인: 정렬 배열 맵의 O(n) 삽입, 바이트 문자열 비교, 전체 검사 모드의 남은 검사(4F 전에는 모든 의무가 실행 시 검사), 아레나 간접 참조. 증명 시간은 Z3가 지배하므로 영향이 작고, 맵·질의·MCP처럼 전단이 지배하는 경로에서 드러난다.
- 제안 기준(4D-01에서 확정): m3 N = 10,000에서 stage2의 전단 질의 CPU ≤ stage0의 2배, `verify` 벽시계 ≤ 1.2배. E1(조기 고정점)에서 처음 잰다.
- MCP 서버는 오래 돈다. 아레나가 요청마다 자라면 메모리가 계속 는다. 요청 단위로 아레나를 비우는 구조가 필요하다(전단 캐시 3A-03과 함께 설계).

### 5.2 자기 증명 비용(4F)

- 규모 추정: BMB 함수 1,000~1,500개, 함수당 의무 15~30개(인덱스·넘침이 많다) → 의무 1.5만~4.5만 개. m3의 의무당 CPU(3.4~32.6ms, 규모에 따라 커짐)로 콜드 증명 CPU 10²~10³초대. 추정이며, 증분 캐시(2D-10)로 편집 뒤 재증명은 바뀐 정의에 비례한다.
- 개수보다 큰 위험은 **증명 가능성**이다.
  - **아레나 인덱스 유효성:** 구문 트리 노드의 자식 인덱스가 `nodes.len()`보다 작다는 사실을 함수마다 쓰려면 수량자 불변식(`forall i < len. child(i) < len`)이 계약에 들어간다. ADR-0023(기저 인스턴스 재시도)이 다루는 불안정의 전형이다. 대안: (1) 노드를 만들 때 자식 인덱스 < 부모 인덱스를 보장하고 그 단조성 하나만 불변식으로 쓴다 (2) 아레나 접근을 범위 검사가 붙은 함수로 감싸 `panic` 효과로 남긴다(미증명이 아니라 "검사 유지"로 집계) (3) 아레나 단위를 탈출구 하나로 신뢰한다. (1)을 기본으로 하고 E1 뒤에 실측으로 정한다.
  - **재귀 종료:** 구문 트리 재귀마다 `decreases`가 필요하다. 연료 매개변수(중첩 상한 256)를 쓰면 의무가 선형이다.
  - **정렬된 맵:** 정렬 배열이면 삽입의 순서 보존이 수량자 하나로 증명된다. 균형 트리는 증명 비용이 크다. 셀프호스팅에는 정렬 배열 + "다 만들고 정렬" 패턴을 권한다.
- **증명이 말하는 것의 범위:** 게이트가 증명하는 것은 넘침·범위·0 나눗셈 부재와 우리가 쓴 계약이다. "컴파일러가 옳다"는 증명되지 않는다. 계약 강도는 선택이고, 4F의 계약 범위(예: 렉서의 토큰 스팬이 원문 안, 서식기의 출력이 다시 같은 구문 트리로 파싱됨)는 4F ADR에서 정한다. 의미 정확성은 계속 차등 시험·퍼저가 맡는다.

### 5.3 비선형 산술과 비트 연산

| 곳 | 연산 | 정수 인코딩(ADR-0017)에서 | 대응 |
| --- | --- | --- | --- |
| SHA-256 압축 함수 | `^`, `&`, `!`, `rotate_right`, `>>`, 32비트 `wrapping_add` | 비트 연산은 선형 정수 산술로 표현되지 않는다 | 비트 연산을 언어에 넣지 않으면 `external_body`(kind `nonlinear`)로 2~4개 함수. 넣는다면 비트벡터 인코딩 의무가 별도 ADR |
| 서식(10진·16진) | `/ 10`, `% 10`, `/ 16` | 상수 제수라 Z3 선형 산술로 다뤄진다 | 증명 가능 |
| JSON `\u` 대리 쌍 | `0x10000 + ((hi − 0xD800) << 10) + (lo − 0xDC00)` | 상수 곱으로 바꾸면 선형 | 증명 가능 |
| 코드 생성 바이트 수 | `len * 8` 등 | 상수 곱 | 증명 가능 |
| 증명기·검사기의 정수 값 | 리터럴 범위 비교, 부정 | 선형 | 증명 가능. `u128`은 `u64` + 넘침 플래그로 |

변수끼리의 곱(진짜 비선형)은 컴파일러 코드에서 찾지 못했다(곱셈은 상수 곱과 문자열로 만든 SMT 텍스트 안에만 있다).

### 5.4 자기 증명 부담(탈출구가 필요한 곳) 추정

| 영역 | 정의 수(추정) | 탈출구 | `kind`(ADR-0029) | 근거 |
| --- | --- | --- | --- | --- |
| 런타임 경계: 파일·디렉터리·이름 바꾸기·삭제, 프로세스 시작·파이프 읽기·쓰기·대기, 인자, 환경 변수, 표준 입출력, 종료, 시계 | 15~25 | `external_body` | `boundary` | §3.4(`selfhost-gap.md`)의 시스템 인터페이스 전수 |
| SHA-256 압축 | 2~4 | `external_body` | `nonlinear` | 비트 연산(§5.3). 비트 연산이 언어에 들어오면 0 |
| 아레나 접근 | 0(방법 (1)) ~ 수십(방법 (2)는 탈출구가 아니라 `panic` 효과) | — | — | §5.2 |
| 맵·정렬·서식·JSON·UTF-8 | 0 | — | — | 선형 산술과 배열 수량자로 증명 가능(검증 필요) |
| 해법기 한계(`unknown`) | 측정 전 미정 | `assume` | `solver_limit` | E1 뒤 실측 |

- 합계 추정: 정의 20~30개, BMB 컴파일러 정의의 약 2~3%. 4F 통과 기준의 미증명 면적 상한 제안: 정의의 5% 이하, `kind`는 `boundary`·`nonlinear`만(값은 4F ADR이 정함).

### 5.5 그 밖

- **2단계 기능 의존:** `selfhost-gap.md` §5.3의 미배정 기능(바이트·문자열, 정렬된 맵, `Vec`을 담는 구조체, 시스템 인터페이스)이 들어오지 않으면 포팅이 시작되지 않거나 우회가 커진다. 4D 착수 조건으로 둔다.
- **v0의 이중 유지:** 동결 뒤 stage0 변경은 버그 수정과 부트스트랩 필수 변경뿐이다(ADR-0037 §3). 포팅에 유리한 stage0 정리는 동결 **전에** 끝낸다(§6). 동결 뒤 stage0의 버그 수정은 BMB 쪽에도 같은 커밋 묶음에서 반영해야 차등 시험이 깨지지 않는다. 그 비용이 v0의 패리티 비용과 같은 종류이므로 포팅 기간을 짧게 유지하는 것이 가장 큰 완화책이다.
- **스택:** stage0은 256MiB 스택 스레드에서 돈다. BMB 실행 파일은 기본 스택(보통 8MiB)이다. 연료로 깊이를 256에 묶고 `tests/nesting.rs`의 경계 입력으로 E1에서 확인한다.

## 6. 동결 전 stage0 정리(부트스트랩 준비, 4D-01 입력)

출력 바이트를 바꾸는 것은 기대 파일과 함께 바꾼다. 모두 언어 기능 변경이 아니다.

| 정리 | 이유 | 출력 변화 |
| --- | --- | --- |
| `bmb dump tokens FILE`, `bmb dump parse FILE` 추가 | 렉서·파서의 차등 시험 관찰점(P1·P2) | 새 명령만 |
| `stability`의 `f64` 제거(`{:.2}` → 정수 고정소수점) | 부동소수 서식 재현 회피 | `rlimit_spread_max` 표기 |
| `{:?}` 제거(`toolchain.rs:72`) | 디버그 이스케이프 재현 회피 | 오류 문구 하나 |
| 공유 타입 단위 분리(`verify::Tables`, `ObKey` 등) | 모듈 순환 제거 | 없음 |
| 퍼저 생성물 내보내기 모드 | 차등 시험 입력 | 없음(테스트 코드) |
| 라이브러리 API만 쓰는 통합 테스트 15개를 CLI 기반으로 | stage0 은퇴 뒤에도 같은 시험 | 없음(테스트 코드) |

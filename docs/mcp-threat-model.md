# `bmb mcp` 위협 모델

> 백로그 2D-20, ADR-0040 §3. 대상은 BMB 자신의 도구 표면(`bmb mcp DIR`, `compiler/src/mcp.rs`)이다. 에이전트 일반의 보안(모델이 주입된 지시를 따르는가, 공급망)은 언어의 보장 범위 밖이다(ADR-0040 §3, DESIGN.md 비목표).

## 경계

| 대상 | 신뢰 | 이유 |
| --- | --- | --- |
| 서버를 띄운 사람과 그 환경(`DIR` 인자, `PATH`의 `z3`·`clang`) | 신뢰 | 서버는 그 사람의 권한으로 돈다. 툴체인은 신뢰 기반이다(`dev-docs/HEALTH.md` §3) |
| MCP 클라이언트(에이전트)의 요청 | 부분 신뢰 | 에이전트는 작업 공간을 바꾸라고 받은 주체다. 다만 작업 공간 안의 글에 속아 엉뚱한 요청을 보낼 수 있으므로, 어떤 요청도 작업 공간 밖에 닿지 못해야 한다 |
| 작업 공간의 내용(`src/`, `.bmb-cache/`, `target/`) | 불신 | 남이 만든 저장소일 수 있다. 파일 이름·링크·단위 요약·정의 이름이 모두 입력이다 |

## 위협과 대응

| # | 위협 | 대응 | 시험 |
| --- | --- | --- | --- |
| T1 | 단위·정의 경로로 작업 공간 밖에 쓰기(`../x`) | 모든 경로 부분을 식별자 규칙으로 검사(1D-15) | `tests/workspace.rs`의 `invalid_paths_are_rejected_and_write_nothing` |
| T2 | 심볼릭 링크로 밖을 읽거나 쓰기: `src`·`.bmb-cache`·`target`이 링크, `src/*.bmb`가 링크, 미리 심은 임시 파일 링크 | 세 디렉터리와 단위 파일이 링크면 모든 도구가 거부한다(`workspace::refuse_symlink`). 임시 파일은 있던 것을 지우고 `create_new`로 연다(`workspace::write_new`, 증명·증분 캐시 파일도 같음). **2D-20 전에는 심은 임시 파일 링크를 따라가 대상 파일을 덮어썼다** | `tests/mcp.rs`의 `untrusted_workspace_cannot_reach_outside` |
| T3 | `build`의 출력 이름으로 밖에 쓰기 | 이름은 경로 구분자·NUL·선행 `.` 없는 평범한 파일 이름만(1D-19), 출력은 `target/` 아래 | `tool_errors_over_mcp` |
| T4 | 깊은 중첩으로 스택 넘침(서버 종료) | JSON `MAX_DEPTH` 256, 파서 `MAX_NESTING` 256, 256MiB 스택(1A-11) | `tests/nesting.rs`, MCP 200k `[` 시험 |
| T5 | 자원 고갈: 열린 트랜잭션마다 작업 공간 사본 | 열린 트랜잭션 `MAX_OPEN_TXS` = 64 | `untrusted_workspace_cannot_reach_outside` |
| T6 | 자원 고갈: 끝나지 않는 질의(`edges` 깊이 10^11) | 경계가 비면 멈춤(1D-14) | `tests/map.rs`의 `edges_with_a_huge_depth_terminate` |
| T7 | 자원 고갈: 해법기 시간 | 의무마다 고정 `rlimit`, 정의마다 `-T` 300초(1D-18) | `tests/gate.rs` |
| T8 | 응답 틀 깨기: 작업 공간의 글(요약)에 프로토콜 메시지처럼 보이는 문자열 | 모든 응답은 JSON 문자열로 이스케이프된 한 줄이다. 글은 데이터로만 나간다 | `untrusted_workspace_cannot_reach_outside`, `tests/mcp_fuzz.rs` |
| T9 | 잘못된 요청으로 서버가 멈추거나 죽음 | 객체가 아닌 요청에 -32600(1D-23), 임의 호출 순서 퍼징 | `tests/mcp_fuzz.rs` |
| T10 | 동시 작성자와의 경합으로 잃어버린 쓰기 | `tx_commit`은 `tx_begin` 뒤 작업 공간이 바뀌었으면 거부, 파일은 임시 파일 후 이름 바꾸기(1D-19) | `tests/workspace.rs`(1D-19의 원자적 쓰기), `untrusted_workspace_cannot_reach_outside`(동시 작성자) |

## 남는 위험(대응하지 않음)

- **지시 주입.** 단위 요약(`#[summary]`)은 자유 글이고 질의 응답으로 에이전트에게 간다. 서버는 그것을 데이터로 이스케이프할 뿐, 에이전트가 그 글을 지시로 따르는 것은 막지 못한다(범위 밖). 정의·단위 이름은 식별자로 제한되어 주입 통로가 좁다.
- **`verify` 한 번의 총 시간.** 정의 수만큼 해법기 시간이 든다. 상한은 정의마다만 있다. 클라이언트가 시간 제한을 둔다.
- **툴체인.** `build`는 `PATH`의 `clang`을, `verify`는 `z3`을 실행한다. 버전은 고정 검사(INF-02·03)를 받지만 실행 파일 자체는 신뢰한다.
- **작업 공간 밖의 링크가 아닌 경로.** `DIR` 자체나 그 위 디렉터리가 링크인 것은 서버를 띄운 사람의 선택이라 검사하지 않는다.

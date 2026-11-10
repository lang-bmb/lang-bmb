# BMB

BMB는 AI 에이전트가 쓰고, 결정론적 도구로 다루는 Rust 방언이다. 표면은 Rust·Verus를 승계하고, 의미는 더 엄격하다. 계약(`requires`·`ensures`)은 증명 게이트(Z3)가 증명하고, 빌드는 모든 의무(계약, 산술 넘침, 경계 등)가 증명되어야 된다. 미결정과 시간 초과도 실패다. 에이전트는 파일 대신 맵(L0~L2)과 정의 단위의 읽기·쓰기 도구로 코드에 접근한다. 컴파일러는 독자 구현이며, 파싱부터 증명, LLVM IR 생성까지 직접 한다.

지금은 v2다. 바닥부터 다시 시작했고 0.x 단계다. 이전 판 v0(v0.100: 검증이 선택이던 판)은 `legacy/v0` 브랜치에만 있다. v0에 대한 설명은 이 브랜치에 맞지 않는다.

## 어디서 읽나

| 문서 | 내용 |
| --- | --- |
| [`docs/DESIGN.md`](docs/DESIGN.md) | 확정된 설계(정의, 전제, 원칙, 언어 표면) |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | 단계와 통과 기준, 가설 등록부 |
| [`compiler/README.md`](compiler/README.md) | 컴파일러 사용법과 표면 규칙 |
| [`docs/diagnostics.md`](docs/diagnostics.md) | 진단 코드 |
| [`dev-docs/reports/utility.md`](dev-docs/reports/utility.md) | 측정으로 본 효용(살아 있는 보고서) |
| [`CLAUDE.md`](CLAUDE.md) | 작업 규칙과 그 밖의 문서 위치 |

## 빌드

```sh
cargo build
target/debug/bmb verify tests/run/fn_values.bmb
```

Z3와 LLVM(clang)이 필요하다. 판은 `scripts/toolchain.env`에 있다.

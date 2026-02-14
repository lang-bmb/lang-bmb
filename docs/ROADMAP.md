# BMB Roadmap

> 목표: **Rust로부터 완전히 독립된 프로그래밍 언어** — BMB 자체로 컴파일, 테스트, 도구 체인 운영

---

## 현재 상태 (2026-02-12)

| 항목 | 상태 | 비고 |
|------|------|------|
| **버전** | v0.93.2 | Cycle 494: select i1 직접 생성 |
| **단계** | v0.93 코드젠 최적화 | Function attrs, byte_at, nullable, select 완료 |
| **Bootstrap** | ✅ 3-Stage Fixed Point | 68,975 lines, Stage 1: ~1.1s |
| **Benchmarks** | ✅ 67/67 정상 | 5 FASTER, 5 PASS, 0 OK, 0 WARN vs Clang -O3 (100% parity+) |
| **Tests** | ✅ 5,234개 통과 | 2845 lib + 47 main + 2319 integration + 23 doc |
| **Golden Tests** | ✅ 27/27 통과 | 부트스트랩 컴파일러 바이너리 검증 |
| **Stability** | ✅ STABILITY.md | 언어/API 동결 문서화 |
| **동시성 지원** | ✅ 부트스트랩 완료 | 토큰/타입/MIR/코드젠/extern 선언 완료 |
| **Golden Binary** | ✅ v0.88.10 | Rust 없이 부트스트랩 가능 |
| **i32 타입** | ✅ 이미 구현됨 | 컴파일러 전 레이어 지원 (lexer~codegen) |

---

## 완료된 Phase 요약 (v0.1 ~ v0.69)

### Foundation & Bootstrap (v0.1 ~ v0.47) ✅

| 범위 | 내용 |
|------|------|
| 언어 기능 | 타입 시스템, 계약, 제어 흐름, f64, Vec, Box |
| 컴파일러 | Lexer, Parser, 타입 추론, MIR, LLVM/WASM 백엔드, SMT 검증 |
| 부트스트랩 | 30K LOC 자체 호스팅, 3-Stage 검증 완료 |
| 도구 | gotgan 패키지 매니저, VS Code, LSP, Formatter, Linter |
| 인프라 | CI/CD, 멀티플랫폼 빌드, 성능 회귀 탐지 |

### Type System & Performance (v0.48 ~ v0.57) ✅

| Phase | 내용 | 상태 |
|-------|------|------|
| v0.48-51 | Fin[N], Range, Aliasing, LTO/PGO | ⚠️ 타입만 완료 |
| v0.52 | 파서 통합 | ✅ 완료 |
| v0.53 | 부트스트랩 완성 | ✅ 완료 |
| v0.54 | 성능 게이트 | ✅ 0.959x 달성 |
| v0.55 | 생태계 (14개 패키지) | ✅ 완료 |
| v0.56 | 시연 & 문서 | ✅ 완료 |
| v0.57 | 보안 감사 | ✅ 완료 |

### Dogfooding & Optimization (v0.60 ~ v0.69) ✅

| Phase | 내용 | 상태 |
|-------|------|------|
| v0.60 | 성능 최적화 (Bootstrap 3x 개선) | ✅ 완료 |
| v0.61 | 완전한 셀프호스팅 | ✅ 완료 |
| v0.62 | 테스트 인프라 (BMB로 BMB 테스트) | ✅ 완료 |
| v0.63 | 벤치마크 도구 | ✅ 완료 |
| v0.64-66 | 개발 도구, 패키지 관리, 검증 | ✅ 완료 |
| v0.67-68 | 릴리스 후보, 안정화 | ✅ 완료 |
| v0.69 | Gate #3.1 달성 (13/14 PASS) | ✅ 완료 |

---

## Concurrency Primitives (v0.70 ~ v0.85) ✅

| Phase | Feature | 상태 |
|-------|---------|------|
| v0.70 | ThreadSpawn/ThreadJoin | ✅ |
| v0.71 | Mutex<T> | ✅ |
| v0.72 | Atomic<T> | ✅ |
| v0.73 | Channel<T> send/recv | ✅ |
| v0.74 | RwLock, Barrier, Condvar | ✅ |
| v0.75 | Future<T>, async/await, .await | ✅ |
| v0.76 | try_recv, T? methods | ✅ |
| v0.77 | recv_timeout | ✅ |
| v0.78 | Async Executor, block_on | ✅ |
| v0.79 | send_timeout | ✅ |
| v0.80 | Channel close, recv_opt | ✅ |
| v0.81 | Channel iteration (for-in) | ✅ |
| v0.82 | select! macro (기본) | ✅ |
| v0.83 | AsyncFile (async I/O) | ✅ |
| v0.83.1 | AsyncSocket (tcp_connect) | ✅ |
| v0.84 | ThreadPool | ✅ |
| v0.85 | Scoped Threads (Scope) | ✅ |

---

## 🎯 개발 로드맵

### Phase v0.82-v0.85: Concurrency 완성

#### v0.82: select! 매크로 ✅ 기본 완료

```bmb
select {
    value = rx1.recv() => { /* handle rx1 */ }
    value = rx2.recv() => { /* handle rx2 */ }
    _ = timeout(100) => { /* timeout */ }
}
```

| 태스크 | 상태 | 설명 |
|--------|------|------|
| select 문법 설계 | ✅ | 파서 + AST 확장 완료 |
| 타입 체킹 | ✅ | 바인딩 변수 스코프 처리 |
| MIR 기본 lowering | ✅ | 첫 번째 arm 블로킹 실행 |
| 폴링 구현 | ✅ | try_recv 기반 다중 채널 폴링 |
| 런타임 (epoll/IOCP) | 📋 v0.82.2 | 플랫폼별 다중화 |

#### v0.83: 비동기 I/O

| 태스크 | 상태 | 설명 |
|--------|------|------|
| AsyncFile 타입 | ✅ | 파서/AST/타입체커 완료 |
| AsyncFile 메서드 | ✅ | read(), write(), close() |
| async_open() 함수 | ✅ | 파일 열기 |
| 런타임 구현 | ✅ | 동기 I/O 모델 (기초) |
| AsyncSocket API | ✅ | tcp_connect, recv/send/disconnect |
| 이벤트 루프 통합 | 📋 Future | async/await 연동 (Beta 예정) |

#### v0.84: Thread Pool ✅

| 태스크 | 상태 | 설명 |
|--------|------|------|
| ThreadPool API | ✅ | 작업 큐 기반 스케줄러 |
| execute() 태스크 | ✅ | 태스크 제출 |
| join()/shutdown() | ✅ | 완료 대기 및 종료 |

#### v0.85: Scoped Threads ✅

| 태스크 | 상태 | 설명 |
|--------|------|------|
| Scope 타입 | ✅ | 구조화된 동시성 |
| thread_scope() | ✅ | 스코프 생성 |
| spawn()/wait() | ✅ | 스코프 스레드 관리 |

---

### Phase v0.86-v0.89: Alpha (품질 게이트) ✅ 완료

> **목표**: 내부 품질 검증, 테스트 커버리지, 부트스트랩 안정화
>
> **주의**: v0.86에서 언어 동결을 시도했으나 **철회**함.
> 성능 목표 미달 상태에서 동결은 workaround다.
> i32 타입 추가 등 언어 스펙 변경이 필요하면 **언어를 바꾼다**.

#### v0.86: ~~Language Freeze~~ → 품질 기준 수립

| 태스크 | 상태 | 설명 |
|--------|------|------|
| ~~문법 동결~~ | ⚠️ 철회 | 성능 목표 달성 전 동결 불가 |
| ~~stdlib API 동결~~ | ⚠️ 철회 | i32 등 타입 추가 가능성 |
| STABILITY.md | ✅ | 안정성 **정책** 문서화 (동결 자체는 아님) |
| 품질 게이트 기준 | ✅ | 테스트/벤치마크/부트스트랩 기준 정립 |

#### v0.87: Bootstrap 강화 ✅ 완료

| 태스크 | 상태 | 설명 |
|--------|------|------|
| --fast-compile 플래그 | ✅ | opt 패스 스킵으로 3x 빠른 컴파일 |
| Bootstrap 성능 최적화 | ✅ | Stage 1: 1.7s → 0.54s (< 1.0s 달성!) |
| Bootstrap 안정성 | ✅ | Winsock2 링크 수정, 3-Stage 검증 통과 |
| 크로스 컴파일 문서화 | ✅ | CROSS_COMPILATION.md 실전 가이드 |

#### v0.88: Self-Hosting 완성 ✅

| 태스크 | 상태 | 설명 |
|--------|------|------|
| Bootstrap 동시성 토큰 | ✅ | async/await/select, Mutex/Channel/Future 토큰 |
| Bootstrap 동시성 타입 | ✅ | 동시성 제네릭 타입 생성자 추가 |
| Bootstrap 동시성 MIR | ✅ | thread_spawn, mutex, channel 등 MIR 명령어 |
| Bootstrap 동시성 코드젠 | ✅ | 런타임 함수 선언 및 호출 생성 |
| Golden Bootstrap 수정 | ✅ | 플랫폼별 링킹 (ws2_32/pthread), Winsock2 include 순서 |
| Bootstrap CLI 확장 | ✅ | emit-ir 명령어, 도움말, 버전 정보 |
| Golden Binary v0.88 | ✅ | 3-Stage 검증 후 golden binary 업데이트 |
| 테스트 러너 스크립트 | ✅ | run-bootstrap-tests.sh 작성 |
| SSA 변수명 버그 | ✅ | v0.88.1: if/else 분기에서 let 변수의 고유 SSA 명칭 생성 (lower.rs, compiler.bmb, lowering.bmb) |
| Arena Allocator | ✅ | v0.88.2: 런타임 메모리 관리 - arena 할당자 추가 |
| Arena Hard Limit | ✅ | v0.88.4: 4GB 기본 제한 + exit(1) on exceed. BMB_ARENA_MAX_SIZE 환경변수 지원. BSOD 완전 해결 |
| 테스트 프레임워크 | ✅ | v0.88.4: BMB로 작성된 test_runner + 5개 테스트 스위트 (parser 257, selfhost 280, lexer 264, codegen 10, error 10) |
| exec_output 런타임 | ✅ | v0.88.4: bmb_exec_output C 런타임 함수 + MIR/코드젠 연결 |
| 컴파일러 메모리 최적화 | ✅ | v0.88.5: 코드젠 리턴 프로토콜 최적화 - mapping 미변경 시 zero-copy, ~2GB 절감 |
| Arena Save/Restore | ✅ | v0.88.6: arena_save/restore로 함수별 메모리 회수 - 6.2GB→420MB (93% 절감), lowering.bmb 4GB 이내 컴파일 |
| 토큰 인코딩 수정 | ✅ | v0.88.7: 토큰 kind base 10000→10000000 확장 - 정수 리터럴 10000-10999 파싱 충돌 해결, mir.bmb/types.bmb 컴파일 성공 |
| 파서 에러 진단 | ✅ | v0.88.8: 파싱 에러에 line:col 위치 정보 추가, 사람이 읽기 쉬운 에러 메시지 |
| 에러 진단 전면 적용 | ✅ | v0.88.9: 전체 40+ 파서 에러를 make_error_at으로 전환 - 100% 위치 정보 커버리지 |
| 동시성 런타임 매핑 | ✅ | v0.88.10: 50+ 동시성 런타임 함수 매핑 (thread/mutex/channel/arc/rwlock/barrier/condvar/async/pool/scope) |
| 동시성 extern 선언 | ✅ | v0.88.10: 40+ extern 선언 추가 + 4개 시그니처 불일치 수정 |
| 동시성 타입 시그니처 | ✅ | v0.88.10: 50+ 동시성 내장 함수 타입 시그니처 추가 |
| 동시성 MIR 코드젠 | ✅ | v0.88.10: 40+ 동시성 MIR 명령어 → LLVM IR 번역 (mutex-new, channel-send, thread-spawn 등) |
| Golden Binary v0.88.10 | ✅ | 3-Stage 검증 후 golden binary 업데이트 |
| 컴파일러 100% BMB 전환 | ✅ → v0.94 | 3-Stage Fixed Point 달성, Rust 졸업 선언 (v0.94) |
| 개발 도구 BMB 전환 | 📋 → v0.96 | gotgan, LSP, Formatter (v0.96에서 BMB 재구현) |

#### v0.89: 내부 품질 게이트 ✅

| 태스크 | 상태 | 설명 |
|--------|------|------|
| 문법 키워드-메서드 수정 | ✅ | v0.89: `spawn` 등 키워드를 `.method()` 위치에서 사용 가능 (MethodName 프로덕션 추가) |
| MIR 후조건 Ret 추출 | ✅ | v0.89: `post ret >= 0` 패턴에서 ContractFact::ReturnCmp/ReturnVarCmp 추출 |
| MIR 최적화 테스트 | ✅ | v0.89: 6개 미테스트 패스 커버 (CopyProp, CSE, SimplifyBranches, UnreachableBlock, PhiSimpl, BlockMerging) — 43개 테스트 |
| 코드젠 라운드트립 테스트 | ✅ | v0.89: 12개 LLVM IR 검증 테스트 (최적화 효과 포함: 상수 접기, TCO→루프, 계약 제거, DCE) |
| 타입체커 단위 테스트 | ✅ | v0.89.1: types/mod.rs 38개 테스트 (levenshtein, BindingTracker, TypeChecker 통합) |
| SMT/CIR 단위 테스트 | ✅ | v0.89.1: SMT translator 30개 + CIR lower 9개 = 39개 테스트 |
| 빌드 파이프라인 개선 | ✅ | v0.89.1: 11개 unwrap() 제거 + path_str() 에러 처리 + 19개 테스트 |
| 복잡 최적화 패스 테스트 | ✅ | v0.89.1: 6개 복잡 패스 테스트 (GlobalFieldCSE, IfElseToSwitch, StringConcat, LICM, LinearRecurrence, ConditionalIncrement) — 12개 |
| 인터프리터 테스트 확장 | ✅ | v0.89.1: 20개 추가 (단항연산, 비교, F64, 전체 프로그램 실행, 에러 경로) — 29개 총 |
| Clippy + 에러/렉서 테스트 | ✅ | v0.89.2: clippy approx_constant 수정, 에러 모듈 27개 + 렉서 18개 테스트 |
| LSP 모듈 테스트 | ✅ | v0.89.2: format_type/expr/pattern 39개 + 좌표 변환 8개 + 유틸 4개 = 51개 테스트 |
| 인덱스/REPL 테스트 | ✅ | v0.89.2: IndexGenerator 24개 + REPL 8개 = 32개 테스트 |
| WASM 코드젠 테스트 확장 | ✅ | v0.89.2: 25개 추가 (비교/비트연산/F64/I32/단항/상수/설정) — 34개 총 |
| dead_code 어노테이션 정리 | ✅ | v0.89.2: LSP 모듈 오류 어노테이션 2개 제거 |
| 생태계 패키지 테스트 | ✅ | v0.89.3: SHA-256 구현 + XorShift64* PRNG + 219개 BMB 생태계 테스트 (Cycles 37-41) |
| 컴파일러 버그 4건 수정 | ✅ | v0.89.4: float/int ==, free() 리턴 타입, let-in-block MIR 스코프, codegen %tmp (Cycle 42) |
| 문법 let-in-while 수정 | ✅ | v0.89.4: BlockExpr 규칙 추가, desugar_block_lets — while/for/loop/spawn/closure에서 let 사용 가능 (Cycle 43) |
| 생태계 재귀→루프 전환 | ✅ | v0.89.4: bmb-sha256 7개 + bmb-hashmap 10개 재귀 워크어라운드 함수를 while 루프로 변환 (Cycle 44) |
| 생태계 재귀→루프 전환 (확장) | ✅ | v0.89.5: 15개 패키지, 92개 재귀 워크어라운드 → while 루프 변환 완료 (Cycles 44-50) |
| 인터프리터 *i64 포인터 인덱싱 | ✅ | v0.89.5: ptr[i] / set ptr[i] = val 지원, bmb-ptr/bmb-sort 15개 테스트 복원 (Cycle 50) |
| 문법 if-branch BlockStmt | ✅ | v0.89.6: if/else 브랜치에서 대입/let 바인딩 직접 지원, {{ }} 이중블록 패턴 제거 (Cycle 52) |
| 생태계 이중블록 제거 | ✅ | v0.89.6: 11개 패키지에서 35개 {{ }} 이중블록 워크어라운드 제거 (Cycle 53) |
| MIR/인터프리터 테스트 확장 | ✅ | v0.89.6: MIR 소스 기반 테스트 13개 + 인터프리터 통합 테스트 13개 + 파서 테스트 3개 = 29개 (Cycles 54-55) |
| 생태계 3패키지 테스트 추가 | ✅ | v0.89.7: bmb-log 16개 + bmb-testing 19개 + bmb-fmt 14개 = 49개 테스트, hex_digit 슬라이스 버그 수정 (Cycle 57) |
| LLVM 코드젠 라운드트립 확장 | ✅ | v0.89.7: TextCodeGen 소스 기반 23개 테스트 (산술/비교/제어흐름/타입/함수) (Cycle 58) |
| MIR 데이터 구조 테스트 | ✅ | v0.89.7: mir/mod.rs 34개 테스트 (타입 시스템, 연산자, LoweringContext, format_mir) (Cycle 59) |
| 인터프리터 E2E + 통합 테스트 | ✅ | v0.89.7: 24개 통합 테스트 (인터프리터 19개 + 에러 처리 4개 + 파이프라인 2개) (Cycle 60) |
| 완전성 검사 테스트 확장 | ✅ | v0.89.8: exhaustiveness.rs 22개 테스트 (튜플/구조체/Or/가드/바인딩) (Cycle 62) |
| CIR + 계약 검증 테스트 | ✅ | v0.89.8: cir/mod.rs 24개 + contract.rs 11개 = 35개 테스트 (Cycle 63) |
| AST + 전처리기 테스트 | ✅ | v0.89.8: output.rs 14개 + preprocessor 7개 + ast/mod.rs 7개 = 28개 테스트 (Cycle 64) |
| 쿼리 + 리졸버 테스트 | ✅ | v0.89.8: query 12개 + resolver 8개 = 20개 테스트 (Cycle 65) |
| CIR 로우어링 + 출력 테스트 | ✅ | v0.89.9: lower.rs 31개 + output.rs 23개 = 54개 테스트 (Cycle 67) |
| 빌드 파이프라인 테스트 확장 | ✅ | v0.89.9: build/mod.rs 15개 테스트 (VerificationMode, BuildConfig, OutputType) (Cycle 68) |
| CIR 검증 + ProofDB 테스트 | ✅ | v0.89.9: verify.rs 14개 + proof_db.rs 11개 = 25개 테스트 (Cycle 69) |
| SMT 솔버 + PIR 로우어링 테스트 | ✅ | v0.89.9: solver.rs 14개 + lower_to_mir.rs 10개 = 24개 테스트 (Cycle 70) |
| PIR 전파 테스트 | ✅ | v0.89.9: propagate.rs 16개 테스트 (PropagationRule, expr_to_proposition, mentions_var) (Cycle 71) |
| AST 표현식 테스트 | ✅ | v0.89.9: expr.rs 13개 테스트 (BinOp/UnOp/RangeKind/StateKind Display) (Cycle 72) |
| 검증 요약 + 증분 검증 테스트 | ✅ | v0.89.9: summary.rs 18개 + incremental.rs 17개 = 35개 테스트 (Cycle 73) |
| SMT 번역기 + PIR→MIR 팩트 테스트 | ✅ | v0.89.9: translator.rs 21개 + to_mir_facts.rs 18개 = 39개 (Cycle 74) |
| 인터프리터 에러 + 스코프 + MIR 증명 테스트 | ✅ | v0.89.9: error.rs 18개 + scope.rs 12개 + proof_guided.rs 16개 = 46개 (Cycle 75) |
| MIR 최적화 패스 테스트 (파트1) | ✅ | v0.89.10: optimize.rs +37 테스트 (Pipeline/Stats/ConstFold/AggressiveInlining/MemoryEffect/LoopBounded) (Cycle 77) |
| MIR 최적화 패스 테스트 (파트2) | ✅ | v0.89.10: optimize.rs +19 테스트 (AlgebraicSimpl/DCE/ConstPropNarrowing/LICM/simplify_binop) (Cycle 78) |
| 타입체커 통합 테스트 확장 | ✅ | v0.89.10: types/mod.rs +23 테스트 (let/if/while/for/tuple/enum/match/arity/struct/contract) (Cycle 79) |
| LLVM 텍스트 코드젠 테스트 | ✅ | v0.89.10: llvm_text.rs +23 테스트 (f64/bool/비교/for/struct/enum/match/contract/bitwise) (Cycle 80) |
| MIR 로우어링 테스트 확장 | ✅ | v0.89.10: lower.rs +20 테스트 (단항/상수/struct/enum/비교/계약/while/if/tuple/bitwise) (Cycle 81) |
| 인터프리터 E2E 테스트 확장 | ✅ | v0.89.10: eval.rs +18 테스트 (for/struct/enum/tuple/재귀/match/string/bitwise/shift) (Cycle 82) |
| PIR + CIR SMT 테스트 | ✅ | v0.89.10: pir/mod.rs +14, cir/smt.rs +48 = 62개 테스트 (타입변환/증명/Proposition/SmtSort/binop) (Cycle 83) |
| 계약 검증 + WASM 테스트 | ✅ | v0.89.10: contract.rs +23, wasm_text.rs +12 = 35개 테스트 (Display/Report/VerifyResult/다중함수/로컬) (Cycle 84) |
| 소규모 모듈 커버리지 스위프 | ✅ | v0.89.10: ast/types.rs +22, env.rs +8, value.rs +12, cfg +5, derive +5 = 52개 테스트 (Cycle 85) |
| 코드 커버리지 > 80% | 📋 | 테스트 범위 확대 (현재 1295개 단위 테스트) |
| 퍼징 테스트 | 📋 | libFuzzer 기반 입력 검증 |
| 메모리 안전성 검증 | 📋 | AddressSanitizer 통과 |

---

### Phase v0.90-v0.93: 성능 + 자체 컴파일 🎯 현재

> **핵심 원칙**: Performance > Everything. 성능 목표 미달 = 언어 스펙 변경.
> 언어 동결은 모든 목표 달성 **후에만** 가능하다.

#### v0.90: LLVM 최적화 파이프라인 ✅ 완료

| 태스크 | 상태 | 설명 |
|--------|------|------|
| 벤치마크 스위트 67개 | ✅ | 11 카테고리, 전체 컴파일/출력 매칭 |
| ConstFunctionEval | ✅ | MIR 레벨 @const 0-arg 인라이닝 |
| Scalarizer 패스 | ✅ | LLVM 자동 벡터화 역전환 |
| --mcpu=native | ✅ | 타겟별 코드젠 |
| mustprogress 속성 | ✅ | 루프 종료 보장 속성 |
| nsz float copies | ✅ | IEEE 754 -0.0 안전 처리 |
| Private linkage @inline | ✅ | 인라인 함수 private 링키지 |
| instcombine 제거 | ✅ | v0.90.21: GEP 정규화 문제 해결, sorting 50%↑ |
| 성능 검증 완료 | ✅ | 11 FASTER, 4 PASS, 2 OK vs C (81% parity+) |

#### v0.91: i32 타입 (언어 스펙 변경) 📋 → v0.95로 이관

> **참고**: Rust 컴파일러에는 이미 i32 지원이 구현되어 있음.
> i32의 부트스트랩 구현은 v0.95 (BMB 언어 완성)에서 compiler.bmb 직접 수정으로 진행.
> Rust 측 추가 작업 없음.

| 태스크 | 상태 | 설명 |
|--------|------|------|
| i32 타입 스펙 설계 | ✅ | Rust 컴파일러에 이미 구현 |
| 파서/AST i32 지원 | ✅ | Rust 측 완료 |
| 타입체커 i32 추론 | ✅ | Rust 측 완료 |
| MIR i32 명령어 | ✅ | Rust 측 완료 |
| 코드젠 i32 LLVM IR | ✅ | Rust 측 완료 |
| **부트스트랩 i32 지원** | 📋 → v0.95 | compiler.bmb에서 직접 구현 (Rust 졸업 후) |
| 벤치마크 검증 | 📋 → v0.95 | 부트스트랩 구현 후 검증 |

#### v0.92: 부트스트랩 기능 확장 📋

> **목표**: 부트스트랩 컴파일러가 임의의 BMB 프로그램 컴파일 가능
>
> **참고**: 3-Stage Fixed Point = 사실상 4-Stage 자체 컴파일 달성.
> compiler.bmb는 순수 함수형 스타일(early return/mutation/deref 미사용)이므로
> 3-Stage에서 이미 자기 자신을 정확히 컴파일함. 아래 기능은 임의의 BMB 프로그램
> 컴파일을 위해 필요.

| 태스크 | 상태 | 설명 |
|--------|------|------|
| Return expression 로워링 | ✅ | 임의 프로그램의 조기 반환 지원 (v0.90.31) |
| Reference `&T`/`*T` 로워링 | ✅ | 참조/포인터 연산 (v0.90.11) |
| Field assignment 로워링 | ✅ | 구조체 필드 대입 (v0.90.5) |
| `loop {}` 무한 루프 | ✅ | break 기반 무한 루프 (v0.90.31) |
| `continue` 문 | ✅ | 루프 반복 건너뛰기 (v0.90.31) |
| Type cast (`as`) 로워링 | ✅ | 명시적 타입 변환 (v0.90.1, no-op for same-width) |
| Nullable `T?` 로워링 | ✅ | 널러블 타입 지원 (v0.90.102: is_some/is_none/unwrap/unwrap_or/expect + T? 파싱) |
| Closure capture 완성 | 📋 → v0.95 | 자유 변수 캡처 분석 (v0.95에서 compiler.bmb 직접 구현) |

#### v0.93: 부트스트랩 코드젠 최적화 📋

> **목표**: 부트스트랩 컴파일러 성능 = Rust 컴파일러 성능

| 태스크 | 상태 | 설명 |
|--------|------|------|
| Function attributes 추가 | ✅ | Cycles 489-492: `nounwind`, `memory(none/read)`, `willreturn`, `norecurse`, `noalias` (25개 extern) |
| `byte_at` 인라인 | ✅ | v0.90.71: runtime call → GEP+load |
| T? nullable params/returns | ✅ | Cycle 493: `skip_nullable` 헬퍼, 7개 파서 함수 업데이트 |
| `select` 직접 생성 | ✅ | Cycle 494: int/bool 상수 전용 (var/string은 LLVM O2 미스컴파일 이슈) |
| Identity copies 제거 | 📋 | `add nsw i64 0, X` 15,818건 (22.9%) — LLVM O2가 제거하므로 런타임 영향 없음, 컴파일 속도에만 영향 |
| Dominator tree CSE | 📋 | 크로스 블록 공통 부분식 제거 |
| copy propagation 완성 | 📋 | optimize.bmb:412 TODO 해결 |

---

### ★ Rust 졸업 선언 (v0.93 → v0.94 전환점)

> **Rust 졸업은 선언이 아니라 달성이다.**
> 아래 5가지 조건이 **전부** 충족되어야 졸업이 완료된다.
> 하나라도 미달이면 졸업은 미완성이다.

#### 졸업 조건 (5/5 필수)

| # | 조건 | 검증 방법 | 달성 Phase |
|---|------|----------|-----------|
| G-1 | **100% 부트스트랩** | 3-Stage Fixed Point + 골든 바이너리로 Rust 없이 컴파일러 빌드 | v0.94 |
| G-2 | **100% 셀프호스팅** | compiler.bmb가 모든 BMB 언어 기능을 컴파일 (i32, closure, generics, where 절 포함) | v0.95 |
| G-3 | **C/Rust 벤치마크 추월** | 모든 벤치마크에서 BMB > C/Rust. 동등 인정 조건: C/Rust가 이미 오버헤드 제로인 경우 동일 IR 출력 확인 | v0.95 |
| G-4 | **100% 에코시스템 BMB 재구현** | gotgan, LSP, Formatter, Linter, REPL, test_runner 전부 BMB로 재작성. Rust 도구 코드 0줄 | v0.96 |
| G-5 | **100+ BMB 패키지 (곳간)** | 곳간(gotgan) 레지스트리에 100개 이상 패키지 등록, 전체 빌드 + 테스트 통과 확인 | v0.96 |

```
졸업 조건 충족 진행도:

G-1 부트스트랩    [██████████] ← 3-Stage Fixed Point 달성 (v0.93)
G-2 셀프호스팅    [██████░░░░] ← i32/closure/generics 미구현 (v0.95 목표)
G-3 벤치마크 추월  [████████░░] ← 67/67 parity+, 일부 gap 존재 (v0.95 목표)
G-4 에코시스템    [░░░░░░░░░░] ← 전부 Rust (v0.96 목표)
G-5 100+ 패키지   [█░░░░░░░░░] ← 현재 ~14개 (v0.96 목표)
```

#### Rust 컴파일러 동결 정책

| 항목 | 정책 |
|------|------|
| **새 기능 추가** | ❌ 금지 — 모든 새 기능은 compiler.bmb에서 직접 구현 |
| **버그 수정** | ⚠️ 부트스트래핑 차단 버그만 수정 |
| **테스트 추가** | ❌ 금지 — 새 테스트는 BMB 골든 테스트로 작성 |
| **최적화 패스** | ❌ 금지 — 부트스트랩 코드젠에서 직접 최적화 |
| **유지보수** | 🔧 `cargo test --release` 통과 유지만 (회귀 방지) |

#### 개발 워크플로 전환

```
[Before — Rust 중심]
1. bmb/src/*.rs 수정
2. cargo test --release
3. bootstrap/compiler.bmb에 포팅
4. 3-Stage 검증

[After — BMB 중심]
1. bootstrap/compiler.bmb 직접 수정
2. Stage 1 빌드 + 골든 테스트
3. 3-Stage Fixed Point 검증
4. 완료 (Rust 측 변경 없음)
```

---

### Phase v0.94: Rust 졸업 + BMB 자립 기반 📋

> **목표**: Rust 의존성 최소화, BMB 네이티브 개발 인프라 확립
>
> 원칙: **새로운 기능은 compiler.bmb에서 직접 구현한다.**
>
> **졸업 조건 담당**: G-1 (100% 부트스트랩)

#### v0.94.1: BMB 네이티브 테스트 확장

| 태스크 | 상태 | 설명 |
|--------|------|------|
| 골든 테스트 확장 (27→50+) | 📋 | 컴파일러 기능별 커버리지 확보 |
| 에러 케이스 골든 테스트 | 📋 | 파서 에러, 타입 에러 등 실패 경로 검증 |
| 성능 골든 테스트 | 📋 | 코드젠 품질 회귀 탐지 (IR 패턴 검증) |
| 부트스트랩 테스트 프레임워크 확장 | 📋 | test_runner.bmb 고도화 |

#### v0.94.2: 개발 파이프라인 BMB 전환

| 태스크 | 상태 | 설명 |
|--------|------|------|
| bootstrap-first 개발 스크립트 | 📋 | 골든 바이너리 → Stage 1 → 테스트 → 검증 (Rust 불필요) |
| 골든 바이너리 업데이트 프로세스 | 📋 | 릴리스마다 플랫폼별 바이너리 갱신 |
| CI: BMB 파이프라인 추가 | 📋 | 골든 바이너리 기반 CI (cargo 없이) |
| Rust 컴파일러 동결 커밋 | 📋 | FROZEN.md 작성, 더이상 기능 추가 없음 선언 |

#### v0.94.3: G-1 검증 — 100% 부트스트랩

| 게이트 | 조건 | 상태 |
|--------|------|------|
| 3-Stage Fixed Point | Stage 2 == Stage 3 바이너리 동일 | ✅ |
| 골든 바이너리 부트스트랩 | Rust 없이 골든 바이너리만으로 컴파일러 빌드 성공 | 📋 |
| 플랫폼별 골든 바이너리 | Windows/Linux/macOS 골든 바이너리 존재 | 📋 |
| 벤치마크 커버리지 | 전체 67 벤치마크 정상 | ✅ |

---

### Phase v0.95: BMB 언어 완성 📋

> **목표**: 부트스트랩 컴파일러가 임의의 BMB 프로그램을 완전히 컴파일
>
> **모든 구현은 compiler.bmb에서 직접 수행.**
> Rust 컴파일러는 수정하지 않는다.

#### v0.95.1: 핵심 미구현 기능

| 태스크 | 상태 | 설명 |
|--------|------|------|
| i32 타입 시스템 | 📋 | lexer→parser→type checker→MIR→codegen 전체, compiler.bmb 직접 구현 |
| Closure capture | 📋 | 자유 변수 캡처 분석, lowering.bmb에서 구현 |
| 제네릭 확장 | 📋 | 부트스트랩 타입체커에서 제네릭 파라미터 지원 확장 |
| Where 절 | 📋 | 타입 바운드 제약 |

#### v0.95.2: 코드젠 최적화 (BMB 네이티브)

| 태스크 | 상태 | 설명 |
|--------|------|------|
| Identity copies 제거 | 📋 | llvm_ir.bmb에서 `add nsw i64 0, X` 패턴 직접 최적화 |
| select 확장 (var/float 지원) | 📋 | LLVM O2 미스컴파일 원인 분석 후 확장 |
| Dominator tree CSE | 📋 | optimize.bmb에서 크로스 블록 CSE |
| Copy propagation | 📋 | optimize.bmb에서 불필요 copy 전파 |

#### v0.95.3: 에러 진단 개선

| 태스크 | 상태 | 설명 |
|--------|------|------|
| 타입 에러 메시지 | 📋 | 인간 친화적 에러 출력 (types.bmb) |
| 위치 추적 고도화 | 📋 | 정확한 line:col:span 정보 |
| 스택 트레이스 | 📋 | 런타임 에러 위치 추적 |

---

### Phase v0.96: BMB 도구 생태계 📋

> **목표**: 개발 도구 체인을 Rust에서 BMB로 전환
>
> 핵심: gotgan, LSP, Formatter가 BMB로 작성되면
> 전체 개발 도구 체인이 BMB 자립.

| 태스크 | 현재 | 목표 | 설명 |
|--------|------|------|------|
| gotgan 재구현 | Rust 5,150 LOC | BMB | 패키지 매니저 (HTTP는 C FFI 활용) |
| LSP 서버 | Rust 2,603 LOC | BMB | 부트스트랩 컴파일러 API 활용 |
| Formatter | Rust | BMB | AST 기반 소스 포매팅 |
| Linter | Rust | BMB | 정적 분석 규칙 |
| REPL | Rust 326 LOC | BMB | 인터프리터 루프 |

**전제 조건**: v0.95에서 closure, generics 등 언어 기능 완성

---

### Phase v0.97: 플랫폼 + 문서 📋

> **목표**: 크로스 플랫폼 지원, 공식 문서 완성

#### v0.97.1: 플랫폼 검증

| 플랫폼 | 설명 |
|--------|------|
| Windows x64 | MSVC/MinGW 호환 (현재 주 플랫폼) |
| Linux x64/ARM64 | 주요 배포판 테스트 |
| macOS x64/ARM64 | Apple Silicon 지원 |

**플랫폼별 골든 바이너리 배포** — Rust 없이 설치 가능

#### v0.97.2: 문서 완성

| 문서 | 설명 |
|------|------|
| Language Specification | 공식 언어 스펙 |
| Standard Library Reference | API 문서 |
| Compiler Internals | BMB 부트스트랩 컴파일러 아키텍처 |
| BMB-only Build Guide | Rust 없이 빌드하는 방법 |

---

### Phase v0.98-v0.99: Release Candidate

> **목표**: 안정화, 커뮤니티 검증

#### v0.98: RC1

| 조건 | 설명 |
|------|------|
| Feature Freeze | 기능 추가 중단 |
| Critical Bug Only | P0 버그만 수정 |
| BMB-only 설치 검증 | 골든 바이너리로 전체 빌드 가능 확인 |

#### v0.99: RC2 (Final)

| 조건 | 설명 |
|------|------|
| Showstopper Only | 출시 차단 버그만 수정 |
| 문서 최종 검토 | 오타/오류 수정 |
| 릴리스 승인 | 커뮤니티 검증 |

---

## 벤치마크 현황 (v0.90.22, Cycle 175)

**Rust 컴파일러 vs Clang -O3 (>50ms 런타임 벤치마크):**

| 판정 | 개수 | 비율 |
|------|------|------|
| ✅ BMB > C (FASTER) | 3개 | 27% |
| ✅ BMB ≈ C (PASS, ≤1.02x) | 3개 | 27% |
| ✅ BMB ≈ C (OK, ≤1.05x) | 3개 | 27% |
| ⚠️ WARN (≤1.10x) | 2개 | 18% |

**주요 벤치마크 (>50ms, vs Clang -O3):**

| 벤치마크 | BMB/Clang | 판정 | 변경 |
|----------|-----------|------|------|
| fasta | 0.94x | FASTER | |
| gcd | 0.97x | FASTER | |
| binary_trees | 0.99x | FASTER | |
| spectral_norm | 1.00x | PASS | |
| mandelbrot | 1.01x | PASS | |
| nqueen | 1.01x | PASS | |
| collatz | 1.05x | OK | was 1.07x → loop bound fix (Cycle 174) |
| primes_count | 1.05x | OK | |
| ackermann | 1.06x | OK | was 1.08x → recursive call fix (Cycle 175) |
| sieve | 1.07x | WARN | loop bound narrowing residual |
| digital_root | 1.07x | WARN | sub-50ms, high variance |

**전체 벤치마크 스위트:** 67개 (11 카테고리), 전체 컴파일/출력 정상

**Cycles 172-175 개선 내역:**

| Cycle | 수정 | 효과 |
|-------|------|------|
| 172 | div/mod param narrowing skip | digital_root 1.26x→1.07x |
| 173 | Bootstrap mustprogress+willreturn | Bootstrap LLVM 최적화 개선 |
| 174 | Loop-invariant bound skip | collatz 1.07x→1.05x, sieve 1.07x→1.04x |
| 175 | Self-recursive call narrowing skip | ackermann 1.08x→1.06x |

**⚠️ 남은 gap:**

| 벤치마크 | 비율 | 원인 | 참고 |
|----------|------|------|------|
| sieve | 1.07x | while-loop bound narrowing | Phi 아닌 mutable var 패턴 |
| digital_root | 1.07x | sub-50ms, measurement noise | 실제 차이 미미 |

---

## 버전 정책

| 버전 유형 | 형식 | 설명 |
|-----------|------|------|
| Minor | v0.X.0 | 로드맵 계획 |
| Patch | v0.X.Y | 버그 수정 |
| RC | v0.98, v0.99 | Release Candidate |

---

## 타임라인

```
═══════════════════ Rust 시대 (v0.1 ~ v0.93) ═══════════════════

v0.70-85 Concurrency Primitives ✅ ────────────────────
         │  스레드, 채널, async/await, select, thread pool
         ▼
v0.86-89 Alpha (품질 게이트) ✅ ───────────────────────
         │  5,234 Rust 테스트, 부트스트랩 강화
         ▼
v0.90    LLVM 최적화 파이프라인 ✅ ────────────────────
         │  67 벤치마크, 100% parity+
         ▼
v0.91-92 Self-Hosting 완성 ✅ ─────────────────────────
         │  return, &T, loop, continue, T? → 3-Stage Fixed Point
         ▼
v0.93    Bootstrap 코드젠 최적화 🎯 현재 ─────────────
         │  function attrs, byte_at, nullable, select
         ▼
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
         ★ RUST 졸업 — Rust 컴파일러 동결 ★
         │  더이상 Rust 측 기능 추가 없음
         │  모든 개발은 BMB에서 직접 수행
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

═══════════════════ BMB 시대 (v0.94+) ══════════════════════

v0.94    Rust 졸업 + BMB 자립 기반 📋 ────────────────
         │  BMB 테스트 확장, 개발 파이프라인 전환
         │  성능 게이트 검증
         ▼
v0.95    BMB 언어 완성 📋 ────────────────────────────
         │  i32, closures, generics (compiler.bmb에서 직접 구현)
         │  코드젠 최적화 (BMB 네이티브)
         ▼
v0.96    BMB 도구 생태계 📋 ──────────────────────────
         │  gotgan, LSP, Formatter를 BMB로 재구현
         │  → 전체 도구 체인 BMB 자립
         ▼
v0.97    플랫폼 + 문서 📋 ────────────────────────────
         │  크로스 플랫폼 골든 바이너리 배포
         ▼
v0.98-99 Release Candidate ★ ─────────────────────────
         BMB-only 설치 검증, 커뮤니티 검증 후 릴리스
```

---

## 문서

| 문서 | 내용 |
|------|------|
| SPECIFICATION.md | 언어 스펙 |
| LANGUAGE_REFERENCE.md | 언어 레퍼런스 |
| ARCHITECTURE.md | 컴파일러 아키텍처 |
| BOOTSTRAP_BENCHMARK.md | 부트스트랩/벤치마크 프로세스 |
| BUILD_FROM_SOURCE.md | BMB-only 빌드 가이드 |

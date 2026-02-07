# BMB Roadmap

> 목표: 완전히 준비된 프로그래밍 언어 - Rust 의존성 제거, 성능 검증, 생태계 구축

---

## 현재 상태 (2026-02-07)

| 항목 | 상태 | 비고 |
|------|------|------|
| **버전** | v0.88.10 | Self-Hosting (Alpha) |
| **단계** | Alpha | Concurrency 완료 → **Alpha (v0.88)** → Beta (v0.90) → RC (v0.98) |
| **Bootstrap** | ✅ 3-Stage 완료 | Stage 1: ~0.55s (--fast-compile) |
| **Benchmarks** | ✅ 18/30 BMB > C | 60% C보다 빠름 |
| **Tests** | ✅ 334개 통과 | 243 + 68 + 23 (cargo) + BMB 테스트 5/5 |
| **Stability** | ✅ STABILITY.md | 언어/API 동결 문서화 |
| **동시성 지원** | ✅ 부트스트랩 완료 | 토큰/타입/MIR/코드젠/extern 선언 완료 |
| **Golden Binary** | ✅ v0.88.10 | Rust 없이 부트스트랩 가능 |

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

### Phase v0.86-v0.89: Alpha (Feature Freeze) 🎯 현재

> **목표**: 언어 기능 동결, API 안정화, 내부 품질 검증

#### v0.86: Language Freeze ✅ 진행 중

| 태스크 | 상태 | 설명 |
|--------|------|------|
| 문법 동결 | ✅ | 새 키워드/구문 추가 중단 |
| stdlib API 동결 | ✅ | 공개 API 확정 |
| STABILITY.md | ✅ | 안정성 정책 문서화 |
| Breaking Change 금지 | ✅ | 하위 호환성 보장 시작 |

#### v0.87: Bootstrap 강화 ✅ 완료

| 태스크 | 상태 | 설명 |
|--------|------|------|
| --fast-compile 플래그 | ✅ | opt 패스 스킵으로 3x 빠른 컴파일 |
| Bootstrap 성능 최적화 | ✅ | Stage 1: 1.7s → 0.54s (< 1.0s 달성!) |
| Bootstrap 안정성 | ✅ | Winsock2 링크 수정, 3-Stage 검증 통과 |
| 크로스 컴파일 문서화 | ✅ | CROSS_COMPILATION.md 실전 가이드 |

#### v0.88: Self-Hosting 완성 📋 진행 중

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
| 컴파일러 100% BMB 전환 | 📋 | Rust 의존성 제거 (현재: IR 생성만 BMB, native build는 스크립트 의존) |
| 개발 도구 BMB 전환 | 📋 | gotgan, LSP, Formatter |

#### v0.89: 내부 품질 게이트

| 태스크 | 설명 |
|--------|------|
| 코드 커버리지 > 80% | 테스트 범위 확대 |
| 퍼징 테스트 | libFuzzer 기반 입력 검증 |
| 메모리 안전성 검증 | AddressSanitizer 통과 |

---

### Phase v0.90-v0.93: Beta (Real-World Validation)

> **목표**: 실제 프로젝트 검증, 외부 사용자 피드백, 생태계 성숙

#### v0.90: Dogfooding - 컴파일러 도구

| 프로젝트 | 설명 |
|----------|------|
| bmb-fmt | BMB 포매터 (BMB로 재작성) |
| bmb-lint | BMB 린터 (BMB로 재작성) |
| bmb-doc | 문서 생성기 |

#### v0.91: Dogfooding - 시스템 도구

| 프로젝트 | 설명 |
|----------|------|
| bmb-build | 빌드 시스템 |
| bmb-test | 테스트 프레임워크 |
| bmb-bench | 벤치마크 도구 |

#### v0.92: 생태계 확장

| 태스크 | 설명 |
|--------|------|
| 패키지 30개+ | 핵심 라이브러리 확충 |
| 문서화 완성 | API 레퍼런스, 튜토리얼 |
| 예제 프로젝트 | 실용적 샘플 코드 |

#### v0.93: 외부 사용자 테스트

| 태스크 | 설명 |
|--------|------|
| Early Adopter 프로그램 | 외부 개발자 초대 |
| 피드백 수집 | 이슈 트래킹, 개선 |
| 마이그레이션 가이드 | 버전 업그레이드 문서 |

---

### Phase v0.94-v0.97: Pre-RC (Final Validation)

> **목표**: 최종 검증, 문서 완성, 릴리스 준비

#### v0.94: 성능 최종 검증

| 태스크 | 설명 |
|--------|------|
| 벤치마크 Gate 100% | 모든 벤치마크 PASS |
| 성능 회귀 0 | 이전 버전 대비 개선만 |
| 메모리 사용량 최적화 | 컴파일러 메모리 효율 |

#### v0.95: 플랫폼 검증

| 태스크 | 설명 |
|--------|------|
| Windows x64 완전 지원 | MSVC/MinGW 호환 |
| Linux x64/ARM64 | 주요 배포판 테스트 |
| macOS x64/ARM64 | Apple Silicon 지원 |

#### v0.96: 문서 완성

| 태스크 | 설명 |
|--------|------|
| Language Specification | 공식 언어 스펙 |
| Standard Library Reference | API 문서 |
| Compiler Internals | 아키텍처 문서 |

#### v0.97: 릴리스 준비

| 태스크 | 설명 |
|--------|------|
| 릴리스 노트 | 변경 사항 정리 |
| 설치 가이드 | 플랫폼별 설치 방법 |
| CI/CD 파이프라인 | 자동화된 릴리스 |

---

### Phase v0.98-v0.99: Release Candidate

> **목표**: 버그 수정만, 새 기능 없음

#### v0.98: RC1

| 조건 | 설명 |
|------|------|
| Feature Freeze | 기능 추가 완전 중단 |
| Critical Bug Only | P0 버그만 수정 |
| 2주 안정화 기간 | 집중 테스트 |

#### v0.99: RC2 (Final)

| 조건 | 설명 |
|------|------|
| Showstopper Only | 출시 차단 버그만 수정 |
| 문서 최종 검토 | 오타/오류 수정 |
| 릴리스 승인 | 커뮤니티 검증 |

---

## 벤치마크 현황 (v0.60.251)

| 판정 | 개수 | 비율 |
|------|------|------|
| ✅ BMB > C | 18개 | 60% |
| ✅ BMB ≈ C (≤1.10x) | 10개 | 33% |
| ⚠️ BMB < C (>1.10x) | 2개 | 7% |

**⚠️ 남은 회귀 (LLVM 한계):**

| 벤치마크 | 비율 | 원인 |
|----------|------|------|
| spectral_norm | 1.72x | GCC fast-math |
| matrix_multiply | 1.41x | LLVM 벡터화 한계 |

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
v0.70-81 Concurrency Primitives ✅ ────────────────────
         │  스레드, 채널, async/await, select 완료
         ▼
v0.82-85 Concurrency 완성 ✅ ─────────────────────────
         │  select!, async I/O, thread pool, scoped threads
         ▼
v0.86-89 Alpha (Feature Freeze) ──────────────────────
         │  v0.88.4 (현재): 언어 동결, Arena 안정화, 테스트 프레임워크
         ▼
v0.90-93 Beta (Real-World) ───────────────────────────
         │  Dogfooding, 생태계, 외부 테스트
         ▼
v0.94-97 Pre-RC (Final Validation) ───────────────────
         │  성능/플랫폼 검증, 문서 완성
         ▼
v0.98-99 Release Candidate ★ ─────────────────────────
         버그 수정만, 커뮤니티 검증 후 릴리스
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

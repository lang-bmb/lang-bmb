# BMB Roadmap

> 목표: 완전히 준비된 프로그래밍 언어 - Rust 의존성 제거, 성능 검증, 생태계 구축

---

## 현재 상태 (2026-02-06)

| 항목 | 상태 | 비고 |
|------|------|------|
| **버전** | v0.81 | Channel Iteration |
| **Bootstrap** | ✅ 3-Stage 완료 | Stage 1: ~1.5s |
| **Benchmarks** | ✅ 18/30 BMB > C | 60% C보다 빠름 |
| **Tests** | ✅ 91개 통과 | 68 + 23 |

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

## Concurrency Primitives (v0.70 ~ v0.81) ✅

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

---

## 🎯 진행 중 & 남은 작업

### Phase v0.82: select! 매크로

**목표**: 여러 채널/퓨처에서 동시 대기

```bmb
select! {
    value = rx1.recv() => { /* handle rx1 */ }
    value = rx2.recv() => { /* handle rx2 */ }
    _ = timeout(100) => { /* timeout */ }
}
```

| ID | 태스크 | 우선순위 |
|----|--------|----------|
| 82.1 | select! 매크로 문법 설계 | P0 |
| 82.2 | 파서 + AST 확장 | P0 |
| 82.3 | MIR 다중 대기 명령 | P0 |
| 82.4 | 런타임 구현 (epoll/IOCP) | P0 |
| 82.5 | 테스트 + 문서 | P1 |

---

### Phase v0.83: 비동기 I/O

**목표**: Non-blocking 파일/네트워크 연산

| ID | 태스크 | 우선순위 |
|----|--------|----------|
| 83.1 | AsyncFile API 설계 | P0 |
| 83.2 | AsyncSocket API | P0 |
| 83.3 | 이벤트 루프 통합 | P0 |
| 83.4 | async/await 연동 | P0 |

---

### Phase v0.84: Thread Pool

**목표**: Work-stealing 스케줄러

| ID | 태스크 | 우선순위 |
|----|--------|----------|
| 84.1 | ThreadPool API 설계 | P0 |
| 84.2 | Work-stealing 큐 | P0 |
| 84.3 | spawn() 태스크 제출 | P0 |
| 84.4 | 벤치마크 검증 | P1 |

---

### Phase v0.85+: 성능 최적화

| 태스크 | 설명 |
|--------|------|
| Atomic fence 최적화 | 불필요한 메모리 배리어 제거 |
| Channel 버퍼링 최적화 | Lock-free 큐 검토 |
| Future polling 최적화 | 효율적 waker 구현 |

---

## v0.58 Release Candidate 조건

| 조건 | 상태 |
|------|------|
| stdlib API 확정 | ✅ |
| 에러 메시지 | ✅ |
| 개발 도구 (LSP, Formatter, Linter) | ✅ |
| 자체 컴파일 (3-Stage) | ✅ |
| 성능 검증 (Gate #3.1) | ✅ 13/14 |
| 생태계 (14+ 패키지) | ✅ |
| 보안 감사 | ✅ |
| **Concurrency Primitives** | ✅ v0.81 |
| select! 매크로 | 📋 v0.82 |
| 비동기 I/O | 📋 v0.83 |

---

## 벤치마크 현황 (v0.60.251)

| 판정 | 개수 | 비율 |
|------|------|------|
| ✅ BMB > C | 18개 | 60% |
| ✅ BMB ≈ C (≤1.10x) | 10개 | 33% |
| ⚠️ BMB < C (>1.10x) | 2개 | 7% |

**⚠️ 남은 회귀 (근본적 한계):**

| 벤치마크 | 비율 | 원인 |
|----------|------|------|
| spectral_norm | 1.72x | GCC fast-math |
| matrix_multiply | 1.41x | LLVM 벡터화 한계 |

---

## 버전 정책

| 버전 유형 | 형식 | 설명 |
|-----------|------|------|
| Major | vX.0.0 | 커뮤니티 검증 후 수작업 릴리스 |
| Minor | v0.X.0 | 로드맵 계획 |
| Patch | v0.X.Y | 버그 수정 |

---

## 타임라인

```
v0.81 (현재) ─────────────────────────────────────────
         │  Concurrency primitives 완료
         ▼
v0.82 select! ────────────────────────────────────────
         │  다중 채널/퓨처 대기
         ▼
v0.83 Async I/O ──────────────────────────────────────
         │  Non-blocking 파일/네트워크
         ▼
v0.84 Thread Pool ────────────────────────────────────
         │  Work-stealing 스케줄러
         ▼
v0.85+ 최적화 ────────────────────────────────────────
         │  Atomic, Channel, Future 최적화
         ▼
v0.58 Release Candidate ★ ────────────────────────────
         │
         ▼
v1.0 ★★★ ─────────────────────────────────────────────
         커뮤니티 검증 완료 후 정식 릴리스
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

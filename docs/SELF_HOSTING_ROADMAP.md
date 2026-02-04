# BMB 완전 Self-Hosting 로드맵

## 목표

Rust와 동등한 수준의 self-hosting 달성:
- **외부 의존성 없이** BMB 컴파일러가 BMB 소스를 네이티브 바이너리로 컴파일
- LLVM, GCC, Clang 없이 독립 동작
- C 런타임 없이 자체 런타임으로 동작

---

## 현재 상태 (v0.60.251)

```
BMB 소스 → [BMB 컴파일러] → LLVM IR → [LLVM opt] → [LLVM llc] → [시스템 링커] → 실행파일
              ↑ BMB로 작성          ↑ 외부 의존         ↑ 외부 의존      ↑ 외부 의존
```

| 구성요소 | 상태 | 의존성 |
|----------|------|--------|
| 프론트엔드 (Lexer, Parser, Type Checker) | ✅ BMB | 없음 |
| MIR 생성 | ✅ BMB | 없음 |
| LLVM IR 생성 | ✅ BMB | 없음 |
| IR 최적화 | ❌ LLVM opt | LLVM |
| 네이티브 코드젠 | ❌ LLVM llc/clang | LLVM |
| 링킹 | ❌ gcc/clang/ld | 시스템 |
| 런타임 | ❌ C | libc |

---

## Phase 1: 자체 코드 생성기 (예상: 3-6개월)

### 1.1 x86-64 어셈블러 (BMB로 작성)

**목표**: MIR → x86-64 기계어 직접 생성

```
MIR → [BMB 코드젠] → x86-64 기계어 (.o)
```

**작업 항목**:

| 작업 | 설명 | 난이도 |
|------|------|--------|
| x86-64 인코딩 라이브러리 | 명령어 → 바이트 인코딩 | High |
| 레지스터 할당기 | Linear scan 또는 graph coloring | High |
| 명령어 선택 | MIR → x86-64 패턴 매칭 | Medium |
| 스택 프레임 관리 | 함수 프롤로그/에필로그 | Medium |
| 호출 규약 | System V AMD64 ABI | Medium |

**마일스톤**:
- [ ] 정수 연산 (add, sub, mul, div)
- [ ] 메모리 연산 (load, store)
- [ ] 제어 흐름 (jmp, jcc, call, ret)
- [ ] 함수 호출 (ABI 준수)
- [ ] 부동소수점 (SSE/AVX)

### 1.2 기본 최적화 패스

**목표**: LLVM opt 대체

| 최적화 | 현재 | 목표 |
|--------|------|------|
| 상수 폴딩 | ✅ MIR | 유지 |
| 죽은 코드 제거 | ✅ MIR | 유지 |
| 인라이닝 | ✅ MIR | 유지 |
| 레지스터 승격 | ❌ LLVM | BMB 구현 |
| 루프 최적화 | ❌ LLVM | BMB 구현 |
| 피프홀 최적화 | ❌ LLVM | BMB 구현 |

---

## Phase 2: 자체 링커 (예상: 2-3개월)

### 2.1 오브젝트 파일 생성

**목표**: ELF/PE/Mach-O 직접 생성

| 플랫폼 | 포맷 | 우선순위 |
|--------|------|----------|
| Linux | ELF64 | High |
| Windows | PE32+ | High |
| macOS | Mach-O 64 | Medium |

**작업 항목**:
- [ ] ELF 헤더/섹션 생성
- [ ] 심볼 테이블 관리
- [ ] 재배치 처리
- [ ] PE 헤더 생성 (Windows)

### 2.2 정적 링커

**목표**: 여러 .o 파일 + 런타임 → 실행파일

```
[a.o] + [b.o] + [runtime.o] → [BMB 링커] → executable
```

**작업 항목**:
- [ ] 심볼 해석 (symbol resolution)
- [ ] 섹션 병합
- [ ] 재배치 적용
- [ ] 실행파일 헤더 생성

---

## Phase 3: 자체 런타임 (예상: 2-4개월)

### 3.1 C 런타임 대체

**현재 C 런타임 함수들**:

```c
// 문자열
bmb_string_new, bmb_string_concat, bmb_string_eq, ...

// 메모리
malloc, free, calloc, realloc

// I/O
printf, fopen, fread, fwrite, ...

// 시스템
exit, getenv, ...
```

**BMB로 재작성**:

| 카테고리 | 현재 | 목표 |
|----------|------|------|
| 문자열 | C | BMB + syscall |
| 메모리 할당 | libc malloc | BMB 할당기 (mmap/VirtualAlloc) |
| 파일 I/O | libc | BMB + syscall |
| 콘솔 출력 | printf | BMB + syscall |

### 3.2 시스템 콜 래퍼

**Linux syscalls (직접 호출)**:
```bmb
// write(fd, buf, count)
fn sys_write(fd: i64, buf: *u8, count: i64) -> i64 = {
    syscall(1, fd, buf as i64, count)  // syscall number 1 = write
};
```

**Windows API (kernel32.dll)**:
```bmb
// WriteFile via LoadLibrary/GetProcAddress 또는 정적 임포트
@extern("kernel32.dll")
fn WriteFile(handle: i64, buffer: *u8, size: i32, written: *i32, overlapped: i64) -> i32;
```

### 3.3 메모리 할당기

**구현 옵션**:

| 할당기 | 복잡도 | 성능 |
|--------|--------|------|
| Bump allocator | Low | 해제 불가 |
| Free list | Medium | 보통 |
| Buddy system | Medium | 좋음 |
| mimalloc 스타일 | High | 최고 |

**최소 구현**:
```bmb
fn bmb_malloc(size: i64) -> i64 = {
    // Linux: mmap(NULL, size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0)
    // Windows: VirtualAlloc(NULL, size, MEM_COMMIT|MEM_RESERVE, PAGE_READWRITE)
};
```

---

## Phase 4: 표준 라이브러리 확장 (예상: 3-6개월)

### 4.1 핵심 모듈

| 모듈 | 기능 | 우선순위 |
|------|------|----------|
| `core` | 기본 타입, 연산 | Critical |
| `alloc` | 메모리 할당 | Critical |
| `io` | 파일/콘솔 I/O | High |
| `string` | 문자열 처리 | High |
| `collections` | Vec, HashMap, etc. | High |
| `fs` | 파일시스템 | Medium |
| `net` | 네트워킹 | Medium |
| `thread` | 멀티스레딩 | Medium |
| `sync` | 동기화 프리미티브 | Medium |

### 4.2 플랫폼 추상화 레이어

```
┌─────────────────────────────┐
│      BMB 표준 라이브러리      │
├─────────────────────────────┤
│    플랫폼 추상화 레이어 (PAL)  │
├──────────┬──────────┬───────┤
│  Linux   │ Windows  │ macOS │
│ syscall  │ Win32    │ Mach  │
└──────────┴──────────┴───────┘
```

---

## Phase 5: 빌드 시스템 자체 호스팅 (예상: 1-2개월)

### 5.1 Gotgan 패키지 매니저 (BMB로 재작성)

**현재**: Rust로 작성됨
**목표**: BMB로 재작성

| 기능 | 설명 |
|------|------|
| 의존성 해석 | SAT solver 또는 greedy |
| 패키지 다운로드 | HTTP 클라이언트 |
| 빌드 오케스트레이션 | 의존성 그래프 기반 빌드 |

### 5.2 부트스트랩 체인

```
Stage 0: 최소 BMB (C 또는 어셈블리로 수동 작성)
    ↓
Stage 1: Stage 0으로 컴파일한 BMB 컴파일러
    ↓
Stage 2: Stage 1로 컴파일한 BMB 컴파일러
    ↓
Stage 3: Stage 2로 컴파일 (Fixed Point 검증)
```

---

## 전체 타임라인

```
2026 Q1  ──┬── Phase 1.1: x86-64 코드젠 기초
           │
2026 Q2  ──┼── Phase 1.2: 기본 최적화
           │
2026 Q3  ──┼── Phase 2: 자체 링커
           │
2026 Q4  ──┼── Phase 3: 자체 런타임
           │
2027 Q1  ──┼── Phase 4: 표준 라이브러리
           │
2027 Q2  ──┴── Phase 5: 빌드 시스템
```

**예상 총 기간**: 12-18개월

---

## 대안적 접근법

### Option A: LLVM 유지 + 정적 링킹

LLVM을 정적으로 링킹하여 단일 바이너리 배포:

| 장점 | 단점 |
|------|------|
| 개발 시간 단축 | 바이너리 크기 증가 (~100MB+) |
| 검증된 최적화 | 여전히 LLVM 의존 |
| 멀티 아키텍처 지원 | 빌드 복잡성 |

### Option B: QBE 백엔드

[QBE](https://c9x.me/compile/) - 경량 컴파일러 백엔드:

| 장점 | 단점 |
|------|------|
| 작은 코드베이스 (~15K LOC) | x86-64/ARM64만 지원 |
| BMB로 포팅 가능 | LLVM보다 낮은 최적화 품질 |
| 빠른 컴파일 | |

### Option C: Cranelift 백엔드

Rust의 대안 코드젠으로 사용되는 Cranelift:

| 장점 | 단점 |
|------|------|
| Rust 에코시스템 | Rust 의존성 유지 |
| 좋은 성능 | BMB로 포팅 어려움 |
| JIT 지원 | |

---

## 권장 전략

### 단기 (6개월)

1. **Phase 1 집중**: x86-64 코드젠 구현
2. **LLVM 최적화는 유지**: 성능 보장
3. **점진적 교체**: 기능별로 LLVM 대체

### 중기 (12개월)

1. **자체 링커 완성**: ELF/PE 생성
2. **런타임 BMB 재작성**: syscall 직접 호출
3. **부트스트랩 체인 완성**

### 장기 (18개월+)

1. **LLVM 완전 제거**
2. **표준 라이브러리 완성**
3. **크로스 컴파일 지원**

---

## 성공 지표

| 지표 | 현재 | 목표 |
|------|------|------|
| 외부 컴파일러 의존성 | LLVM, GCC | 없음 |
| 외부 런타임 의존성 | libc | 없음 (syscall 직접) |
| 부트스트랩 | 3-stage (LLVM 경유) | 3-stage (순수 BMB) |
| 바이너리 크기 | N/A | < 1MB (컴파일러) |
| 컴파일 속도 | ~2초 (bootstrap) | < 5초 |
| 코드 품질 (vs LLVM) | 100% | > 80% |

---

## 참고 자료

### Self-Hosting 컴파일러 사례

| 언어 | 코드젠 | 런타임 | 링커 |
|------|--------|--------|------|
| Rust | LLVM | 자체 (libc 래퍼) | 시스템 |
| Go | 자체 | 자체 | 자체 |
| Zig | LLVM + 자체 | 자체 | 자체 |
| D | LLVM + 자체 | 자체 | 시스템 |
| Nim | C 변환 | C | 시스템 |

### 추천 학습 자료

1. **[Crafting Interpreters](https://craftinginterpreters.com/)** - 컴파일러 기초
2. **[Engineering a Compiler](https://www.elsevier.com/books/engineering-a-compiler/cooper/978-0-12-815412-0)** - 코드젠, 최적화
3. **[Linkers and Loaders](https://linker.iecc.com/)** - 링커 구현
4. **[QBE 소스코드](https://c9x.me/git/qbe.git/)** - 경량 백엔드 참고
5. **[chibicc](https://github.com/rui314/chibicc)** - 작은 C 컴파일러

---

## 다음 단계

1. **결정 필요**: Option A/B/C 중 선택 또는 자체 구현
2. **PoC 작성**: 간단한 x86-64 코드젠 프로토타입
3. **팀 구성**: 백엔드/런타임/표준라이브러리 담당자

이 로드맵은 BMB가 "AI 네이티브 시스템 언어"로서 완전한 독립성을 갖추기 위한 장기 계획입니다.

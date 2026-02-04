# BMB 로드맵: 표준 라이브러리 → 패키지 매니저 → Self-Hosting

## 목표

Rust와 동등한 수준의 언어 생태계 구축:
1. **표준 라이브러리**: 별도 import 없이 사용 가능한 핵심 라이브러리
2. **패키지 매니저**: Gotgan을 통한 의존성 관리
3. **Self-Hosting**: 런타임을 BMB로 재작성 (LLVM 유지)

---

## 현재 상태 (v0.60.251)

| 구성요소 | 상태 | 비고 |
|----------|------|------|
| 컴파일러 (프론트엔드) | ✅ BMB | Self-hosted |
| 코드젠 | ✅ LLVM | 유지 예정 |
| 링커 | ✅ 시스템 | 유지 예정 |
| 런타임 | ❌ C | BMB 재작성 필요 |
| 표준 라이브러리 | 🔄 초기 | `packages/` 존재하나 미연동 |
| 패키지 매니저 | 🔄 Rust | BMB 재작성 필요 |

---

## Phase 1: 표준 라이브러리 완성 (3-4개월)

### 1.1 자동 로드 시스템 구현

**목표**: `packages/` 디렉토리의 표준 라이브러리를 별도 import 없이 사용

```bmb
// 현재: 수동 include 필요
@include "bmb-core/src/lib.bmb"
let x = abs(-42);

// 목표: 자동 사용 가능
let x = abs(-42);  // bmb-core 자동 로드
```

**구현 방안**:

| 방안 | 설명 | 복잡도 |
|------|------|--------|
| **A. 프리루드** | 컴파일 시 자동 prepend | Low |
| **B. 암시적 import** | 심볼 해석 시 자동 탐색 | Medium |
| **C. 컴파일러 내장** | 표준 라이브러리 빌트인 | High |

**권장**: 방안 A (프리루드) - 가장 간단하고 예측 가능

### 1.2 핵심 표준 라이브러리 모듈

| 모듈 | 기능 | 우선순위 | 상태 |
|------|------|----------|------|
| `bmb-core` | Unit, Never, abs, min, max | Critical | 🔄 초기 |
| `bmb-option` | Option<T>, Some, None | Critical | 🔄 초기 |
| `bmb-result` | Result<T,E>, Ok, Err | Critical | 🔄 초기 |
| `bmb-traits` | 핵심 트레이트 | High | 🔄 초기 |
| `bmb-iter` | Iterator | High | 🔄 초기 |
| `bmb-string` | String 확장 메서드 | High | ❌ 미작성 |
| `bmb-collections` | Vec, HashMap | High | ❌ 미작성 |
| `bmb-io` | 파일/콘솔 I/O | Medium | ❌ 미작성 |

### 1.3 마일스톤

```
Week 1-2:  프리루드 시스템 구현
Week 3-4:  bmb-core, bmb-option 완성
Week 5-6:  bmb-result, bmb-traits 완성
Week 7-8:  bmb-iter 완성
Week 9-10: bmb-string, bmb-collections 완성
Week 11-12: bmb-io 완성, 통합 테스트
```

---

## Phase 2: Gotgan 패키지 매니저 강화 (2-3개월)

### 2.1 현재 Gotgan 상태

```
ecosystem/gotgan/     # Rust로 작성됨
├── src/
│   ├── main.rs
│   ├── config.rs     # Gotgan.toml 파싱
│   ├── resolver.rs   # 의존성 해석
│   ├── registry.rs   # 패키지 레지스트리
│   └── build.rs      # 빌드 오케스트레이션
```

### 2.2 강화 작업

| 기능 | 현재 | 목표 |
|------|------|------|
| 로컬 의존성 | ✅ | 유지 |
| 버전 해석 | ✅ | 유지 |
| 레지스트리 | 🔄 로컬만 | 원격 레지스트리 |
| 빌드 캐시 | ❌ | 증분 빌드 |
| Lock 파일 | ✅ | 유지 |
| 워크스페이스 | ❌ | 모노레포 지원 |

### 2.3 원격 레지스트리

```toml
# Gotgan.toml
[dependencies]
bmb-json = "0.2.0"           # 원격 레지스트리에서 다운로드
bmb-http = { git = "https://github.com/..." }  # Git 의존성
bmb-local = { path = "../my-lib" }             # 로컬 의존성
```

**레지스트리 옵션**:

| 옵션 | 설명 | 복잡도 |
|------|------|--------|
| GitHub Releases | 간단, 무료 | Low |
| 자체 레지스트리 | 완전 제어 | High |
| crates.io 스타일 | 표준화 | Medium |

### 2.4 마일스톤

```
Week 1-2:  증분 빌드 구현
Week 3-4:  원격 레지스트리 (GitHub Releases)
Week 5-6:  워크스페이스 지원
Week 7-8:  문서화 및 안정화
```

---

## Phase 3: Self-Hosting 완성 (2-3개월)

### 3.1 목표: Rust 동등 수준

| 구성요소 | Rust | BMB 목표 |
|----------|------|----------|
| 프론트엔드 | 자체 | ✅ BMB (완료) |
| 코드젠 | LLVM | ✅ LLVM (유지) |
| 런타임 | 자체 | ⭐ BMB 재작성 |
| 링커 | 시스템 | ✅ 시스템 (유지) |

### 3.2 런타임 재작성 범위

**현재 C 런타임 (`bmb/runtime/`)**:

```c
// string.c - 문자열 연산
BmbString* bmb_string_new(const char* data, int64_t len);
BmbString* bmb_string_concat(BmbString* a, BmbString* b);
int64_t bmb_string_eq(BmbString* a, BmbString* b);

// memory.c - 메모리 관리
// (libc malloc/free 래퍼)

// io.c - 입출력
void println(int64_t n);
void println_str(BmbString* s);
BmbString* bmb_read_file(BmbString* path);

// stringbuilder.c - StringBuilder
int64_t bmb_sb_new();
int64_t bmb_sb_push(int64_t sb, BmbString* s);
BmbString* bmb_sb_build(int64_t sb);
```

**BMB로 재작성**:

```bmb
// runtime/string.bmb
struct BmbString {
    data: *u8,
    len: i64
}

fn bmb_string_new(data: *u8, len: i64) -> *BmbString = {
    let s = malloc(16) as *BmbString;
    set s.data = data;
    set s.len = len;
    s
};

fn bmb_string_concat(a: *BmbString, b: *BmbString) -> *BmbString = {
    let new_len = a.len + b.len;
    let new_data = malloc(new_len) as *u8;
    memcpy(new_data, a.data, a.len);
    memcpy(new_data + a.len, b.data, b.len);
    bmb_string_new(new_data, new_len)
};
```

### 3.3 의존성 정리

| 현재 의존 | 재작성 후 |
|----------|----------|
| libc (malloc, free) | 유지 (시스템 제공) |
| libc (printf, fopen) | BMB 래퍼 |
| string.c | BMB |
| stringbuilder.c | BMB |
| io.c | BMB |

### 3.4 마일스톤

```
Week 1-2:  string.bmb 구현
Week 3-4:  stringbuilder.bmb 구현
Week 5-6:  io.bmb 구현
Week 7-8:  통합 및 부트스트랩 검증
```

---

## 전체 타임라인

```
2026 Q1 (현재)
    │
    ├─ Phase 1: 표준 라이브러리 ──────────────────┐
    │   Week 1-12                                 │
    │                                             │
2026 Q2                                           │
    │                                             │
    ├─ Phase 2: Gotgan 강화 ─────────────────────┤
    │   Week 1-8                                  │
    │                                             │
2026 Q3                                           │
    │                                             │
    ├─ Phase 3: Self-Hosting ────────────────────┤
    │   Week 1-8                                  │
    │                                             │
2026 Q4                                           │
    │                                             │
    └─ 완료: Rust 동등 수준 달성 ─────────────────┘
```

**총 예상 기간: 7-10개월**

---

## 우선순위 정리

| 순위 | Phase | 이유 |
|------|-------|------|
| 1 | 표준 라이브러리 | 사용자 경험 직접 영향 |
| 2 | Gotgan 강화 | 생태계 성장 기반 |
| 3 | Self-Hosting | 기술적 완성도 |

---

## 성공 지표

### Phase 1 완료 조건

- [ ] `abs(-42)` 가 import 없이 동작
- [ ] `Option<T>`, `Result<T,E>` 제네릭 사용 가능
- [ ] Iterator 패턴 사용 가능
- [ ] Vec, HashMap 기본 컬렉션 사용 가능

### Phase 2 완료 조건

- [ ] `gotgan add bmb-json` 으로 원격 패키지 설치
- [ ] 증분 빌드로 재컴파일 시간 50% 감소
- [ ] 워크스페이스에서 여러 패키지 동시 관리

### Phase 3 완료 조건

- [ ] 런타임이 100% BMB로 작성됨
- [ ] 3-Stage 부트스트랩 통과 (Fixed Point)
- [ ] 기존 벤치마크 성능 유지 (회귀 없음)

---

## 리스크 및 완화

| 리스크 | 심각도 | 완화 방안 |
|--------|--------|-----------|
| 제네릭 구현 복잡도 | High | 단형화(monomorphization) 우선 |
| 원격 레지스트리 운영 | Medium | GitHub Releases 활용 |
| 런타임 재작성 버그 | Medium | 기존 C 버전과 출력 비교 |

---

## 비교: Rust 생태계

| 구성요소 | Rust | BMB 목표 |
|----------|------|----------|
| 표준 라이브러리 | `std` | `packages/bmb-*` |
| 패키지 매니저 | Cargo | Gotgan |
| 패키지 레지스트리 | crates.io | gotgan.io (또는 GitHub) |
| 코드젠 | LLVM | LLVM |
| 런타임 | 자체 | 자체 (BMB) |

---

## 다음 단계

1. **즉시**: Phase 1.1 프리루드 시스템 설계
2. **이번 주**: bmb-core 함수 목록 확정
3. **다음 주**: 프리루드 프로토타입 구현

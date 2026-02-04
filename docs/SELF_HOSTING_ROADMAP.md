# BMB 로드맵: 표준 라이브러리 → 패키지 매니저 → Self-Hosting

## 목표

Rust와 동등한 수준의 언어 생태계 구축:
1. **표준 라이브러리**: 별도 import 없이 사용 가능한 핵심 라이브러리
2. **패키지 매니저**: Gotgan을 통한 의존성 관리
3. **Self-Hosting**: 런타임을 BMB로 재작성 (LLVM 유지)

---

## 현재 상태 (v0.60.262)

| 구성요소 | 상태 | 비고 |
|----------|------|------|
| 컴파일러 (프론트엔드) | ✅ BMB | Self-hosted |
| 코드젠 | ✅ LLVM | 유지 예정 |
| 링커 | ✅ 시스템 | 유지 예정 |
| 런타임 | ❌ C | BMB 재작성 필요 |
| 표준 라이브러리 | ✅ 완료 | Phase 1 완료: 8개 패키지 |
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
| `bmb-core` | Unit, Never, Pair<A,B>, abs, min, max | Critical | ✅ 완료 (v0.60.261) |
| `bmb-option` | Option<T>, Some, None | Critical | ✅ 완료 (v0.2.0) |
| `bmb-result` | Result<T,E>, Ok, Err | Critical | ✅ 완료 (v0.2.0) |
| `bmb-traits` | Ordering, 핵심 트레이트 | High | ✅ 완료 (v0.60.262) |
| `bmb-iter` | Range, Repeat, fibonacci, combinators | High | ✅ 완료 (v0.17.1) |
| `bmb-string` | String 확장 메서드 | High | ✅ 완료 (v0.60.262) |
| `bmb-collections` | HashMap, VecDeque, Stack | High | ✅ 완료 (v0.60.262) |
| `bmb-io` | 파일/콘솔 I/O | Medium | ✅ 완료 (v0.1.0) |

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
│   ├── fetcher.rs    # 패키지 다운로드
│   └── build.rs      # 빌드 오케스트레이션
```

### 2.2 설계 철학: Go 스타일 분산형

**중앙 레지스트리 없음** - Go 모듈처럼 소스에서 직접 가져옴:
- GitHub 리포지토리에서 직접 fetch
- 로컬 파일 경로 지원
- 버전은 Git 태그/브랜치로 관리

**Gotgan의 역할**:
| 역할 | 설명 |
|------|------|
| 패키지 인덱스 | 공개 패키지 검색/발견 (pkg.go.dev 처럼) |
| 의존성 관리 | 설치, 업데이트, 버전 해석 |
| 빌드 오케스트레이션 | 컴파일 순서, 증분 빌드 |
| **NOT** 호스팅 | 패키지 바이너리를 호스팅하지 않음 |

### 2.3 강화 작업

| 기능 | 현재 | 목표 |
|------|------|------|
| 로컬 의존성 | ✅ | 유지 |
| 버전 해석 | ✅ | 유지 |
| GitHub fetch | ✅ 완료 | shallow clone + ~/.gotgan/cache/ |
| 빌드 캐시 | ✅ 완료 | FNV-1a fingerprinting (v0.60.263) |
| Lock 파일 | ✅ | 유지 |
| 워크스페이스 | ✅ 완료 | [workspace] members = ["packages/*"] (v0.60.263) |
| 패키지 검색 | ❌ | 인덱스 서비스 |

### 2.4 패키지 소싱 (Go 스타일)

```toml
# Gotgan.toml
[dependencies]
# GitHub에서 직접 (권장)
bmb-json = "github.com/lang-bmb/bmb-json@v0.2.0"
bmb-http = "github.com/iyulab/bmb-http@main"

# Git URL 명시
bmb-custom = { git = "https://github.com/user/repo", tag = "v1.0.0" }
bmb-dev = { git = "https://github.com/user/repo", branch = "dev" }

# 로컬 경로
bmb-local = { path = "../my-lib" }
```

**소싱 우선순위**:

| 소스 | 형식 | 용도 |
|------|------|------|
| GitHub 단축 | `github.com/org/repo@version` | 공개 패키지 |
| Git URL | `{ git = "...", tag/branch = "..." }` | 사설 리포/특정 ref |
| 로컬 경로 | `{ path = "..." }` | 개발/테스트 |

### 2.5 패키지 인덱스 (선택)

중앙 레지스트리 대신 **검색 인덱스** 제공:

```bash
# 패키지 검색 (인덱스 조회)
gotgan search json
# → github.com/lang-bmb/bmb-json - JSON parser for BMB
# → github.com/user/fast-json - Fast JSON library

# 패키지 추가 (GitHub에서 직접 fetch)
gotgan add github.com/lang-bmb/bmb-json@v0.2.0
```

**인덱스 구현 옵션**:

| 옵션 | 설명 | 복잡도 |
|------|------|--------|
| GitHub Topics 크롤링 | 자동 수집 | Low |
| 정적 JSON 인덱스 | 수동 등록 | Low |
| 별도 인덱스 서비스 | 검색 API | Medium |

### 2.6 마일스톤

```
Week 1-2:  GitHub fetch 구현 (clone + checkout)
Week 3-4:  증분 빌드 구현
Week 5-6:  워크스페이스 지원
Week 7-8:  패키지 인덱스 + 문서화
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

- [x] `abs(-42)` 가 import 없이 동작 (v0.60.253 prelude 자동로드)
- [x] `Option<T>`, `Result<T,E>` 제네릭 사용 가능 (v0.60.261)
- [x] Iterator 패턴 사용 가능 (v0.17.1 bmb-iter)
- [x] Vec, HashMap 기본 컬렉션 사용 가능 (v0.60.262 bmb-collections)

### Phase 2 완료 조건

- [x] `gotgan add github.com/lang-bmb/bmb-json@v0.1.0` 으로 GitHub 패키지 설치 (v0.60.263 - Git clone 구현됨)
- [x] 증분 빌드 캐시 (v0.60.263 - BuildCache, FNV-1a fingerprinting)
- [x] 워크스페이스에서 여러 패키지 동시 관리 (v0.60.263 - [workspace] members glob)
- [ ] `gotgan search json` 으로 패키지 검색 가능

### Phase 3 완료 조건

- [ ] 런타임이 100% BMB로 작성됨
- [ ] 3-Stage 부트스트랩 통과 (Fixed Point)
- [ ] 기존 벤치마크 성능 유지 (회귀 없음)

---

## 리스크 및 완화

| 리스크 | 심각도 | 완화 방안 |
|--------|--------|-----------|
| 제네릭 구현 복잡도 | High | 단형화(monomorphization) 우선 |
| GitHub API 레이트 제한 | Low | 캐싱 + 인증 토큰 지원 |
| 런타임 재작성 버그 | Medium | 기존 C 버전과 출력 비교 |
| 버전 일관성 | Medium | Lock 파일 + 체크섬 검증 |

---

## 비교: 언어 생태계

| 구성요소 | Rust | Go | BMB 목표 |
|----------|------|-----|----------|
| 표준 라이브러리 | `std` | `std` | `packages/bmb-*` |
| 패키지 매니저 | Cargo | `go mod` | Gotgan |
| 패키지 소싱 | 중앙 (crates.io) | **분산 (GitHub)** | **분산 (GitHub/경로)** |
| 패키지 검색 | crates.io | pkg.go.dev | Gotgan 인덱스 |
| 코드젠 | LLVM | 자체 | LLVM |
| 런타임 | 자체 | 자체 | 자체 (BMB) |

**BMB는 Go 모델을 따름**: 중앙 레지스트리 없이 GitHub에서 직접 fetch

---

## 다음 단계

1. **즉시**: Phase 1.1 프리루드 시스템 설계
2. **이번 주**: bmb-core 함수 목록 확정
3. **다음 주**: 프리루드 프로토타입 구현

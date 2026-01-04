# Gotgan 2.0: AI-Native Package Manager Design

> 곳간(Gotgan) - BMB의 AI-네이티브 패키지 매니저
> "AI 에이전트가 코드를 탐색하고, 계약을 검증하며, 자동으로 통합하는 차세대 패키지 생태계"

---

## 1. 설계 철학

### 1.1 기존 패키지 매니저의 한계

| 기존 방식 | 한계점 | AI-Native 해결책 |
|-----------|--------|------------------|
| 소스 코드 중심 | AI가 의미 파악 어려움 | 계약 + 메타데이터 우선 |
| 버전 문자열 의존 | 호환성 수동 검증 | 계약 기반 자동 호환성 검사 |
| 빌드 결과물 분리 | 바이너리-소스 불일치 가능 | 통합 번들 포맷 (.bmbx) |
| 문서 분리 | 코드-문서 동기화 어려움 | 계약이 곧 문서 |
| 단일 타깃 | 플랫폼별 재빌드 필요 | LLVM + WASM 듀얼 타깃 |

### 1.2 핵심 원칙

1. **Contract-First**: 계약이 패키지의 일급 시민
2. **AI-Discoverable**: AI 에이전트가 직접 탐색 가능한 구조
3. **Unified Bundle**: 바이너리 + 소스 + 계약 + 메타데이터 통합
4. **Dual-Target**: LLVM 네이티브 + WASM 포터블 동시 지원
5. **Self-Describing**: 패키지가 스스로 사용법 설명

---

## 2. 패키지 포맷

### 2.1 BMBX 번들 포맷

```
package.bmbx (Unified Bundle)
├── manifest.toml          # 패키지 메타데이터
├── contracts.json         # 모든 계약의 JSON 표현
├── src/                   # 소스 코드 (.bmb)
├── bin/                   # 컴파일된 바이너리
│   ├── x86_64-linux/     # LLVM 네이티브
│   ├── aarch64-darwin/   # LLVM 네이티브
│   └── wasm32/           # WebAssembly
├── types.json             # 타입 시그니처 (AI 탐색용)
├── symbols.json           # 심볼 인덱스 (AI 탐색용)
└── docs/                  # 자동 생성 문서
```

### 2.2 계약 JSON 스키마

```json
{
  "package": "math",
  "version": "1.0.0",
  "functions": [
    {
      "name": "divide",
      "signature": "(a: i64, b: i64) -> i64",
      "preconditions": [
        { "expr": "b != 0", "message": "divisor must be non-zero" }
      ],
      "postconditions": [
        { "expr": "ret * b == a", "message": "division correctness" }
      ],
      "verified": true,
      "verification_time_ms": 42
    }
  ],
  "types": [
    {
      "name": "NonZero",
      "base": "i64",
      "refinement": "self != 0"
    }
  ]
}
```

### 2.3 AI 탐색 인덱스 (symbols.json)

```json
{
  "package": "string-utils",
  "exports": [
    {
      "name": "int_to_string",
      "kind": "function",
      "signature": "(n: i64) -> String",
      "description_inferred": "정수를 문자열로 변환",
      "complexity": "O(log n)",
      "pure": true,
      "contracts": ["pre: true", "post: parse(ret) == n"]
    }
  ],
  "semantic_tags": ["conversion", "string", "integer"],
  "ai_hints": {
    "use_when": ["정수를 출력할 때", "문자열 포맷팅"],
    "alternatives": ["std::fmt::format"],
    "common_patterns": ["int_to_string(x) + \" items\""]
  }
}
```

---

## 3. 핵심 기능

### 3.1 계약 기반 의존성 관리

```toml
# gotgan.toml
[dependencies]
math = { version = "^1.0", contracts = ["divide.pre: b != 0"] }

# 의존성 버전이 올라가도 계약이 유지되면 호환
# 계약이 변경되면 자동으로 경고/차단
```

**계약 호환성 검사:**
```
$ gotgan check --contracts
[math v1.0 → v1.1] ✓ 모든 계약 호환
[io v0.5 → v0.6] ⚠ write.pre 변경: "buf.len > 0" → "buf.len >= 0"
                   (약화됨 - 호환)
[parser v2.0 → v3.0] ✗ parse.post 변경 - 계약 강화 (비호환)
```

### 3.2 AI 에이전트 API

```bash
# 자연어 검색
$ gotgan search --ai "정수를 문자열로 바꾸는 함수"
→ string-utils::int_to_string (99% 관련성)
→ fmt::format (85% 관련성)

# 계약 기반 검색
$ gotgan search --contract "pre: x > 0, post: ret > x"
→ math::factorial (정확히 일치)
→ math::double (post 부분 일치)

# 코드 탐색 (MCP 통합)
$ gotgan explore math --symbols
→ 12 functions, 3 types, 45 contracts

# 사용 예제 생성
$ gotgan example math::divide
→ let result = divide(10, 3);  // result = 3
→ // Pre: b != 0 (✓ verified)
→ // Post: result * 3 == 10 (✓ verified)
```

### 3.3 단일 파일 빌드

```bash
# 모든 의존성을 하나의 .bmb 파일로 번들
$ gotgan bundle --single-file
→ dist/app.bundled.bmb (self-contained)

# 의존성 인라인 + 계약 보존
$ gotgan bundle --preserve-contracts
→ 계약 주석으로 보존됨

# WASM 타깃 단일 파일
$ gotgan bundle --target wasm32
→ dist/app.wasm (브라우저 실행 가능)
```

### 3.4 듀얼 타깃 빌드

```toml
# gotgan.toml
[targets]
native = ["x86_64-linux", "aarch64-darwin", "x86_64-windows"]
wasm = ["wasm32-unknown-unknown", "wasm32-wasi"]

[target.wasm32]
features = ["no-std", "browser"]
optimize = "size"

[target.native]
features = ["simd", "threads"]
optimize = "speed"
```

```bash
# 모든 타깃 빌드
$ gotgan build --all-targets
→ target/x86_64-linux/release/app
→ target/aarch64-darwin/release/app
→ target/wasm32/release/app.wasm

# 특정 타깃만
$ gotgan build --target wasm32
```

---

## 4. 패키지 등록 지침

### 4.1 부트스트랩 컴포넌트 패키지화

| 컴포넌트 | 패키지명 | 설명 | 우선순위 |
|----------|----------|------|----------|
| String 유틸리티 | `bmb-std/string` | int_to_string, char_to_string 등 | P0 |
| 파싱 유틸리티 | `bmb-std/parse` | skip_ws, find_char, starts_with 등 | P0 |
| MIR 타입 | `bmb-compiler/mir` | MIR 데이터 구조 | P1 |
| LLVM IR 생성 | `bmb-compiler/llvm` | LLVM IR 텍스트 생성 | P1 |
| Lexer | `bmb-compiler/lexer` | 토큰화 | P2 |
| Parser | `bmb-compiler/parser` | AST 파싱 | P2 |

### 4.2 패키지 등록 워크플로우

```bash
# 1. 패키지 생성
$ gotgan new --lib string-utils
$ cd string-utils

# 2. 코드 작성 + 계약 추가
$ edit src/lib.bmb

# 3. 계약 검증
$ gotgan verify
→ 15/15 contracts verified ✓

# 4. AI 인덱스 생성
$ gotgan index --ai
→ Generated symbols.json, types.json, contracts.json

# 5. 테스트
$ gotgan test

# 6. 번들 생성
$ gotgan pack --all-targets
→ string-utils-1.0.0.bmbx

# 7. 게시
$ gotgan publish
→ Published to registry.bmb-lang.org
```

### 4.3 품질 기준

```toml
# gotgan.toml 품질 섹션
[quality]
# 필수
contracts_coverage = 0.8      # 80% 이상 함수에 계약
verification_required = true  # 모든 계약 검증 필수

# 권장
tests_coverage = 0.7          # 70% 테스트 커버리지
documentation = true          # 문서 필수
examples = true               # 예제 필수

# AI 최적화
ai_hints = true               # AI 힌트 제공
semantic_tags = true          # 의미론적 태그
```

---

## 5. LLVM + WASM 듀얼 타깃 아키텍처

### 5.1 공통 파이프라인

```
BMB Source
    ↓
  Parser → AST
    ↓
  Type Checker
    ↓
  Contract Verifier (SMT)
    ↓
  MIR (공통 중간 표현)
    ↓
    ├─────────────────────────────┐
    ↓                             ↓
LLVM IR Generator           WASM IR Generator
    ↓                             ↓
 .ll 파일                    .wat 파일
    ↓                             ↓
  clang/lld                    wat2wasm
    ↓                             ↓
Native Binary                  .wasm
```

### 5.2 타깃별 특성

| 특성 | LLVM Native | WASM |
|------|-------------|------|
| 성능 | C와 동등 (-O3) | JS 대비 빠름 |
| 포터빌리티 | 플랫폼별 빌드 | 어디서나 실행 |
| 런타임 | OS 직접 호출 | WASI/브라우저 |
| 메모리 | 네이티브 스택 | 선형 메모리 |
| SIMD | 네이티브 SIMD | WASM SIMD |
| 스레드 | OS 스레드 | SharedArrayBuffer |

### 5.3 조건부 컴파일

```bmb
-- 타깃별 코드
#[cfg(target = "wasm32")]
fn print(s: String) -> unit =
    js_console_log(s);

#[cfg(target = "native")]
fn print(s: String) -> unit =
    libc_puts(s);

-- 공통 코드
fn greet(name: String) -> String =
    "Hello, " + name;
```

### 5.4 런타임 추상화

```bmb
-- runtime.bmb (공통 인터페이스)
type Runtime = trait {
    fn print(s: String) -> unit;
    fn read_line() -> String;
    fn current_time_ms() -> i64;
};

-- runtime_native.bmb
impl Runtime for NativeRuntime {
    fn print(s) = libc_puts(s);
    fn read_line() = libc_gets();
    fn current_time_ms() = clock_gettime();
};

-- runtime_wasm.bmb
impl Runtime for WasmRuntime {
    fn print(s) = js_console_log(s);
    fn read_line() = js_prompt();
    fn current_time_ms() = js_date_now();
};
```

---

## 6. 생태계 통합

### 6.1 MCP 서버 통합

```json
// gotgan-mcp-server
{
  "name": "gotgan-mcp",
  "tools": [
    {
      "name": "search_packages",
      "description": "Search BMB packages by natural language or contracts",
      "parameters": {
        "query": "string",
        "contract_filter": "optional<string>"
      }
    },
    {
      "name": "explore_package",
      "description": "Get package symbols, types, and contracts",
      "parameters": {
        "package": "string"
      }
    },
    {
      "name": "generate_example",
      "description": "Generate usage example for a function",
      "parameters": {
        "function": "string"
      }
    }
  ]
}
```

### 6.2 IDE 통합

```json
// .vscode/settings.json
{
  "bmb.gotgan.autoComplete": true,
  "bmb.gotgan.showContracts": true,
  "bmb.gotgan.aiSuggestions": true
}
```

### 6.3 CI/CD 통합

```yaml
# .github/workflows/bmb.yml
- name: Verify Contracts
  run: gotgan verify --strict

- name: Build All Targets
  run: gotgan build --all-targets

- name: Pack Bundle
  run: gotgan pack

- name: Publish
  run: gotgan publish --token ${{ secrets.GOTGAN_TOKEN }}
```

---

## 7. 로드맵 통합

### Phase 1: 기반 강화 (v0.10.x - 현재)

- [x] v0.10.12: Text-based LLVM IR Backend
- [ ] v0.10.13: 부트스트랩 컴포넌트 패키지화 준비
- [ ] v0.10.14: std/string 패키지 추출
- [ ] v0.10.15: std/parse 패키지 추출

### Phase 2: AI-Native 기반 (v0.11.x)

- [ ] v0.11.0: BMBX 번들 포맷 구현
- [ ] v0.11.1: contracts.json 생성기
- [ ] v0.11.2: symbols.json AI 인덱스
- [ ] v0.11.3: 계약 기반 의존성 검사
- [ ] v0.11.4: gotgan search --ai 구현
- [ ] v0.11.5: gotgan bundle --single-file

### Phase 3: 듀얼 타깃 (v0.12.x)

- [ ] v0.12.0: MIR → WASM IR 변환기
- [ ] v0.12.1: WASI 런타임 바인딩
- [ ] v0.12.2: 브라우저 런타임 바인딩
- [ ] v0.12.3: 조건부 컴파일 (#[cfg])
- [ ] v0.12.4: gotgan build --all-targets

### Phase 4: 생태계 완성 (v0.13.x)

- [ ] v0.13.0: gotgan-mcp-server
- [ ] v0.13.1: 레지스트리 자동화
- [ ] v0.13.2: 패키지 품질 점수
- [ ] v0.13.3: 의존성 그래프 시각화

### Phase 5: 자기 호스팅 (v0.14.x → v1.0)

- [ ] v0.14.0: gotgan을 BMB로 재작성
- [ ] v0.14.1: 패키지 레지스트리 BMB 재작성
- [ ] v1.0.0: 완전한 자기 호스팅 생태계

---

## 8. 부록

### A. 명령어 요약

| 명령어 | 설명 |
|--------|------|
| `gotgan new` | 새 프로젝트 생성 |
| `gotgan build` | 프로젝트 빌드 |
| `gotgan verify` | 계약 검증 |
| `gotgan test` | 테스트 실행 |
| `gotgan search` | 패키지 검색 (AI 지원) |
| `gotgan explore` | 패키지 심볼 탐색 |
| `gotgan bundle` | 단일 파일 번들 |
| `gotgan pack` | BMBX 패키지 생성 |
| `gotgan publish` | 레지스트리 게시 |
| `gotgan index` | AI 인덱스 생성 |

### B. 환경 변수

| 변수 | 설명 | 기본값 |
|------|------|--------|
| `GOTGAN_REGISTRY` | 레지스트리 URL | registry.bmb-lang.org |
| `GOTGAN_TOKEN` | 인증 토큰 | - |
| `GOTGAN_CACHE` | 캐시 디렉토리 | ~/.cache/gotgan |
| `GOTGAN_TARGET` | 기본 빌드 타깃 | 호스트 시스템 |

### C. 관련 문서

- [BMB 언어 명세](./SPECIFICATION.md)
- [MIR 설계](./MIR_DESIGN.md)
- [계약 시스템](./CONTRACTS.md)
- [LLVM 백엔드](./LLVM_BACKEND.md)

---

## D. 구현 현황 (v0.11.4-6)

> 2025년 1월 기준 Rust 구현 상태

### 구현 완료

| 기능 | 파일 | 설명 |
|------|------|------|
| **BMBX 번들 생성** | `bmbx.rs` | `gotgan bundle` 명령어 |
| **contracts.json** | `bmbx.rs` | 함수/타입 계약 추출 |
| **symbols.json** | `bmbx.rs` | AI 탐색용 심볼 인덱스 |
| **types.json** | `bmbx.rs` | 타입 시그니처 추출 |
| **AI 힌트 추론** | `bmbx.rs` | 함수명에서 설명/태그 자동 추론 |
| **계약 탐색** | `bmbx.rs` | `gotgan explore --contracts` |
| **심볼 탐색** | `bmbx.rs` | `gotgan explore --symbols` |
| **JSON 출력** | `bmbx.rs` | `gotgan explore --json` |
| **필터링** | `bmbx.rs` | `gotgan explore --filter` |
| **호환성 검사** | `bmbx.rs` | `gotgan compat` 명령어 |
| **계약 변경 감지** | `bmbx.rs` | Breaking/Compatible 분류 |

### 사용법

```bash
# BMBX 번들 생성
gotgan bundle

# 심볼 탐색
gotgan explore --symbols

# 계약 확인
gotgan explore --contracts

# AI용 JSON 출력
gotgan explore --json

# 패턴 필터링
gotgan explore --filter "parse"

# 버전 간 호환성 검사
gotgan compat --old v1.0/contracts.json --new v2.0/contracts.json
```

### 계약 호환성 규칙

| 변경 유형 | Breaking? | 설명 |
|-----------|-----------|------|
| pre 제거 | ❌ No | 더 관대해짐 (더 많은 입력 허용) |
| pre 추가 | ⚠️ Yes | 더 제한적 (기존 호출 실패 가능) |
| post 추가 | ❌ No | 더 많은 보장 |
| post 제거 | ⚠️ Yes | 보장 감소 (의존 코드 실패 가능) |
| 함수 제거 | ⚠️ Yes | 기존 코드 컴파일 실패 |
| 함수 추가 | ❌ No | 기존 코드 영향 없음 |

### 미구현 (계획)

| 기능 | 상태 | 비고 |
|------|------|------|
| `--single-file` | 미구현 | 단일 .bmbx 압축 번들 |
| 자연어 검색 | 미구현 | `gotgan search --ai` |
| 계약 기반 검색 | 미구현 | `gotgan search --contract` |
| 레지스트리 연동 | 미구현 | registry.bmb-lang.org |

# BMB Roadmap to v1.0.0-beta

> 목표: 완전히 준비된 프로그래밍 언어 - Rust 의존성 제거, 성능 검증, 생태계 구축

---

## 현재 상태 요약

| 버전 | 이름 | 상태 | 핵심 성과 |
|------|------|------|----------|
| v0.1-v0.30 | Foundation | ✅ 완료 | 언어 설계, 컴파일러, 부트스트랩 |
| v0.31-v0.37 | Maturity | ✅ 완료 | Stage 3, 벤치마크, 스펙 준수 |
| v0.38-v0.44 | Stabilization | ✅ 완료 | CI, 안정성, API 동결, 릴리스 준비 |
| **v0.45** | **Foundation Completion** | ✅ 완료 | **stdlib 확정, 도구 안정화, bmb lint 추가** |
| **v0.46** | **Independence** | ⏳ 진행중 | **Rust 제거, 자체 컴파일** |
| **v0.47** | **Performance** | 📋 계획 | **성능 Gate 통과** |
| **v0.48** | **Ecosystem** | 📋 계획 | **패키지, 크로스 컴파일** |
| **v0.49** | **Showcase** | 📋 계획 | **샘플 앱, 시나리오** |
| **v0.50** | **Final Verification** | 📋 계획 | **보안 감사, 최종 검증** |
| **v1.0.0-beta** | **Golden** | 🎯 목표 | **완전한 프로그래밍 언어** |

---

## v1.0.0-beta 필수 조건

### Exit Criteria (모두 충족 필수)

| 조건 | 설명 | 현재 상태 | 담당 페이즈 |
|------|------|----------|------------|
| **stdlib API 확정** | 표준 라이브러리 공개 API 안정화 | ⚠️ 진행중 | v0.45 |
| **에러 메시지** | 사용자 친화적 컴파일 에러 | ❌ 미완료 | v0.45 |
| **개발 도구** | LSP, Formatter, Linter 안정화 | ⚠️ 부분 | v0.45 |
| **Rust 제거** | Cargo.toml 불필요, BMB-only 빌드 | ⏳ 대기 (Stage 1 완료) | v0.46 |
| **자체 컴파일** | BMB 컴파일러가 자신을 컴파일 | ⏳ CLI 필요 | v0.46 |
| **디버깅 지원** | DWARF 정보, 소스맵 | 📋 계획 | v0.46 |
| **성능 검증** | Gate #3.1 통과 (C 대비 ≤1.10x) | ⚠️ LLVM 필요 | v0.47 |
| **크로스 컴파일** | Linux/Windows/macOS/WASM | ❌ 미완료 | v0.48 |
| **생태계** | 14+ 핵심 패키지 | ⚠️ 12/14 | v0.48 |
| **샘플/문서** | 5개 샘플 앱, 5개 시나리오 | ❌ 미완료 | v0.49 |
| **보안 감사** | 컴파일러/런타임 보안 검토 | ❌ 미완료 | v0.50 |
| **테스트 통과** | 전체 테스트 스위트 (1,753+) | ✅ 완료 | v0.50 |
| **AI Query** | RFC-0001 완전 구현 | ✅ Phase 3 완료 | v0.50 |

---

## Phase v0.45: 기반 완성 (Foundation Completion)

**목표**: 개발자 경험(DX) 완성 - 도구와 라이브러리 안정화

> **의존성**: 이 페이즈가 완료되어야 v0.46 자체 컴파일이 가능

### 태스크

| ID | 태스크 | 설명 | 우선순위 | 상태 |
|----|--------|------|----------|------|
| 45.1 | **stdlib API 확정** | 표준 라이브러리 공개 API 확정 및 문서화 | P0 | ⏳ 제네릭 필요 |
| 45.2 | **stdlib 완성도 검토** | core/, string/, array/, io/ 모듈 기능 검토 | P0 | ✅ 완료 |
| 45.3 | **에러 메시지 개선** | 컴파일 에러 사용자 친화적 포맷팅 | P0 | ⏳ ariadne 기반 양호 |
| 45.4 | **LSP 안정화** | 자동완성, 진단, 정의로 이동, 리팩토링 | P0 | ✅ v0.9.0 성숙 |
| 45.5 | **Formatter 완성** | `bmb fmt` 모든 구문 지원 | P1 | ⏳ 주석 보존 필요 |
| 45.6 | **Linter 추가** | 기본 린트 규칙 및 `bmb lint` 명령 | P1 | ✅ 완료 |
| 45.7 | **REPL 안정화** | 대화형 환경 안정화 및 기능 확장 | P1 | ✅ v0.45 다중 타입 |
| 45.8 | **API 안정성 문서** | stdlib 호환성 보장 문서 작성 | P1 | ✅ 완료 |

### 검증 기준

```bash
# stdlib API 테스트
bmb test stdlib/**/*.bmb

# LSP 기능 테스트
bmb lsp --test

# Formatter 검증
bmb fmt --check stdlib/**/*.bmb
```

### 산출물

- `docs/STDLIB_API.md` - 표준 라이브러리 API 레퍼런스
- 개선된 에러 메시지 템플릿
- LSP 프로토콜 완전 구현

---

## Phase v0.46: 독립성 (Independence)

**목표**: Cargo.toml 없이 BMB만으로 컴파일러 빌드

> **의존성**: v0.45 stdlib 완성 필요 (부트스트랩이 stdlib 사용)

### 태스크

| ID | 태스크 | 설명 | 우선순위 | 상태 |
|----|--------|------|----------|------|
| 46.1 | **LLVM 백엔드 검증** | WSL에서 `bmb build bootstrap/compiler.bmb` 성공 | P0 | ✅ 완료 |
| 46.2 | **Golden Binary 생성** | 첫 번째 네이티브 BMB 컴파일러 바이너리 | P0 | ✅ 완료 |
| 46.3 | **자체 컴파일 검증** | Golden Binary로 자신 재컴파일 (3-Stage) | P0 | ⏳ 대기 (CLI 필요) |
| 46.4 | **Cargo.toml 제거** | Rust 의존성 완전 제거 | P0 | ⏳ 대기 |
| 46.5 | **디버깅 지원** | DWARF 디버그 정보 생성 | P1 | 📋 계획 |
| 46.6 | **소스맵 생성** | 디버거용 소스 위치 매핑 | P1 | 📋 계획 |
| 46.7 | **빌드 문서화** | BMB-only 빌드 가이드 작성 | P1 | ✅ 완료 |

### 완료된 작업 (2026-01-12 ~ 01-13)

1. **PHI 타입 추론 수정** (커밋 `55b5953`)
   - If/Match 표현식의 PHI 결과 타입을 `ctx.locals`에 등록
   - 메서드 호출 반환 타입 추적 (`len`, `slice` 등)
   - 런타임 함수 반환 타입 테이블 완성

2. **문자열 연산 LLVM codegen** (커밋 `d6dae1c`)
   - `bmb_string_from_cstr` 래핑 추가
   - StringBuilder API: `sb_new`, `sb_push`, `sb_build`, `sb_clear`
   - 포인터 산술 연산 지원

3. **CLI 런타임 함수 구현** (커밋 `86ec840`, `330bab7`)
   - `arg_count`, `get_arg`: C 런타임 및 LLVM codegen 완료
   - `read_file`, `write_file`, `file_exists`: File I/O 지원

4. **타입 추론 수정** (커밋 `b171ca0`, `96f1114`)
   - LLVM codegen: `get_arg` → `ptr` 반환 타입
   - MIR lowering: `get_arg` → `MirType::String` 매핑

5. **Bootstrap 검증** (커밋 `3946f8a`)
   - `compiler.bmb` 네이티브 테스트: 777→385→888→8→393→999 ✓
   - `lexer.bmb`, `types.bmb` 네이티브 테스트 통과 ✓
   - `bmb_unified_cli.bmb` 네이티브 컴파일 성공 ✓

### 현재 블로커

- **46.3 블로커**: `bmb-stage1` (bmb_unified_cli 네이티브 바이너리)이 BMB 소스 파싱 시 SIGSEGV
  - 빈 파일: 정상 동작
  - "fn" 키워드 포함: SIGSEGV 발생
  - 동일 코드 인터프리터에서는 정상 작동
  - 추정 원인: 네이티브 컴파일 시 문자열 처리 또는 재귀 로직 문제

### 검증 기준

```bash
# 3-Stage 자체 컴파일 검증
bmb build bootstrap/compiler.bmb -o bmb-stage1   # Stage 1: Rust BMB로 빌드
./bmb-stage1 build bootstrap/compiler.bmb -o bmb-stage2  # Stage 2: BMB로 빌드
./bmb-stage2 build bootstrap/compiler.bmb -o bmb-stage3  # Stage 3: 동일성 검증
diff bmb-stage2 bmb-stage3  # 동일해야 함

# 디버깅 검증
gdb ./bmb-stage2 -ex "info functions"  # DWARF 정보 확인
```

### 산출물

- `bmb-golden` - 첫 번째 네이티브 컴파일러 바이너리
- `docs/BUILD_FROM_SOURCE.md` - BMB-only 빌드 가이드

---

## Phase v0.47: 성능 검증 (Performance)

**목표**: 모든 벤치마크 Gate 통과

> **의존성**: v0.46 네이티브 컴파일 필요 (인터프리터는 2-4x 느림)

### 태스크

| ID | 태스크 | 설명 | 우선순위 |
|----|--------|------|----------|
| 47.1 | **Gate #3.1 검증** | Compute ≤1.10x C, Contract ≤0.90x C | P0 |
| 47.2 | **Gate #3.2 검증** | Benchmarks Game ≤1.05x C | P1 |
| 47.3 | **Gate #3.3 검증** | 3+ 벤치마크 C보다 빠름 | P1 |
| 47.4 | **Gate #4.1 유지** | 자체 컴파일 <60s (현재 0.56s) | P0 |
| 47.5 | **성능 회귀 방지** | CI에서 2% 임계값 적용 | P0 |
| 47.6 | **최적화 패스 완성** | MIR 최적화 패스 검토 및 개선 | P1 |
| 47.7 | **`bmb q proof`** | 검증 결과 인덱스 (`proofs.idx`) | P2 |
| 47.8 | **증명 상태 쿼리** | `--unverified`, `--timeout`, `--failed` 필터 | P2 |

### 벤치마크 현황

| 카테고리 | 벤치마크 수 | 목표 |
|----------|------------|------|
| Compute | 10 | ≤1.10x C |
| Contract | 5 | <0.90x C (계약 최적화) |
| Memory | 5 | ≤1.10x Rust |
| Compilation | 3 | <60s |

### 검증 기준

```bash
# WSL Ubuntu에서 실행 필수
cd ecosystem/benchmark-bmb
./runner/target/release/benchmark-bmb gate all -v
```

---

## Phase v0.48: 생태계 (Ecosystem)

**목표**: 핵심 패키지 및 크로스 컴파일 지원

> **의존성**: v0.47 성능 검증 완료 필요 (패키지가 성능 기준 충족해야 함)

### 현재 gotgan-packages 상태 (12개, v0.32 호환 완료)

| 패키지 | 설명 | LOC | 상태 |
|--------|------|-----|------|
| `bmb-fmt` | 문자열 포매팅 | 111 | ✅ v0.32 |
| `bmb-fs` | 파일시스템 | 100 | ✅ v0.32 |
| `bmb-http` | HTTP 유틸리티 | 120 | ✅ v0.32 |
| `bmb-json` | JSON 파싱 | 479 | ✅ v0.32 |
| `bmb-log` | 로깅 | 109 | ✅ v0.32 |
| `bmb-math` | 수학 함수 | 154 | ✅ v0.32 |
| `bmb-rand` | 난수 생성 | 60 | ✅ v0.32 |
| `bmb-regex` | 정규표현식 | 92 | ✅ v0.32 |
| `bmb-semver` | 시맨틱 버저닝 | 203 | ✅ v0.32 |
| `bmb-testing` | 테스팅 프레임워크 | 118 | ✅ v0.32 |
| `bmb-time` | 시간 유틸리티 | 168 | ✅ v0.32 |
| `bmb-toml` | TOML 파싱 | 279 | ✅ v0.32 |

### 태스크

| ID | 태스크 | 설명 | 우선순위 |
|----|--------|------|----------|
| 48.1 | **collections 패키지** | HashMap, BTreeMap, VecDeque | P0 |
| 48.2 | **args 패키지** | CLI 인자 파싱 (clap 포팅) | P1 |
| 48.3 | **크로스 컴파일 Linux** | `--target x86_64-linux` | P0 |
| 48.4 | **크로스 컴파일 Windows** | `--target x86_64-windows` | P0 |
| 48.5 | **크로스 컴파일 macOS** | `--target x86_64-macos`, `aarch64-macos` | P1 |
| 48.6 | **WASM 백엔드 안정화** | `--target wasm32` | P1 |
| 48.7 | **gotgan 레지스트리** | 패키지 검색 및 다운로드 서버 | P2 |
| 48.8 | **`bmb q deps`** | 의존성 쿼리 (`--reverse`, `--transitive`) | P1 |
| 48.9 | **`bmb q contract`** | 계약 상세 쿼리 (`--uses-old`) | P1 |

### Rust 포팅 워크플로우

```bash
# 1. Rust 크레이트 소스 가져오기
# 2. tools/rust_to_bmb.mjs로 변환
node tools/rust_to_bmb.mjs path/to/*.rs --apply

# 3. BMB 계약 추가
# 4. 테스트 작성
# 5. gotgan-packages/에 등록
```

---

## Phase v0.49: 시연 (Showcase)

**목표**: 실제 사용 사례 시연 및 문서 완성

> **의존성**: v0.48 생태계 패키지 완성 필요 (샘플 앱이 패키지 사용)

### 샘플 애플리케이션 (5개)

| 샘플 | 설명 | LOC | 우선순위 | 사용 패키지 |
|------|------|-----|----------|------------|
| `bmb-grep` | grep 클론 | 500 | P0 | regex, fs, args |
| `bmb-calc` | 계산기 (REPL) | 300 | P0 | math |
| `bmb-json-tool` | JSON 처리 CLI | 400 | P1 | json, fs, args |
| `bmb-httpd` | 간단한 HTTP 서버 | 800 | P1 | http, log |
| `bmb-compiler` | 미니 언어 컴파일러 | 1000 | P2 | - |

### 시나리오 문서 (5개)

| 시나리오 | 설명 | 파일 |
|----------|------|------|
| 시스템 프로그래밍 | 메모리 안전성과 계약 | `docs/scenarios/SYSTEMS.md` |
| 계약 기반 검증 | 정적 검증으로 버그 제거 | `docs/scenarios/CONTRACTS.md` |
| 성능 최적화 | C 수준 성능 달성 방법 | `docs/scenarios/PERFORMANCE.md` |
| Rust에서 마이그레이션 | Rust 개발자를 위한 가이드 | `docs/scenarios/FROM_RUST.md` |
| AI 코드 생성 | LLM과 BMB의 시너지 | `docs/scenarios/AI_NATIVE.md` |

### 태스크

| ID | 태스크 | 설명 | 우선순위 |
|----|--------|------|----------|
| 49.1 | **샘플 앱 5개** | 위 목록 구현 | P0 |
| 49.2 | **시나리오 문서 5개** | 위 목록 작성 | P0 |
| 49.3 | **튜토리얼 완성** | Getting Started, By Example 확장 | P1 |
| 49.4 | **마이그레이션 도구 완성** | pre-v0.32 → v0.32 완전 지원 | P1 |
| 49.5 | **`bmb q ctx`** | AI 컨텍스트 생성 | P1 |
| 49.6 | **`bmb q sig`** | 시그니처 검색 (`--accepts`, `--returns`) | P1 |
| 49.7 | **`--format llm`** | LLM 최적화 출력 형식 | P1 |

---

## Phase v0.50: 최종 검증 (Final Verification)

**목표**: v1.0.0-beta 릴리스 준비 완료

> **의존성**: 모든 이전 페이즈 완료 필수

### 보안 감사 항목

| 항목 | 설명 | 심각도 |
|------|------|--------|
| 컴파일러 입력 검증 | 악의적 소스 코드 처리 | High |
| LLVM IR 생성 안전성 | 버퍼 오버플로우, 메모리 누수 | High |
| 계약 검증 우회 방지 | @trust 남용 탐지 | Medium |
| 패키지 보안 | 의존성 무결성 검증 | Medium |
| WASM 샌드박싱 | 웹 환경 격리 | Medium |

### 최종 체크리스트

| 항목 | 검증 방법 | 담당 |
|------|----------|------|
| 전체 테스트 통과 | `cargo test && bmb test bootstrap/*.bmb` | CI |
| 벤치마크 Gate 통과 | `benchmark-bmb gate all` | CI |
| 자체 컴파일 성공 | Stage 3 동일성 검증 | CI |
| 문서 완성도 | 모든 공개 API 문서화 | 수동 |
| 패키지 검증 | 14개 핵심 패키지 테스트 | CI |
| 샘플 실행 | 5개 샘플 앱 빌드/실행 | CI |
| 크로스 플랫폼 | Linux, Windows, macOS | CI |
| 보안 감사 | 체크리스트 통과 | 수동 |
| AI Query 완료 | RFC-0001 전체 기능 구현 | 수동 |

### 태스크

| ID | 태스크 | 설명 | 우선순위 |
|----|--------|------|----------|
| 50.1 | **보안 감사 실행** | 위 체크리스트 통과 | P0 |
| 50.2 | **전체 테스트 검증** | 모든 테스트 스위트 통과 | P0 |
| 50.3 | **크로스 플랫폼 검증** | 3개 OS 빌드 및 실행 | P0 |
| 50.4 | **릴리스 노트 작성** | CHANGELOG, 마이그레이션 가이드 | P0 |
| 50.5 | **`bmb q batch`** | 배치 쿼리 (queries.json) | P1 |
| 50.6 | **`bmb q impact`** | 변경 영향 분석 | P1 |
| 50.7 | **`bmb q serve`** | HTTP 쿼리 서버 모드 | P2 |
| 50.8 | **`bmb index --watch`** | 실시간 인덱스 갱신 | P2 |
| 50.9 | **RFC-0001 문서 갱신** | "Draft" → "Implemented" | P1 |

---

## 버전 타임라인

```
현재 (v0.44) ──────────────────────────────────────────────────
         │
         ▼
v0.45 기반 완성 (Foundation Completion) ────────────────────────
         │ - stdlib API 확정 및 문서화
         │ - 에러 메시지 개선 (DX)
         │ - LSP/Formatter/Linter 안정화
         │ - REPL 안정화
         ▼
v0.46 독립성 (Independence) ────────────────────────────────────
         │ - LLVM 백엔드 검증
         │ - Golden Binary 생성
         │ - 자체 컴파일 검증 (3-Stage)
         │ - 디버깅 지원 (DWARF)
         ▼
v0.47 성능 검증 (Performance) ──────────────────────────────────
         │ - Gate #3.1, #3.2, #3.3 통과
         │ - 성능 회귀 방지 CI
         │ - 최적화 패스 완성
         ▼
v0.48 생태계 (Ecosystem) ───────────────────────────────────────
         │ - 14개 핵심 패키지 완성
         │ - 크로스 컴파일 (Linux/Windows/macOS/WASM)
         │ - gotgan 레지스트리
         ▼
v0.49 시연 (Showcase) ──────────────────────────────────────────
         │ - 5개 샘플 애플리케이션
         │ - 5개 시나리오 문서
         │ - 튜토리얼/마이그레이션 완성
         ▼
v0.50 최종 검증 (Final Verification) ───────────────────────────
         │ - 보안 감사
         │ - 전체 체크리스트 통과
         │ - 릴리스 준비
         ▼
v1.0.0-beta Golden ★ ────────────────────────────────────────
         완전한 프로그래밍 언어 + AI-Native Query System
```

---

## 의존성 그래프

```
v0.45 기반 완성
  │
  ├── stdlib 완성 ────────────────────┐
  │                                   │
  ├── LSP/도구 안정화                  │
  │                                   │
  └── 에러 메시지 개선                  │
                                      ▼
v0.46 독립성 ◄───────────────────── (stdlib 의존)
  │
  ├── 네이티브 컴파일 ────────────────┐
  │                                   │
  └── 디버깅 지원                      │
                                      ▼
v0.47 성능 검증 ◄───────────────── (네이티브 필요)
  │
  └── 벤치마크 통과 ──────────────────┐
                                      │
                                      ▼
v0.48 생태계 ◄─────────────────── (성능 기준 충족)
  │
  └── 패키지 완성 ────────────────────┐
                                      │
                                      ▼
v0.49 시연 ◄───────────────────── (패키지 사용)
  │
  └── 샘플/문서 완성 ─────────────────┐
                                      │
                                      ▼
v0.50 최종 검증 ◄──────────────── (전체 완성)
  │
  └── 보안 감사 + 체크리스트
                                      │
                                      ▼
                            v1.0.0-beta Golden ★
```

---

## 완료된 작업 요약

### 언어 기능 (v0.1-v0.34)
- ✅ 타입 시스템 (제네릭, 열거형, 구조체)
- ✅ 계약 시스템 (pre, post, invariant, where)
- ✅ 제어 흐름 (if-else, match, while, for)
- ✅ 연산자 (산술, 비교, 논리, 비트, 시프트)
- ✅ f64 부동소수점
- ✅ 동적 컬렉션 (Vec, Box)

### 컴파일러 (v0.1-v0.37)
- ✅ Lexer (logos)
- ✅ Parser (lalrpop)
- ✅ 타입 추론 (Hindley-Milner)
- ✅ MIR 생성
- ✅ LLVM 백엔드
- ✅ WASM 백엔드 (실험적)
- ✅ SMT 검증 (Z3)

### 부트스트랩 (v0.30-v0.38)
- ✅ 30K LOC BMB 자체 호스팅 컴파일러
- ✅ Stage 3 테스트 100% 통과
- ✅ 1,580 부트스트랩 테스트
- ✅ v0.32 문법 마이그레이션 완료

### 도구 (v0.7-v0.9)
- ✅ gotgan 패키지 매니저
- ✅ VS Code 확장
- ✅ Tree-sitter 문법
- ✅ 플레이그라운드

### 인프라 (v0.40-v0.44)
- ✅ CI/CD (GitHub Actions)
- ✅ 멀티플랫폼 빌드
- ✅ 성능 회귀 탐지
- ✅ API 안정성 문서
- ✅ 릴리스 자동화

### AI Query System (v0.25-v0.49 - RFC-0001 Phase 1-3)
- ✅ `bmb index` - 인덱스 생성 (`.bmb/index/`)
- ✅ `bmb q sym` - 심볼 검색
- ✅ `bmb q fn` - 함수 조회 (`--has-pre`, `--has-post`, `--recursive`)
- ✅ `bmb q type` - 타입 조회
- ✅ `bmb q metrics` - 프로젝트 통계
- ⏳ v0.48: `bmb q deps`, `bmb q contract`
- ⏳ v0.49: `bmb q ctx`, `bmb q sig`, `--format llm`
- ⏳ v0.50: `bmb q batch`, `bmb q impact`, `bmb q serve`

---

## 문서 구조 (정리 후)

```
docs/
├── SPECIFICATION.md      # 언어 스펙 (v0.32)
├── LANGUAGE_REFERENCE.md # 언어 레퍼런스
├── ARCHITECTURE.md       # 컴파일러 아키텍처
├── ROADMAP.md           # 이 문서
├── API_STABILITY.md     # API 안정성 보장
├── STDLIB_API.md        # 표준 라이브러리 API (v0.45 예정)
├── BUILD_FROM_SOURCE.md # BMB-only 빌드 (v0.46 예정)
├── ECOSYSTEM.md         # 생태계 개요
├── GOTGAN.md            # 패키지 매니저
├── scenarios/           # 시나리오 문서 (v0.49 예정)
│   ├── SYSTEMS.md
│   ├── CONTRACTS.md
│   ├── PERFORMANCE.md
│   ├── FROM_RUST.md
│   └── AI_NATIVE.md
├── tutorials/           # 튜토리얼
│   ├── GETTING_STARTED.md
│   ├── CONTRACT_PROGRAMMING.md
│   ├── FROM_RUST.md
│   └── BY_EXAMPLE.md
└── archive/             # 아카이브 (과거 문서)
    ├── RELEASE_v0.29.md
    ├── GAP_ANALYSIS.md
    └── ...
```

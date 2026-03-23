# BMB Development Roadmap
Updated: 2026-03-21

---

## 진척도 게이지

```
Bootstrap   ██████████████████░░ 98%   Fixed Point ✅, i8*=0, inttoptr=5638→2901(opt)
Self-Host   ██████████████████░░ 99%   CLI 41개, LSP 9개, Test Runner ✅
Benchmark   ████████████████████ 100%  309 빌드 ✅, 16+ FASTER, 0 FAIL, BMB > C+Rust
Ecosystem   ██████████████░░░░░░ 70%   gotgan dep E2E ✅, stdlib 15/15, docs 4 tutorials
```

### 핵심 수치

| 항목 | 현재 |
|------|------|
| **Bootstrap** | 3-Stage Fixed Point (S2 == S3), compiler.bmb 19,814 LOC, 전체 bootstrap 55,654 LOC |
| **Rust 컴파일러** | 132,537 LOC (동결, 회귀 방지만) |
| **골든 테스트** | 701 BMB + 6,186 Rust regression = 전체 통과 |
| **벤치마크** | 309 빌드 ✅, 16+ FASTER (knapsack 0.15x, lcs 0.53x), 0 FAIL |
| **3-Way 검증** | BMB beats C AND Rust: knapsack, lcs, floyd, spectral, n_body |
| **EXISTENTIAL** | 7/7 완료 — 계약→성능 파이프라인 증명됨 |
| **에코시스템** | 14 서브프로젝트, 102+ gotgan 패키지, stdlib 15/15 (time, fs 신규) |
| **진단 출력** | JSON에 line:col 포함 (v0.97) |
| **gotgan E2E** | 3-tier dep chain (pkg-top→pkg-mid→pkg-base) + circular detection ✅ |
| **문서** | 4 튜토리얼 (Ownership, Concurrency, Modules, ErrorHandling), API 14 모듈 |
| **@export + SharedLib** | `@export` 어트리뷰트 + `--shared` (.dll/.so) 빌드 ✅ |
| **bmb-algo** | 41 algorithms — Python ctypes 바인딩 E2E ✅ (Cycle 2051) |
| **bmb-compute** | 25 functions (Math, Statistics, Random, Vector, Utility) — Python E2E ✅ (Cycle 2073) |
| **bmb-crypto** | 11 functions (SHA256, MD5, CRC32, HMAC, Base64/32, Adler32, Fletcher16) — Python E2E ✅ (Cycle 1993) |
| **bmb-text** | 20 functions (KMP, find, replace, case, trim, repeat, palindrome) — Python E2E ✅ (Cycle 2033) |
| **bmb-json** | 8 functions (validate, stringify, get, array) — Python E2E ✅ (Cycle 1985) |
| **Bootstrap** | @export 완전 구현: parser + lowering + codegen (Cycles 2025-2065) |
| **벤치마크** | knapsack 90.7x, nqueens 181.6x, prime_count 25.6x vs Pure Python (Cycle 2005) |
| **컴파일러 수정** | @export i32 sext + non-param filter + loop metadata fix (Cycles 1964-1989) |
| **문서** | README.md × 5 libraries (Cycle 2045) |
| **Stage 1 부트스트랩** | 검증 완료: 골든 테스트 6/6 통과 (Cycle 2085) |
| **Edge Case 테스트** | 81 edge case tests (경계 조건, 빈 입력, 대규모 값) ✅ (Cycle 2091) |
| **최종 검증** | Cycle 2104: cargo 6,186 ✅, 5 libs × 115 tests ✅, 81 edge cases ✅, 105 @export |
| **패키징 인프라** | Cycles 2105-2124: pyproject.toml + pytest(957) + benchmarks + .pyi + build_all.py + CI + E2E ✅ |

---

## 현재 단계 — 남은 작업

### Phase C: Bootstrap 코드젠 품질 (v0.98) — **다음 주요 작업**

Bootstrap IR의 근본적 한계 해소. inttoptr 7,947개를 native ptr로 전환.

| 작업 | 상세 | 위험도 | 예상 규모 |
|------|------|--------|----------|
| **C-1. Native Ptr 타입 시스템** | i64 타입 소거 → ptr 타입 + nonnull/noalias | EXTREME | lowering + llvm_ir + compiler (~6-8주) |
| **C-2. inttoptr 점진적 감소** | malloc/realloc → native ptr, 함수 파라미터 → ptr 추론 | HIGH | 7,947 → <1,000 목표 |

> **위험**: Fixed Point 깨질 가능성 높음. 3-Stage 검증 반복 필수.

---

### Phase D: Playground WASM

| 작업 | 상태 |
|------|------|
| WASM 빌드 셋업 | 미착수 |
| wasm-bindgen 인터페이스 | 미착수 |
| 프론트엔드 통합 + 배포 | 미착수 |

---

### Phase E: 에코시스템 성숙 — **진행 중**

| 작업 | 상태 |
|------|------|
| ~~gotgan 의존성 해석~~ | ✅ 완료 |
| ~~tree-sitter-bmb v0.3.0~~ | ✅ 완료 |
| ~~stdlib math + collections~~ | ✅ 완료 |
| ~~VS Code LSP 연결~~ | ✅ 완료 |
| ~~stdlib 추가: time, fs~~ | ✅ 완료 (Cycle 1929, 1932) |
| stdlib 추가: net | 미착수 |
| ~~gotgan: 패키지 간 의존성 E2E 테스트~~ | ✅ 완료 (Cycle 1939-1940) |
| 디버거 지원 (DWARF 정보 생성) | 미착수 |
| Playground WASM 빌드 + 배포 | 미착수 (Phase D) |
| lang-bmb-site 문서화 완성 | 미착수 |

---

### P1: 컴파일러 품질

| 작업 | 상태 |
|------|------|
| ~~MIR 옵티마이저 BMB 이식 15/15~~ | ✅ 완료 |
| ~~에러 진단: JSON에 line:col 추가~~ | ✅ 완료 (Cycle 1934) |
| 런타임 스택 트레이스 | 미착수 |
| 벤치마크 CI 자동화 (2% 임계값) | 미착수 |
| Rust --release 비교 추가 | 미착수 |

---

### P3: 공개 준비

| 작업 | 상태 |
|------|------|
| 크로스 플랫폼 (Linux/macOS/ARM64) | 미착수 |
| 커뮤니티 빌딩 (HN/Reddit) | 미착수 |
| AI-Native 실증 (60문제 × 5언어 × 3 LLM) | 미착수 |
| 언어 스펙 최종판 | 미착수 |
| Go/No-Go 게이트 | 미착수 |

---

## 추천 다음 단계 — Dogfooding 기반 (우선순위순)

> 바인딩 라이브러리 개발 = BMB 언어 한계 발견 + 해결 = 언어 완성

```
★★★ IMMEDIATE: FFI 안전성 (bmb_panic → 에러코드, String FFI, TLS)
     ↓ BMB 언어 개선: 런타임 에러 핸들링 아키텍처
★★  SPRINT 2: bmb-algo 프로덕션화 (PyPI, 벤치마크, 크로스플랫폼)
     ↓ BMB 증명: pip install → "C보다 6.8x 빠르다"
★   SPRINT 3-6: bmb-crypto, bmb-json, bmb-text
     ↓ BMB dogfooding: 실전 사용에서 한계 발견 + 해결
    SPRINT 7+: 제네릭, Native Ptr, 커뮤니티
     ↓ BMB v1.0: 완성된 언어
```

**상세 로드맵**: `docs/BINDING_ROADMAP.md`

---

## Phase 로드맵 타임라인

```
═══════════════════ 완료 (v0.1 ~ v0.96, Cycles 1-1955) ════════════════

v0.1-v0.93   Foundation → Concurrency → LLVM Opt → Self-Hosting
             ★ Rust 졸업 — Rust 컴파일러 동결
v0.94        G-1: 3-Stage Fixed Point
v0.95        G-2+G-3: 셀프호스팅 + 벤치마크 Clang 동등
v0.96.1-25   EXISTENTIAL 7/7: 계약→성능 파이프라인 증명
v0.96.26-46  MIR 옵티마이저, 벤치마크 70→309, 성능 최적화
v0.96.47+    LSP, stdlib 15/15, gotgan E2E, 4 tutorials, 14 API docs
             @export + --shared + bmb-algo Python E2E (8 algorithms)

═══════════════════ 현재 위치: Dogfooding 시작 ═════════════════════════

v0.97        ★★★ FFI 안전성 (exit(1) → 에러코드, String FFI, TLS)
             ★★  bmb-algo PyPI (크로스플랫폼, 벤치마크 CI)
          ▼
v0.98        bmb-crypto (SHA256, Base64, HMAC)
             bmb-json (Node.js WASM)
             Native Ptr 타입 시스템
          ▼
v0.99        제네릭 타입 시스템 (Vec<T>, HashMap<K,V>)
             크로스 플랫폼 CI + 공개 준비
          ▼
v1.0         Release + HN/Reddit + 커뮤니티
```
```

---

## 구조적 한계 (변경 불가)

| 항목 | 이유 |
|------|------|
| Z3 verify 자체호스팅 | 외부 SMT 솔버 의존 — IPC로만 연동 가능 |
| Rust 완전 퇴역 | CLI/에러의 clap/ariadne 대체는 이미 bootstrap에 구현, Rust는 회귀 방지 목적으로만 유지 |
| LLVM 한계 벤치마크 | insertion_sort/running_median/max_consecutive_ones — IR 동등, ISel 휴리스틱 차이 |

---

## 완료 요약

<details>
<summary>전체 완료 항목 (클릭하여 펼치기)</summary>

### EXISTENTIAL 7/7 — 계약→성능 파이프라인 ✅

계약이 코드를 더 빠르게 만든다는 것을 코드와 측정으로 증명함.
- `llvm.assume` 생성: pre/post 모든 조건 ✅
- Bounds check elimination: 43% 성능 차이, BMB가 C를 24% 추월 ✅
- Divzero check elimination: 32% 성능 차이, BMB가 C를 ~2% 추월 ✅
- 8개 벤치마크 실동작 + 3-way IR 분석 ✅
- `--safe` 모드 동작 (safe+contract ≈ C > safe-only) ✅

### Bootstrap 완성 ✅

- 3-Stage Fixed Point (S2 == S3)
- compiler.bmb 19,814 LOC + 전체 55,654 LOC
- i8*/i64* → ptr 완전 마이그레이션
- noundef 1,452개, nonnull 40개 적용
- MIR 옵티마이저 15/15 이식

### 벤치마크 ✅

- 70개 벤치마크: 14+ FASTER, 48+ PASS, 3 LLVM-OK
- bench.sh v5: --stats (95% CI, Mann-Whitney U), --dir, IQR outlier removal
- classify_faster.sh: METADATA/PIPELINE/MIXED 3-way 분류
- compare.py: JSON 스키마 호환, 자동 비교

### LLVM 속성 전파 ✅

nonnull, noalias, nocapture, readonly, noundef, dereferenceable, align, nosync,
range, private, alwaysinline, nofree, tail, norecurse, memory(none/read), speculatable

### 셀프호스팅 ✅

- CLI 41개 커맨드 (BMB로 구현)
- LSP 서버 9기능 (diagnostics/hover/completion/definition/documentSymbol/references/rename/formatting)
- Test Runner ✅
- REPL, fmt, lint ✅

### 에코시스템 (진행 중)

- VS Code LSP 연결 (bmb.lspServerPath) ✅
- stdlib 12/12 모듈 (core, string, array, io, json, math, collections, parse, process, test, action-bmb 등) ✅
- tree-sitter-bmb v0.3.0 (16개 신규 기능) ✅
- gotgan resolver (의존성 해석, 토폴로지 빌드 순서) ✅
- 102+ gotgan 패키지 ✅

### 성능 최적화 스프린트 (Cycles 1809-1884)

- TBAA metadata + inline load/store
- GEP inbounds nuw + nonnull
- ThinLTO 13-17% 오버헤드 해소
- Cold-then branch weights + loop vectorization fix
- MIR pattern expansion + Store-Load Forwarding
- Codegen NEAR-OPTIMAL 확인 (IR ≈ C)

</details>

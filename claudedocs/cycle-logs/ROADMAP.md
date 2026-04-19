# BMB Development Roadmap
Updated: 2026-04-19 (post-Cycles 2301-2309: Tasks B-11 Phase 1 ✅, B-12 ✅, B-13 ✅)

---

## 진척도 게이지

```
Bootstrap   ██████████████████░░ 98%   Fixed Point ✅ (S2==S3, 65s, Cycle 2296), compiler.bmb 19,818 LOC
Self-Host   ████████████████████ 99%   CLI 41개 (+ bmb bench), LSP 9개, Test Runner, REPL
Benchmark   ████████████████████ 100%  309 빌드 ✅, 16+ FASTER, 0 FAIL, BMB > C+Rust
Ecosystem   ████████████████░░░░ 82%   5 libs 140 @export, C headers, WASM, pytest 1,017+
SIMD        ████████████████████ 99%   1급 타입 ✅ (f64xN, **f32xN**, i32xN, i64xN, maskN)
                                       text/inkwell codegen Rule 7 parity ✅
                                       stdlib/simd **183 fns** (147 + **36 shuffle** variants)
                                       `@include "stdlib/simd"` 자동 로딩 ✅ (build + check)
                                       런타임 correctness: f64 10 + mask 5 + f32 12 + **shuffle 13** = 40 checks ✅
                                       SAXPY/matvec/stencil 벤치 + SAXPY f32 ✅
                                       mask{2,4,8,16} + cmp/blend/any/all ✅
                                       f32 ↔ f64 fpext/fptrunc + full int↔f32 casts ✅
                                       **shuffle Phase 1 ✅** — reverse/broadcast_lane/slide_left/slide_right (single-vector)
                                       **scalar store/load 32-bit ✅** — store_i32/load_i32/store_f32/load_f32 (B-12/B-13)
                                       Pending: 2-source shuffle (B-11.5, stencil 회복용)
```

### 핵심 수치

| 항목 | 현재 |
|------|------|
| **Bootstrap** | 3-Stage Fixed Point (S2 == S3), compiler.bmb 19,818 LOC, 전체 59,046 LOC |
| **Rust 컴파일러** | 132,537 LOC (동결, 회귀 방지만) |
| **골든 테스트** | 3,660 BMB + 6,186 Rust regression = 전체 통과 |
| **벤치마크** | 309 빌드 ✅, 16+ FASTER (knapsack 0.15x, lcs 0.53x), 0 FAIL |
| **3-Way 검증** | BMB beats C AND Rust: knapsack, lcs, floyd, spectral, n_body |
| **EXISTENTIAL** | 7/7 완료 — 계약→성능 파이프라인 증명됨 |
| **바인딩 에코시스템** | 5 libs, 140 @export, 1,017 pytest, C headers ×5, WASM ×5 |

### 바인딩 라이브러리 상세

| 라이브러리 | @export | 버전 | 하이라이트 |
|-----------|---------|------|-----------|
| **bmb-algo** | 55 | v0.3.0 | knapsack 90.7x, nqueens 181.6x vs Python |
| **bmb-compute** | 33 | v0.2.0 | math, statistics, random, vector |
| **bmb-text** | 24 | v0.2.0 | KMP, find, replace, case, trim |
| **bmb-crypto** | 15 | v0.3.0 | SHA-256, MD5, CRC32, HMAC, Base64/32 |
| **bmb-json** | 13 | v0.2.0 | validate, stringify, get, array |

---

## 현재 단계 — 남은 작업

> Dogfooding 완료 (Cycle 2185). 바인딩 에코시스템 안정화됨.
> stdlib/simd 완성 (Cycle 2265). 남은 작업은 **인프라/플랫폼 수준** 중심.

### Phase B-cont: SIMD 마무리 (v0.97.x)

| 작업 ID | 작업 | 상태 | 세션 |
|---------|------|------|------|
| **B-4** | Inkwell BinOp Vector emission (Rule 7 parity) | ✅ 완료 | Cycles 2266-2272 |
| **B-7** | stdlib 모듈 auto-import (build + check) | ✅ 완료 | Cycles 2275-2276, 2284 |
| **B-9** | SIMD 실증 워크로드 (SAXPY + matvec + stencil) | ✅ 완료 | Cycles 2273-2274, 2288 |
| **B-8** | Comparison + mask 타입 (cmp/blend/any/all) | ✅ 완료 | Cycles 2283-2287 |
| **B-10** | SIMD perf user guide (`SIMD_PERF_NOTES.md`) | ✅ 완료 | Cycle 2289 |
| **B-11** | Shuffle/permute Phase 1 (`reverse`/`broadcast_lane`/`slide_{left,right}`, single-vector) | ✅ 완료 | Cycles 2301-2305 (36 fns, 13 runtime checks, Rule 7 parity) |
| **B-11.5** | 2-source shuffle (`shuffle_VxN(a, b, mask)`) — stencil 회복용 | 대기 | Phase 1 후속. cross-block slide 필요. 3-5 cycles 예상 |
| **B-12** | `store_i32`/`load_i32` 런타임 헬퍼 + i32 SIMD 벤치 | ✅ 완료 | Cycles 2307-2308 (text backend dispatch, inkwell 기존) |
| **A-1** | f32 primitive + f32x{4,8,16} | ✅ 완료 | Cycles 2291-2297 (8 cycles, under 20-budget) |
| **B-13** | `store_f32`/`load_f32` 스칼라 runtime 헬퍼 | ✅ 완료 | Cycles 2307-2308 (양 백엔드 dispatch) |

### Phase C: Bootstrap 코드젠 품질 (v0.98)

Bootstrap IR의 근본적 한계 해소. inttoptr을 native ptr로 전환.

| 작업 | 상세 | 위험도 |
|------|------|--------|
| **C-1. Native Ptr 타입 시스템** | i64 타입 소거 → ptr 타입 + nonnull/noalias | EXTREME |
| **C-2. inttoptr 점진적 감소** | malloc/realloc → native ptr, 함수 파라미터 → ptr 추론 | HIGH |

> **위험**: Fixed Point 깨질 가능성 높음. 3-Stage 검증 반복 필수.

---

### Phase D: Playground WASM

| 작업 | 상태 |
|------|------|
| WASM 빌드 셋업 | 미착수 |
| wasm-bindgen 인터페이스 | 미착수 |
| 프론트엔드 통합 + 배포 | 미착수 |

---

### Phase E: 에코시스템 성숙

| 작업 | 상태 |
|------|------|
| ~~gotgan 의존성 해석~~ | ✅ |
| ~~tree-sitter-bmb v0.3.0~~ | ✅ |
| ~~stdlib 15/15~~ | ✅ |
| ~~VS Code LSP 연결~~ | ✅ |
| ~~stdlib time, fs~~ | ✅ |
| ~~gotgan E2E 의존성 테스트~~ | ✅ |
| ~~@export + --shared 빌드~~ | ✅ |
| ~~바인딩 5개 라이브러리~~ | ✅ 140 @export, 1,017 tests |
| ~~C 헤더 생성~~ | ✅ 5개 .h |
| ~~WASM 빌드~~ | ✅ 5개 (62-289 KB) |
| ~~패키징 인프라~~ | ✅ pyproject.toml, .pyi, CI |
| stdlib net 모듈 | 미착수 |
| 디버거 지원 (DWARF) | 미착수 |
| Playground WASM 배포 | 미착수 (Phase D) |
| lang-bmb-site 문서화 | 미착수 |
| PyPI wheel 빌드 + 배포 | 패키징 완료, 배포 미완 |
| 크로스 플랫폼 빌드 (Linux/macOS) | Windows만 검증 |
| Node.js WASM 바인딩 | 미착수 |

---

### P1: 컴파일러 품질

| 작업 | 상태 |
|------|------|
| ~~MIR 옵티마이저 BMB 이식 15/15~~ | ✅ |
| ~~에러 진단: JSON line:col~~ | ✅ |
| 런타임 스택 트레이스 | 미착수 |
| 벤치마크 CI 자동화 (2% 임계값) | 미착수 |

---

### P3: 공개 준비

| 작업 | 상태 |
|------|------|
| 크로스 플랫폼 (Linux/macOS/ARM64) | 미착수 |
| 커뮤니티 빌딩 (HN/Reddit) | 미착수 |
| AI-Native 실증 Phase 2 인프라 | ✅ 30문제, 34패턴, 388테스트 |
| AI-Native 실증 실험 실행 | 미착수 (LLM API 필요) |
| 언어 스펙 최종판 | 미착수 |
| Go/No-Go 게이트 | 미착수 |

---

## 다음 단계 우선순위 (2026-04-19 업데이트)

> SIMD B-4/B-7/B-8/B-9/B-10 완료. **A-1 완료** (Cycles 2291-2297) — f32 1급 타입 + f32x{4,8,16} + cast + stdlib/simd 147 fns. SAXPY/matvec/stencil 실증 결과: 선형 접근에서 LLVM auto-vec이 manual SIMD를 동률/추월. SIMD 진정한 승리는 데이터 의존 분기(B-8 mask)·shuffle(B-11 미구현)·forced FMA에서만 발생함을 `SIMD_PERF_NOTES.md`로 정리.

```
0. ★★★ Task B-11: shuffle/permute 인트린식 (6-8 cycles)
   stencil 같은 auto-vec 패배 영역 회복. 단일 인트린식 패밀리.
   → 매뉴얼 SIMD가 의미 있는 워크로드 폭 확장

1. ★★★ 배포 + 크로스플랫폼
   PyPI wheel 빌드 → Linux/macOS 빌드 → pip install bmb-algo 가능
   → "pip install → C보다 90x 빠르다" 증명

2. ★★  컴파일러 품질 (v0.98)
   Native Ptr 타입 시스템 → inttoptr 제거 → IR 품질 향상
   → Bootstrap 코드젠이 handwritten IR 수준에 도달

3. ★   Task B-12/B-13: store_{i32,f32}/load_{i32,f32} 헬퍼 (2-4 cycles, low-hanging)
   현재 SIMD 워크로드 varied-init에 스칼라 store 부재. 병목 없지만 ergonomic 보강.
   → f32 SAXPY 등에서 non-uniform 초기화 가능

4. ★   공개 준비 (v0.99 → v1.0)
   언어 스펙 최종판 → AI-Native 실증 → HN/Reddit
   → 커뮤니티 형성
```

---

## Phase 로드맵 타임라인

```
════════════════════ 완료 (v0.1 ~ v0.97, Cycles 1-2185) ═══════════════

v0.1-v0.93   Foundation → Concurrency → LLVM Opt → Self-Hosting
             ★ Rust 졸업 — Rust 컴파일러 동결
v0.94        G-1: 3-Stage Fixed Point
v0.95        G-2+G-3: 셀프호스팅 + 벤치마크 Clang 동등
v0.96        EXISTENTIAL 7/7 + MIR 옵티마이저 + 벤치마크 309
             LSP, stdlib 15/15, gotgan E2E, tree-sitter, 4 tutorials
v0.97        @export + --shared + FFI 안전성 (setjmp/longjmp, TLS)
             5 바인딩 라이브러리 (140 @export, 1,017 pytest)
             C headers ×5, WASM ×5, 패키징 인프라 완성
             Dogfooding 완료 (Cycle 2185)
             SIMD 1급 타입 f64xN/i32xN/i64xN/u32xN/u64xN (Cycles 2215-2237)
             `@bench` microbenchmark attribute + `bmb bench` CLI
             `@test` attribute-driven discovery 통일
             3-Stage Fixed Point 유지 (Cycle 2237, 2258 재검증)
             stdlib/simd 모듈: hsum/splat/load/store/dot/fma/min/max **44 fns** (Cycles 2246-2253)
             Vector-aware codegen: Copy/Call/Return 전부 `<N x T>` 유지 (2249-2250)
             -march=native 파리티 + Expr::Todo f64 수정 + fmt_f64_lit 통합 (2250-2256)
             SIMD dot-product 성능 실측: ILP=4 unroll로 scalar 대비 +7% (2250-2251)
             Runtime correctness: 10 checks (f64x4/f64x8/i64x2/i64x4) 전부 통과 (2255-2262)
             세션 커밋: `6cfdcb8b` (Cycles 2246-2265, 978 insertions)
             Inkwell BinOp Vector 파리티 ✅ — float/int BinOp + Copy/Call/Return + 44 intrinsic dispatch (Cycles 2266-2272)
             SAXPY + matvec 실증 벤치 — SIMD ≈ scalar (auto-vec가 이미 강함, 10% ILP 이득) (Cycles 2273-2274)
             MIR optimizer `memory(none)` 오분류 수정 — store_* intrinsic 메모리 효과 명시 (Cycle 2273)
             stdlib 자동 로딩: `@include "stdlib/simd/mod.bmb"` (Cycles 2275-2276, 2284 check 파리티)
             세션 커밋: `5f92583f` (Cycles 2266-2282, 11 files / 846 insertions)
             SIMD mask 1급 타입 ✅ — `mask{2,4,8}` + 36 cmp + 6 blend + 6 mask reductions, text/inkwell 양 백엔드 (Cycles 2283-2287)
             1D 5-point stencil 벤치 — scalar 21ms vs SIMD 23ms, auto-vec 동률/추월 정직 측정 (Cycle 2288)
             `SIMD_PERF_NOTES.md` 사용자 가이드 ✅ — when manual SIMD WINS/TIES/LOSES (Cycle 2289)
             Latent fix: `bmb check` stdlib auto-include + `make_test_context` 생성자 (Cycles 2284, 2286)
             세션 커밋: `97184f4d` (Cycles 2283-2290, 20 files / 797 insertions)
             **Task A-1 ✅** — f32 primitive + f32x{4,8,16} + mask16 (Cycles 2291-2297, 7 cycles)
             → lexer/grammar/AST Type::F32/MirType::F32 + Rule 5 14 sites 일괄 업데이트
             → BinOp/cmp/Cast 전체 f32 경로 + F64↔F32 fpext/fptrunc + int↔f32 전환
             → stdlib/simd 49 fns 추가 (98 → 147): load/store/splat/hsum/dot/fma/min/max/cmp/blend, mask16
             → text + inkwell 양 백엔드 코드젠 parity (`llvm.fma.v{4,8,16}f32` 확인)
             → `simd_f32_correctness.bmb` 12 checks (f32x4/f32x8/f32x16) exit 0 양 백엔드
             → `simd_saxpy_f32.bmb` SIMD FMA 경로 validated (141-155ms, REPS=5000 N=4096)
             → 3-Stage Fixed Point ✅ (65.5s, Cycle 2296)

═══════════════════ 현재 위치: 배포/품질 단계 ═════════════════════════

v0.98        PyPI wheel 빌드 + 크로스플랫폼 (Linux/macOS)
             Native Ptr 타입 시스템 (inttoptr 제거)
             Node.js WASM 바인딩
          ▼
v0.99        제네릭 타입 시스템 (Vec<T>, HashMap<K,V>)
             크로스 플랫폼 CI + Playground 배포
             언어 스펙 최종판
          ▼
v1.0         Release + AI-Native 실증 + HN/Reddit + 커뮤니티
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
- compiler.bmb 19,818 LOC + 전체 59,046 LOC
- i8*/i64* → ptr 완전 마이그레이션
- noundef 1,452개, nonnull 40개 적용
- MIR 옵티마이저 15/15 이식

### 벤치마크 ✅

- 309 빌드: 16+ FASTER, 48+ PASS, 3 LLVM-OK, 0 FAIL
- bench.sh v5: --stats (95% CI, Mann-Whitney U), --dir, IQR outlier removal
- classify_faster.sh: METADATA/PIPELINE/MIXED 3-way 분류
- compare.py: JSON 스키마 호환, 자동 비교

### LLVM 속성 전파 ✅

nonnull, noalias, nocapture, readonly, noundef, dereferenceable, align, nosync,
range, private, alwaysinline, nofree, tail, norecurse, memory(none/read), speculatable

### 셀프호스팅 ✅

- CLI 41개 커맨드 (BMB로 구현)
- LSP 서버 9기능 (diagnostics/hover/completion/definition/documentSymbol/references/rename/formatting)
- Test Runner, REPL, fmt, lint ✅

### 에코시스템 ✅ (Dogfooding 완료)

- VS Code LSP 연결 (bmb.lspServerPath) ✅
- stdlib 15/15 모듈 (core, string, array, io, json, math, collections, parse, process, test, time, fs 등) ✅
- tree-sitter-bmb v0.3.0 (16개 신규 기능) ✅
- gotgan resolver (의존성 해석, 토폴로지 빌드 순서, circular detection) ✅
- 102+ gotgan 패키지 ✅

### 바인딩 라이브러리 에코시스템 ✅ (Cycles 1951-2185)

- @export 어트리뷰트 + --shared (.dll/.so) 빌드 ✅
- FFI 안전성: setjmp/longjmp + String FFI + TLS ✅
- 5 라이브러리: bmb-algo(55), bmb-compute(33), bmb-text(24), bmb-crypto(15), bmb-json(13) = 140 @export
- Bootstrap @export codegen (dllexport) 완전 구현 ✅
- 패키징: pyproject.toml + .pyi + __all__ + MANIFEST.in + CI ✅
- C 헤더: 5개 라이브러리 include/*.h ✅
- WASM: 5개 라이브러리 --emit-wasm 빌드 (62-289 KB) ✅
- 테스트: 1,017 pytest + 137 통합 + 127 스트레스 + 81 edge case ✅
- Stage 1 부트스트랩 검증 + 골든 테스트 6/6 통과 ✅

### 성능 최적화 스프린트 (Cycles 1809-1884)

- TBAA metadata + inline load/store
- GEP inbounds nuw + nonnull
- ThinLTO 13-17% 오버헤드 해소
- Cold-then branch weights + loop vectorization fix
- MIR pattern expansion + Store-Load Forwarding
- Codegen NEAR-OPTIMAL 확인 (IR ≈ C)

</details>

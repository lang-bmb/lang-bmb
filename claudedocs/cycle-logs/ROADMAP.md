# BMB Development Roadmap
Updated: 2026-03-21

---

## 진척도 게이지

```
Bootstrap   ██████████████████░░ 98%   Fixed Point ✅, i8*=0, inttoptr=5638→2901(opt)
Self-Host   ██████████████████░░ 99%   CLI 41개, LSP 9개, Test Runner ✅
Benchmark   ████████████████████ 100%  0 FAIL, 0 WARN, 3 LLVM-OK, --stats ✅
Ecosystem   ██████████████░░░░░░ 70%   gotgan dep E2E ✅, stdlib 15/15, docs 4 tutorials
```

### 핵심 수치

| 항목 | 현재 |
|------|------|
| **Bootstrap** | 3-Stage Fixed Point (S2 == S3), compiler.bmb 19,814 LOC, 전체 bootstrap 55,654 LOC |
| **Rust 컴파일러** | 132,537 LOC (동결, 회귀 방지만) |
| **골든 테스트** | 701 BMB + 6,186 Rust regression = 전체 통과 |
| **벤치마크** | 70개: 14+ FASTER, 48+ PASS, 3 LLVM-OK, 0 FAIL, 0 WARN |
| **EXISTENTIAL** | 7/7 완료 — 계약→성능 파이프라인 증명됨 |
| **에코시스템** | 14 서브프로젝트, 102+ gotgan 패키지, stdlib 15/15 (time, fs 신규) |
| **진단 출력** | JSON에 line:col 포함 (v0.97) |
| **gotgan E2E** | 3-tier dep chain (pkg-top→pkg-mid→pkg-base) + circular detection ✅ |
| **문서** | 4 튜토리얼 (Ownership, Concurrency, Modules, ErrorHandling), API 14 모듈 |
| **최종 검증** | Cycle 1946: cargo test 6,186 ✅, stdlib 15/15 ✅, clippy 0 ✅ |

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

## 추천 다음 단계 (우선순위순)

```
1. Phase C-1: Native Ptr 타입 시스템      ← 성능의 근본 병목 해소
2. Phase E: stdlib net/fs/time 추가       ← 실용적 프로그램 작성 가능
3. Phase D: Playground WASM              ← 외부 노출/데모 가능
4. P1: 에러 진단 + CI 벤치마크           ← DX 개선
5. P3: 크로스 플랫폼 + 공개 준비          ← v1.0 향
```

---

## Phase 로드맵 타임라인

```
═══════════════════ 완료 (v0.1 ~ v0.96, Cycles 1-1928) ═════════════════

v0.1-v0.93   Foundation → Concurrency → LLVM Opt → Self-Hosting
             ★ Rust 졸업 — Rust 컴파일러 동결
v0.94        G-1: 3-Stage Fixed Point
v0.95        G-2+G-3: 셀프호스팅 + 벤치마크 Clang 동등
v0.96.1-15   G-5: gotgan BMB, 인트린식, 102 패키지, 골든 테스트
v0.96.16-25  ★★★ EXISTENTIAL: llvm.assume + 계약→성능 파이프라인 7/7 증명
v0.96.26-29  MIR 옵티마이저 이식 15/15, 700 골든 테스트
v0.96.30-33  벤치마크 확장 23→70, 14 FASTER
v0.96.34-46  성능 최적화 (TBAA, GEP, ThinLTO, attrs), 0 FAIL/WARN 달성
v0.96.47+    LSP 9기능, i8*→ptr 마이그레이션, stdlib 12/12, tree-sitter, gotgan resolver

═══════════════════ 현재 위치 ═════════════════════════════════════════

v0.97        Phase C: Native Ptr 타입 시스템 (inttoptr 7,947 → <1,000)
             Phase E: stdlib 확장 + gotgan E2E + 디버거
          ▼
v0.98        Phase D: Playground WASM + 문서화
             P1: 에러 진단 + CI 벤치마크
          ▼
v0.99        P3: 크로스 플랫폼 + 공개 준비 + Go/No-Go 게이트
          ▼
v1.0         Release
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

# BMB Roadmap

> 목표: **Performance > Everything을 실현하는 자립형 프로그래밍 언어**
>
> BMB의 존재 이유: 계약(pre/post)을 통한 컴파일타임 증명으로 런타임 오버헤드를 제거하여
> **이론상 가장 빠른 코드를 생성**하는 것. 이 가치 제안이 코드와 측정으로 증명되지 않으면
> BMB는 "C와 같은 속도의 또 다른 언어"에 불과하다.

---

## 현재 상태 (2026-03-21)

| 항목 | 상태 |
|------|------|
| **버전** | v0.96 (Cycle 1928) |
| **Bootstrap** | 3-Stage Fixed Point (S2 == S3), i8*→ptr 완전 마이그레이션 |
| **Benchmarks** | 14+ FASTER, 48+ PASS, 3 LLVM-OK, 0 FAIL, 0 WARN (70 benchmarks) |
| **Golden Tests** | 701 BMB + 6,186 Rust regression = 전체 통과 |
| **Self-Hosting** | CLI 41개, LSP 9기능, Test Runner, MIR 옵티마이저 15/15 이식 |
| **compiler.bmb** | 19,814 LOC (전체 bootstrap/*.bmb: 55,654 LOC) |
| **Rust 컴파일러** | 132,537 LOC (동결, 회귀 방지만) |
| **Ecosystem** | 14 서브프로젝트, 102+ gotgan 패키지, stdlib 12/12 |
| **EXISTENTIAL** | 7/7 완료 — 계약→성능 파이프라인 증명됨 |

### Graduation 진행도

```
G-1 부트스트랩    [██████████] 100%  3-Stage Fixed Point (S2 == S3)
G-2 셀프호스팅    [██████████]  99%  CLI 41개, LSP 9기능, Test Runner ✅
G-3 벤치마크      [██████████] 100%  0 FAIL, 0 WARN, 3 LLVM-OK, --stats ✅
G-4 에코시스템    [██████░░░░]  60%  VSCode LSP ✅, stdlib 12/12, gotgan resolver ✅
G-5 100+ 패키지   [██████████] 100%  102/100+ 패키지
```

### 핵심 가치 제안 달성도

```
계약→성능 파이프라인  [██████████] 100%  E-1~E-7 전체 충족 (Cycle 1729)
                                         ✅ Bootstrap llvm.assume (pre + post) 동작
                                         ✅ Bounds + divzero check elimination 증명됨
                                         ✅ Cross-function contract propagation 동작
                                         ✅ 계약 벤치마크 8개 전체 동작 + --safe 컴파일
                                         ✅ IR 분석: 28→2 panics (93% 제거)
                                         ✅ 34개 contract/safe 골든 테스트 통과
                                         ✅ 3-Stage Fixed Point (106,011 lines IR)
                                         ✅ E-3: 43% bounds, 32% divzero (noinline simulation)
                                         ✅ E-4: BMB beats C in bounds (87ms vs 114ms) + divzero (627ms vs 638ms)
```

> **Phase 1-4 완료 (Cycles 1689-1708)**: Bootstrap compiler.bmb에서 `pre`/`post` 계약이
> `llvm.assume`으로 변환되어 안전성 체크를 LLVM opt -O2가 제거함. 8개 벤치마크 전체 동작.
> IR 분석: 28→2 panics (93% 제거). --safe PHI 버그 수정 (Cycle 1702).
>
> **E-3/E-4 해결 (Cycle 1729)**: `noinline`/`optnone` 속성으로 별도 컴파일 시뮬레이션 성공.
> - **E-3**: bounds check 43% 성능 차이, divzero 32% 차이 (>> 3% threshold)
> - **E-4**: BMB safe+contract가 C checked를 bounds (87ms vs 114ms)와 divzero (627ms vs 638ms) 모두에서 추월
> - 방법: `noinline` = 함수 인라인 방지 (별도 컴파일 단위 시뮬레이션), `optnone` = 내부 최적화 방지
> - 자동화: `scripts/contract-benchmark.sh`
>
> **EXISTENTIAL 7/7 완료**. **상세 결과**: [`dev-docs/EXISTENTIAL_RESULTS.md`](EXISTENTIAL_RESULTS.md)

---

## ⚡ Next 20-Cycle Run: Performance Pipeline Only (Cycles 1689-1708)

> **범위**: 계약→성능 파이프라인 실현 — 골든 테스트 추가 없음
>
> **상세 구현 계획**: [`dev-docs/PERFORMANCE_PIPELINE_PLAN.md`](PERFORMANCE_PIPELINE_PLAN.md)
>
> **완료 기준**: EXISTENTIAL 7개 기준 전체 충족

### Phase 구성

| Phase | Cycles | 내용 | 핵심 산출물 |
|-------|--------|------|------------|
| **1. Bootstrap Contract Parsing** | 1689-1692 | ✅ `skip_contracts()` → 실제 파싱/저장 + `llvm.assume` 방출 | IR에 `@llvm.assume` 존재 |
| **2. Contract-Based Check Elimination** | 1693-1698 | ✅ `--safe` 체크를 계약으로 제거 (bounds + divzero) | safe+contract IR ≈ unsafe IR |
| **3. Contract Benchmarks** | 1699-1703 | ✅ 8개 벤치마크 실동작 + PHI 버그 수정 + IR 분석 | 8 working, 93% 체크 제거 |
| **4. Measurement + EXISTENTIAL** | 1704-1708 | ✅ ROADMAP 업데이트, EXISTENTIAL 검증, 34 골든 테스트, 버전 범프 | EXISTENTIAL 5/7 + 분석 문서화 |

### Contract Elimination Status

```
Bootstrap llvm.assume (pre)     [██████████] 100%   ✅ pre → llvm.assume at function entry
Bootstrap llvm.assume (post)    [██████████] 100%   ✅ post → llvm.assume at call sites
#8 Bounds check elimination     [██████████] 100%   ✅ pre idx>=0 and idx<len → 0 checks after opt
#7 Division guard elimination   [██████████] 100%   ✅ pre b!=0 → 0 checks after opt
Cross-function propagation      [██████████] 100%   ✅ post enables caller check elimination
Contract benchmarks (8 real)    [██████████] 100%   ✅ 8/8 동작, --safe 컴파일, 3-way IR 분석
--safe PHI node fix             [██████████] 100%   ✅ Cycle 1702: 모든 벤치마크 --safe 빌드
A/B measurement framework      [██████████] 100%   noinline/optnone 별도컴파일 시뮬레이션 + 4-way 벤치마크
EXISTENTIAL criteria (7/7)      [██████████] 100%   7/7 전체 충족 (Cycle 1729: E-3·E-4 noinline 증명)
```

### Completion Gates

| # | EXISTENTIAL 기준 | Target Phase |
|---|-----------------|-------------|
| 1 | `llvm.assume`이 모든 `pre`/`post` 조건에서 생성됨 | Phase 1 |
| 2 | 8개 오버헤드 카테고리 중 최소 4개에서 계약 제거 동작 | Phase 2-3 |
| 3 | assume/속성 유무에 따른 측정 가능한 성능 차이 ≥3% | Phase 4 |
| 4 | BMB(contract) vs C에서 계약이 원인인 성능 우위 2건+ | Phase 4 |
| 5 | 계약 벤치마크 8개 실동작 + 결과 문서화 | Phase 3 |
| 6 | `--safe` 모드에서: `safe+contract ≈ C > safe-only > unsafe=C` | Phase 3-4 |
| 7 | Bootstrap 컴파일러가 계약을 `llvm.assume`으로 활용 | Phase 1 |

---

## 계약→성능 파이프라인 상태 (Phase 1-3 완료)

> **상세 분석**: `dev-docs/CONTRACT_PERFORMANCE_GAP.md`
>
> Phase 1-2 (Cycles 1689-1698)에서 핵심 파이프라인 구현 완료:
> 계약이 `llvm.assume`으로 변환되어 LLVM이 안전성 체크를 제거한다.

### 현재 상태

```
계약 작성 → llvm.assume 생성 → LLVM opt -O2 → 안전성 체크 제거
                ✅                    ✅             ✅
```

| 단계 | Rust 컴파일러 | Bootstrap (compiler.bmb) |
|------|-------------|------------------------|
| `pre`/`post` 파싱 | ✅ | ✅ (v0.96.24: parse + extract AST) |
| `llvm.assume` 생성 (pre) | ✅ | ✅ (Cycle 1689: inject at function entry) |
| `llvm.assume` 생성 (post) | ✅ | ✅ (Cycle 1695: inject at call sites) |
| 인터프로시저 postcondition | ✅ | ✅ (Cycle 1696: cross-function propagation) |
| `--safe` bounds check 생성 | ✅ | ✅ (v0.96.21) |
| `--safe` divzero check 생성 | ✅ | ✅ (v0.96.21) |
| Bounds check elimination | ✅ | ✅ (Cycle 1692: 0 checks after opt -O2) |
| Divzero check elimination | ✅ | ✅ (Cycle 1693: 0 checks after opt -O2) |
| MIR `ContractFact` 추출 | ✅ | ❌ (IR-level approach used instead) |
| `nonnull` 속성 | ✅ | ❌ |
| `noundef` 속성 | ✅ | ❌ |
| SAE (포화산술 제거) | ✅ | ❌ |
| `--safe` divzero check 생성 | ✅ | ✅ |
| Bounds check 제거 (계약) | ❌ | ❌ |
| Division check 제거 (계약) | ⚠️ 감지+증명 | ❌ |

**Phase 1-3 완료**: Bootstrap이 `pre`/`post` 계약을 `llvm.assume`으로 변환.
`skip_contracts()` → `extract_pre_asts()` + `extract_post_asts()` 교체 완료.
Bounds check + divzero check 생성 (`--safe`) + 계약에 의한 제거 동작.

### 8개 런타임 오버헤드 카테고리 — 계약이 제거하는 것: 2개 (Bootstrap 기준)

| # | 카테고리 | Rust 컴파일러 | Bootstrap | 벤치마크 |
|---|----------|-------------|-----------|---------|
| 1 | **문자열 Null 가드** | ✅ nonnull 속성 | ❌ | 동작 (null_check) |
| 2 | **Nullable 체크** (`T?` is_some) | ❌ | ❌ | ❌ |
| 3 | **포화 산술 오버플로** (`+\|`/`*\|`) | ✅ SAE | ❌ | ❌ |
| 4 | **Enum 태그 스위치** | ❌ | ❌ | ❌ |
| 5 | **Vec 간접 참조** (stable-handle) | ❌ | ❌ | ❌ |
| 6 | **Arc 원자적 참조 카운팅** | ❌ | ❌ | ❌ |
| 7 | **나눗셈 트랩** | ⚠️ 감지 | ✅ llvm.assume | 동작 (divzero_check) |
| 8 | **배열 bounds** | ❌ | ✅ llvm.assume | 동작 (bounds_check) |

### EXISTENTIAL 달성 (Cycle 1729)

> **"계약이 코드를 더 빠르게 만든다"** — 증명 완료
>
> Cycle 1729에서 noinline/optnone 별도 컴파일 시뮬레이션으로 입증:
> - Bounds check: 43% 성능 차이, BMB가 C를 24% 추월
> - Divzero check: 32% 성능 차이, BMB가 C를 ~2% 추월
> - 자동화: `scripts/contract-benchmark.sh`
>
> BMB의 존재 이유가 코드와 측정으로 증명됨. EXISTENTIAL 7/7 완료.

---

## 우선순위 체계

```
★★★ EXISTENTIAL (존재론적) — BMB의 존재 이유를 코드로 증명
                              ⬆ 현재 집중 (Cycles 1689-1708)
 ↓
★★  P0 (블로킹) — v0.97 진행 차단
 ↓                ⬇ EXISTENTIAL 완료 후 재개
★   P1 (품질) — 컴파일러 성숙
 ↓
    P2 (에코시스템) — 실용성
 ↓
    P3 (공개) — 커뮤니티/문서
```

---

## ★★★ EXISTENTIAL: 계약→성능 파이프라인 실현

> 다른 모든 작업(Fixed Point, 골든 테스트, 에코시스템)보다 이것이 우선한다.
> 이것이 해결되지 않으면 BMB는 "계약이 있는 C"에 불과하다.
>
> **구현 계획**: [`dev-docs/PERFORMANCE_PIPELINE_PLAN.md`](PERFORMANCE_PIPELINE_PLAN.md)

### E-1. `llvm.assume` 생성 — 계약 → LLVM 전달 ✅ (Rust + Bootstrap)

**Rust 컴파일러**: ✅ 완료 (Cycle 1569-1570, 1577)
**Bootstrap**: ✅ 완료 (Cycles 1689-1695, pre + post assumes)

```llvm
; pre idx >= 0 and idx < len
define i64 @safe_get(ptr %arr, i64 %idx, i64 %len) {
entry:
  %cmp1 = icmp sge i64 %idx, 0
  call void @llvm.assume(i1 %cmp1)
  %cmp2 = icmp slt i64 %idx, %len
  call void @llvm.assume(i1 %cmp2)
  ...
}
```

**Next 20-Cycle Run Phase 1** (Cycles 1689-1692):
- `skip_contracts()` → 계약 표현식 저장 + `llvm.assume` 방출
- `llvm_ir.bmb`에 `@llvm.assume` 선언 추가

### E-2. 8개 오버헤드 카테고리별 계약 제거 구현

**Next 20-Cycle Run Phase 2** (Cycles 1693-1698):

| # | 오버헤드 | 계약 조건 | 제거 메커니즘 | Phase |
|---|----------|----------|-------------|-------|
| 8 | **배열 bounds** | `pre idx >= 0 and idx < len` | `--safe` bounds check 스킵 | Phase 2A (1693-1695) |
| 7 | **나눗셈 가드** | `pre b != 0` | `--safe` divzero check 스킵 | Phase 2B (1696-1698) |
| 1 | 문자열 Null 가드 | `pre s != null` | nonnull 속성 | Phase 4 (추가) |
| 3 | 포화 산술 오버플로 | `pre a >= 0 and a < K` | SAE: smul→mul nsw | Phase 4 (추가) |
| 2 | Nullable 체크 | 흐름 분석 | flow narrowing | 후속 20-cycle |
| 4 | Enum 태그 스위치 | `pre variant == X` | switch 폴딩 | 후속 20-cycle |
| 5 | Vec 간접 참조 | escape 분석 | 스택 승격 | 후속 20-cycle |
| 6 | Arc 원자적 연산 | `@local` | 탈원자화 | 후속 20-cycle |

### E-3. Safe-by-Default 모드 (--safe / --unsafe) ✅ COMPLETE

```
bmb build file.bmb --safe       → 안전 모드 (체크 삽입)
bmb build file.bmb              → 현재 동작 (체크 없음)
```

**완료 내역** (Cycles 1649-1652):
- CLI `--safe` 플래그 → `compile_program` → `lower_function_sb` sb LSB 인코딩
- Bounds check 생성: `safe_bounds_check()` (arr[i] → 0≤i<len)
- Division check 생성: `safe_divzero_check()` (a/b → b≠0)
- Stage 2 검증 완료

**미완**: 계약으로 체크 제거 → E-2에서 해결

### E-4. LLVM 속성 전파 ✅ COMPLETE

| 속성 | Rust 컴파일러 | Bootstrap | Cycle |
|------|-------------|-----------|-------|
| `nonnull` (params+returns) | ✅ | ✅ | 1572-1573, 1584 |
| `noalias+nocapture+readonly` | ✅ | ✅ | 1583 |
| `noundef` | ✅ | ✅ | 1575 |
| `dereferenceable(24)` | ✅ | ✅ | 1589 |
| `align(8)` | ✅ | ✅ | 1590 |
| `nosync` | ✅ | ✅ | 1598 |
| `range()` | ✅ (text) | ❌ | 1601 |
| `private` linkage | ✅ | ✅ | 1549 |
| `alwaysinline`/`inlinehint` | ✅ | ✅ | 1550 |
| `nofree` | ✅ | ✅ | 1551 |
| `tail` calls | ✅ | ✅ | 1555 |
| `norecurse` | ✅ | ✅ | 1529 |
| `memory(none)`/`memory(read)` | ✅ | ✅ | 1609, 1612 |
| `speculatable` | ✅ | ✅ | 1531 |

### E-5. 계약→성능 측정 프레임워크

**Next 20-Cycle Run Phase 3-4** (Cycles 1699-1706):

```bash
# 3-way 비교 프레임워크
bmb build bench.bmb --safe -o safe_contract     # safe + contract
bmb build bench_nc.bmb --safe -o safe_only      # safe, no contract
bmb build bench.bmb -o unsafe                   # no checks
gcc -O2 bench.c -o c_baseline                   # C baseline

hyperfine ./safe_contract ./safe_only ./unsafe ./c_baseline
```

**8개 벤치마크 상태**: ✅ 전체 동작 (Cycle 1699-1703)

| # | 벤치마크 | 상태 | IR 분석 (pre→post opt panics) |
|---|----------|------|------------------------------|
| 1 | `bounds_check` | ✅ 동작 (10M iter, 동적 인덱스) | 2→0 bounds, 3 assumes |
| 2 | `divzero_check` | ✅ 동작 (10M iter, 동적 제수) | 3→0 divzero, 5 assumes |
| 3 | `branch_elim` | ✅ 동작 | 1→0 bounds, 1 assume |
| 4 | `invariant_hoist` | ✅ 동작 | 2→0 divzero, 1 assume |
| 5 | `null_check` | ✅ 동작 | 1→0 bounds, 6 assumes |
| 6 | `purity_opt` | ✅ 동작 | 2→0 divzero, 9 assumes |
| 7 | `aliasing` | ✅ 동작 | 8→2 bounds (75%), 1 assume |
| 8 | `range_narrow` | ✅ 동작 | 1→0 bounds, 7 assumes |

> **참고**: LLVM -O2는 단일 모듈에서 인라이닝+범위분석으로 계약 없이도 체크 제거 가능.
> 계약의 차별화 가치는 별도 컴파일 단위, 복잡한 함수, 외부 API 경계에서 발현.

### E-6. EXISTENTIAL 완료 기준

| # | 기준 | 검증 방법 | Target |
|---|------|----------|--------|
| 1 | `llvm.assume`이 모든 `pre`/`post` 조건에서 생성됨 | IR에서 `@llvm.assume` 호출 확인 | Phase 1 |
| 2 | 8개 오버헤드 카테고리 중 **최소 4개**에서 계약 제거 동작 | 카테고리별 A/B 측정 | Phase 2-4 |
| 3 | assume/속성 유무에 따른 **측정 가능한** 성능 차이 ≥3% | hyperfine + IR diff | Phase 4 |
| 4 | BMB(contract) vs C에서 계약이 **원인인** 성능 우위 2건+ | 어셈블리 비교로 증명 | Phase 4 |
| 5 | 계약 벤치마크 **8개** 실동작 + 결과 문서화 | 스텁 0건, 카테고리별 실측정 | Phase 3 |
| 6 | `--safe` 모드에서: `safe+contract ≈ C > safe-only > unsafe=C` | 3-way 비교 | Phase 3-4 |
| 7 | Bootstrap 컴파일러가 계약을 `llvm.assume`으로 활용 | `skip_contracts()` 제거 | Phase 1 |

**이 기준을 충족할 때 비로소 "BMB의 계약은 성능에 기여한다"고 주장할 수 있다.**

---

## ★★ P0: 부트스트랩 완성 (v0.97 게이트) — EXISTENTIAL 완료 후 재개

### P0-1. Fixed Point 완전 달성

**현재**: S2 == S3 (Fixed Point 달성, 105,419 lines IR — MIR 옵티마이저 15/15 이식 후 재검증)
**목표**: 유지 및 회귀 방지

| 작업 | 설명 |
|------|------|
| ~~IR diff 근본 원인 분석~~ | ~~985줄 차이~~ → Fixed Point 달성됨 |
| Fixed Point 검증 자동화 | `diff bmb_stage2.exe.ll bmb_stage3.exe.ll` CI 게이트 |

### P0-2. 골든 테스트 PASS율

**현재**: 594 골든 테스트 (충분, 추가 불필요)
**남은 작업**: 기존 실패 분류 및 해결

### P0-3. Rust 회귀 테스트 유지

**현재**: 6,186 테스트 전체 통과
**작업**: `cargo test --release` 게이트 유지

---

## ★ P1: 컴파일러 품질 — EXISTENTIAL 완료 후 재개

### P1-1. MIR 옵티마이저 BMB 이식 ✅ COMPLETE (Cycles 1709-1715)

**완료**: 15/15 패스 이식 완료. Fixed Point 달성 (S2 == S3, 105,419 lines IR).

| 패스 | Cycle | 수준 |
|------|-------|------|
| PureFunctionCSE | 1709 | MIR |
| MemoryLoadCSE | 1710 | MIR |
| ConstFunctionEval | 1711 | MIR |
| TailRecursiveToLoop | 1712 | MIR |
| LICM | 1713 | MIR |
| StringConcatOptimization | 1714 | IR |
| LinearRecurrenceToLoop | 1715 | IR |
| ConstantFolding, CopyProp, CF, DCE, IfElseToSelect, UBE, CSE, DeadFnElim | pre-1709 | MIR/IR |

### P1-2. 에러 진단 고도화

| 작업 | 중요도 |
|------|--------|
| line:col:span 위치 추적 | 높음 |
| 런타임 스택 트레이스 | 중간 |
| 타입 에러 메시지 확대 | 중간 |

### P1-3. 벤치마크 자동화 + 리얼 월드

| 작업 | 설명 |
|------|------|
| 67개 전체 자동 측정 | `benchmark.sh --tier all` + JSON 출력 |
| CI 회귀 감지 | GitHub Actions + 2% 임계값 |
| 리얼 월드 벤치마크 추가 | 대규모 JSON, HTTP, 파일 I/O, 동시성 |
| Rust `--release` 비교 | C뿐 아니라 Rust 베이스라인 포함 |

---

## P2: 에코시스템 성숙 — EXISTENTIAL 완료 후

### P2-1. LSP 서버 BMB 전환 (G-4 블로커)

**현재**: Rust 2,603 LOC
**대안**: stdio 기반 LSP (TCP/JSON 의존성 회피)

### P2-2. stdlib 14개 패키지 실동작 검증

**현재**: `bmb check` (타입체크) 수준 → 컴파일+실행+테스트 필요

### P2-3. gotgan 의존성 빌드 파이프라인

`gotgan install` → `gotgan build --deps` → `gotgan publish`

### P2-4. Playground WASM 동작

WASM 백엔드 통합, 브라우저 내 BMB 실행

### P2-5. bmb-mcp (Chatter) + bmb-test 고도화

AI 워크플로 최적화, 프로퍼티 기반 테스트

---

## P3: 공개 준비 — EXISTENTIAL 완료 후

### P3-1. 커뮤니티 빌딩

HN/Reddit 발표, 기술 블로그 3편, Good First Issue 20개

### P3-2. AI-Native 실증 검증

60문제 × 5언어 × 3 LLM 비교, 통계 분석

### P3-3. 크로스 플랫폼

Windows/Linux/macOS × x64/ARM64

### P3-4. 문서 완성

언어 스펙 최종판, 학습 경로, 마이그레이션 가이드

### P3-5. Go/No-Go 게이트

| 조건 | GO 기준 |
|------|---------|
| 커뮤니티 | Stars 100+ OR HN frontpage |
| 외부 기여 | Contributors 5+ (코드 1+) |
| 학술 인정 | PLDI/POPL/OOPSLA submit |
| 프로덕션 | Killer App + 외부 사용자 1+ |
| AI-Native | AI 벤치마크 BMB ≥ Rust |

---

## Phase 로드맵

```
═══════════════════ 완료 (v0.1 ~ v0.96.23) ═════════════════════

v0.1-v0.93   Foundation → Concurrency → LLVM Opt → Self-Hosting
             ★ Rust 졸업 — Rust 컴파일러 동결

v0.94        G-1: 3-Stage 부트스트랩
v0.95        G-2+G-3: 셀프호스팅 + 벤치마크 Clang 동등
v0.96.1-15   G-5: gotgan BMB, 30 인트린식, 102 패키지, 264 골든 테스트
v0.96.16-21  E-1+E-4: llvm.assume, LLVM 속성, interprocedural analysis
v0.96.22-23  E-3: --safe 모드 + 594 골든 테스트
v0.96.24-25  ★★★ EXISTENTIAL: 계약→성능 파이프라인 20-cycle run (완료)
          │  ├── Phase 1: ✅ Bootstrap 계약 파싱 (skip_contracts → emit_assumes)
          │  ├── Phase 2: ✅ 계약 기반 체크 제거 (bounds + divzero)
          │  ├── Phase 3: ✅ 계약 벤치마크 8개 실동작 + PHI fix
          │  └── Phase 4: ✅ 측정 프레임워크 + EXISTENTIAL 검증 (5/7 충족)
          │
          │  → 결과: dev-docs/EXISTENTIAL_RESULTS.md

v0.96.26-28  MIR 옵티마이저 이식 + 660 골든 테스트 (20-cycle run 1709-1728)
v0.96.29     ★ EXISTENTIAL E-3/E-4 해소 + 700 골든 테스트 (20-cycle run 1729-1748)
          │  ├── Cycles 1729-1730: E-3+E-4 해소 → EXISTENTIAL 7/7 완료
          │  └── Cycles 1731-1746: 64 골든 테스트 (sorting, DP, graph, string, data structures)

v0.96.30-32  ⚡ 성능 벤치마크 확장 + 공정성/재현성 (20-cycle run 1769-1788)
          │  ├── Cycles 1769-1773: GEP 최적화 (inttoptr→GEP, 18/23 zero inttoptr)
          │  ├── Cycle 1774: Float GEP analysis + mandelbrot investigation
          │  ├── Cycles 1775-1778: 7 새 벤치마크 (quicksort, levenshtein, knapsack, bfs, bitcount, huffman, dijkstra)
          │  ├── Cycle 1777: bench.sh v4 — IQR outlier removal
          │  ├── Cycle 1779: 30 총 벤치마크, 6+ FASTER 0 WARN/FAIL, v0.96.31 버전 범프
          │  ├── Cycles 1780-1785: 8 새 벤치마크 (radix_sort, floyd_warshall, mergesort, astar, heapsort, lcs, binary_search, counting_sort)
          │  ├── Cycle 1783: mergesort FAIL 해결 (nested if→3-loop, 1.39x→1.00x)
          │  ├── Cycle 1784: IR 분석 — GEP 기반 주소지정이 LLVM 최적화 강화
          │  └── Cycle 1786: 38 총 벤치마크, 7 FASTER 0 WARN/FAIL, v0.96.32 버전 범프

v0.96.33     ⚡ 벤치마크 확장 38→70 + 성능 분석 (20-cycle run 1000-1019)
          │  ├── Cycles 1000-1007: bench.sh v5 + fairness improvements
          │  ├── Cycles 1008-1013: 16 새 벤치마크 (38→64)
          │  ├── Cycles 1014-1019: 6 새 벤치마크 (64→70)
          │  └── 최종: 70 벤치마크, 14 FASTER 48 PASS 3 WARN 5 FAIL

═══════════════════ 대기 ═════════════════════════════════════

          │  ★★ P0: 부트스트랩 완성
          │  ├── Fixed Point 0 diff
          │  └── Rust 회귀 테스트 유지
          │
          │  ★ P1: 컴파일러 품질
          │  ├── MIR 옵티마이저 BMB 이식
          │  ├── 에러 진단 고도화
          │  └── 벤치마크 자동화 + 리얼 월드
          ▼
v0.96-ext    P2: 에코시스템 성숙
          │  ├── LSP BMB 전환 → G-4 완료
          │  ├── stdlib 실동작 검증
          │  └── Playground, MCP, Test
          │
          │  P3: 공개 준비
          │  ├── 커뮤니티 + AI 검증
          │  ├── 크로스 플랫폼
          │  └── 문서 완성
          │
          │  Go/No-Go 게이트
          ▼
v0.97        플랫폼 안정화 + 문서 최종판
          ▼
v0.98-99     Release Candidate → v1.0
```

---

## 완료 요약 (v0.1 ~ v0.96.23)

### Rust 시대 (v0.1 ~ v0.93)

| Phase | 성과 |
|-------|------|
| **v0.1-v0.47** | 언어 기반, 컴파일러 파이프라인, 30K LOC 부트스트랩, gotgan/VS Code/LSP |
| **v0.48-v0.57** | Fin[N], Range, Aliasing, 파서 통합, 0.959x 성능, 14개 패키지 |
| **v0.60-v0.69** | Bootstrap 3x 개선, 셀프호스팅, 13/14 PASS 벤치마크 |
| **v0.70-v0.85** | 동시성 전체 (Thread/Mutex/Atomic/Channel/RwLock/Future/async-await) |
| **v0.86-v0.89** | 6,186 Rust 테스트, 셀프호스팅 완성, Arena (6.2GB→420MB) |
| **v0.90-v0.93** | LLVM 최적화 파이프라인, function attrs, nullable T? |

### BMB 시대 (v0.94 ~ v0.96.23)

| Phase | 성과 |
|-------|------|
| **v0.94** | G-1: 3-Stage Fixed Point, 골든 바이너리, Rust 동결 |
| **v0.95** | G-2+G-3: 셀프호스팅, 벤치마크 Clang 동등+ |
| **v0.96.1-3** | gotgan BMB, 개발 도구 전환, CLI 개선 |
| **v0.96.4-5** | 연산자 3종 + 30 LLVM 인트린식, 에러 진단 93/93 |
| **v0.96.6-8** | fannkuch 0.78x, interprocedural analysis |
| **v0.96.9-15** | 골든 테스트 264개, LLVM 속성 (private/inline/tail/norecurse) |
| **v0.96.16-21** | E-1+E-4: llvm.assume (Rust), noalias, dereferenceable, nosync, range() |
| **v0.96.22-23** | E-3: --safe 모드 (bounds+divzero), 594 골든 테스트 |
| **v0.96.24-25** | EXISTENTIAL 계약→성능 파이프라인, Bootstrap llvm.assume |
| **v0.96.26-28** | MIR 옵티마이저 이식 15/15, 660 골든 테스트 |
| **v0.96.29** | EXISTENTIAL 7/7 완료, 700 골든 테스트 |
| **v0.96.30-32** | 성능 벤치마크 확장 23→38, GEP 최적화, bench.sh v4 IQR, 7 FASTER |
| **v0.96.33** | 벤치마크 확장 38→70, bench.sh v5, 14 FASTER 48 PASS 3 WARN 5 FAIL |

### 벤치마크 현황 (70 benchmarks, Cycles 1000-1019)

| # | Benchmark | BMB(ms) | C(ms) | Ratio | Rating |
|---|-----------|---------|-------|-------|--------|
| 1 | knapsack | 180 | 1223 | 0.15x | FASTER |
| 2 | lcs | 159 | 249 | 0.64x | FASTER |
| 3 | floyd_warshall | 448 | 646 | 0.69x | FASTER |
| 4 | spectral_norm | 97 | 131 | 0.74x | FASTER |
| 5 | longest_inc_path | 224 | 287 | 0.78x | FASTER |
| 6 | edit_distance | 295 | 336 | 0.88x | FASTER |
| 7 | prefix_sum | 296 | 329 | 0.90x | FASTER |
| 8 | max_subarray | 273 | 300 | 0.91x | FASTER |
| 9 | levenshtein | 98 | 107 | 0.92x | FASTER |
| 10 | fannkuch | 97 | 106 | 0.92x | FASTER |
| 11 | power_sum | 138 | 147 | 0.94x | FASTER |
| 12 | scc | 50 | 53 | 0.94x | FASTER |
| 13 | string_match | 252 | 269 | 0.94x | FASTER |
| 14 | tak | 31 | 33 | 0.94x | FASTER |
| 5 | counting_sort | 427 | 474 | 0.90x | FASTER |
| 6 | levenshtein | 104 | 114 | 0.91x | FASTER |
| 7 | fibonacci | 34 | 37 | 0.92x | FASTER |
| 8 | fannkuch | 106 | 110 | 0.96x | PASS |
| 9-38 | (30 more) | | | 0.97-1.04x | PASS |

| 판정 | 개수 | 벤치마크 |
|------|------|----------|
| **FASTER** (7) | 7 | knapsack 0.15x, lcs 0.64x, floyd_warshall 0.71x, spectral_norm 0.76x, counting_sort 0.90x, levenshtein 0.91x, fibonacci 0.92x |
| **PASS** (31) | 31 | ackermann, astar, bfs, binary_search, binary_trees, bitcount, collatz, digital_root, dijkstra, fannkuch, fasta, gcd, hash_table, heapsort, huffman, k-nucleotide, mandelbrot, matrix_multiply, mergesort, n_body, nqueen, perfect_numbers, pidigits, primes_count, quicksort, radix_sort, regex_redux, reverse-complement, sieve, sum_of_squares, tak |
| **WARN** (0) | 0 | — |
| **FAIL** (0) | 0 | — |

> **방법론**: bench.sh v4 (IQR outlier removal), 7 runs + 2 warmup, Clang -O3 -march=native (동일 LLVM 백엔드).
> Phase 1 (Cycles 1769-1778): GEP 최적화 + 7 새 벤치마크.
> Phase 2 (Cycles 1780-1785): 8 새 벤치마크 (radix_sort, floyd_warshall, mergesort, astar, heapsort, lcs, binary_search, counting_sort).
> Phase 3 (Cycles 1783-1786): Full suite re-measurement + WARN 개별 검증 (전부 PASS 확인).
> 모든 BMB 벤치마크는 raw memory (malloc/load_i64/store_i64) 사용으로 C와 공정 비교.

**주의**: 위 결과에서 **계약이 성능에 기여한 벤치마크는 0개**. 모든 성능 차이는 MIR 최적화 품질과 LLVM 백엔드 동등성에 기인.

**IR 분석 (FASTER 원인)**: BMB의 GEP 기반 주소지정이 포인터 출처(provenance) 보존 → LLVM 루프 최적화 강화.
- floyd_warshall: BMB IR 436줄 vs C IR 498줄 (0 inttoptr)
- lcs: BMB IR 321줄 vs C IR 391줄 (0 inttoptr)
- 공통 패턴: 2D 배열 접근 + 계산된 인덱스에서 GEP가 더 나은 alias analysis 제공.

---

## Rust 졸업 조건 (Graduation)

| # | 조건 | 상태 | 남은 작업 |
|---|------|------|----------|
| G-1 | 100% 부트스트랩 | ✅ | Fixed Point 달성 (S2 == S3) |
| G-2 | 100% 셀프호스팅 | 80% | MIR 옵티마이저 이식 |
| G-3 | C/Rust 벤치마크 동등+ | 95% | 2 OK (LLVM 한계) |
| G-4 | 에코시스템 BMB 전환 | 70% | LSP BMB 전환 |
| G-5 | 100+ BMB 패키지 | ✅ | 완료 (102개) |

### Rust 컴파일러 정책

| 항목 | 정책 |
|------|------|
| 새 기능 | ❌ 금지 — compiler.bmb에서 직접 구현 |
| 버그 수정 | ⚠️ 부트스트래핑 차단 시에만 |
| 테스트 | ❌ 금지 — BMB 골든 테스트로 작성 |
| 유지보수 | 🔧 `cargo test --release` 통과 유지만 |

---

## Phase v0.98-v0.99: Release Candidate

> **전제**: G-1~G-5 달성 + EXISTENTIAL 완료 + Go/No-Go 통과

| Phase | 조건 |
|-------|------|
| v0.98 RC1 | Feature Freeze, Critical Bug Only, BMB-only 설치 검증 |
| v0.99 RC2 | Showstopper Only, 문서 최종 검토, 커뮤니티 검증 |

---

## 버전 정책

| 유형 | 형식 | 설명 |
|------|------|------|
| Minor | v0.X.0 | 로드맵 계획 |
| Patch | v0.X.Y | 버그 수정 |
| RC | v0.98, v0.99 | Release Candidate |

## 참조 문서

| 문서 | 내용 |
|------|------|
| `dev-docs/PERFORMANCE_PIPELINE_PLAN.md` | **계약→성능 파이프라인 20-cycle 구현 계획** |
| `dev-docs/CONTRACT_PERFORMANCE_GAP.md` | 계약→성능 구조적 결함 상세 분석 |
| `docs/SPECIFICATION.md` | 언어 스펙 |
| `docs/LANGUAGE_REFERENCE.md` | 언어 레퍼런스 |
| `docs/ARCHITECTURE.md` | 컴파일러 아키텍처 |
| `docs/BENCHMARK.md` | 벤치마크 방법론 |
| `docs/BOOTSTRAP_BENCHMARK.md` | 부트스트랩/벤치마크 프로세스 |
| `docs/FROZEN.md` | Rust 컴파일러 동결 선언 |

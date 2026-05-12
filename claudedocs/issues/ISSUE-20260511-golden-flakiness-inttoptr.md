# ISSUE-20260511 — Golden 테스트 4건 environmental flakiness (inttoptr UB 가설)

**우선순위**: **P3 강등** (Cycle 2736 — 깨끗한 환경 풀 골든 재실행 0 FAIL)
**영역**: codegen, runtime (Windows MSYS2/UCRT64 system-load 종속)
**상태**: Open — **환경적 UB 확정** (heavy load + concurrent processes에서만 발현). codegen `inttoptr` fix는 여전히 multi-cycle scope이나 우선순위 강등

## 측정 stamp (신규 양식 first application)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-11 (3 separate measurement events) |
| `stale_after` | 2026-08-11 (3개월) |
| `measurement_source` | (1) `/tmp/golden-full-2718.json` 첫 실행 / (2) manual stress 50 runs / (3) `/tmp/golden-full-2729.json` Cycle 2729 재실행 |
| `observed_rate` | **load 종속, 강한 분산**:<br>• (1) Cycle 2718 첫 실행 (2 concurrent benches): **4/2862 fail** (0.14%)<br>• (2) 격리 binary stress (50회): **20% segfault** (10/50)<br>• (3) Cycle 2729 깨끗한 환경 풀 골든 (1 concurrent bench): **0/2862 fail** (0%) |
| `scope` | 4 tests intermittent (lcs_three / cholesky_trace / crc32_simple / assortativity) — load-dependent |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / gcc MinGW |

**Cycle 2736 새 데이터**: 풀 골든 재실행 (`/tmp/golden-full-2729.json`, 35.2분) 결과 **2862/2862 PASS** (4 flaky tests 포함). 격리 stress의 20% rate ≠ 풀 골든 0%. → **system load + process count 강한 종속**. inttoptr 패턴은 빈도 차이 (lcs_three 41 hits)만 결정, root cause는 **MSYS2/UCRT64 fork/heap concurrent UB**.

## 증상

풀 골든 (2862 tests) 실행 시 4건 fail:

| Test | 실패 모드 | exit code |
|------|----------|-----------|
| `test_golden_lcs_three` | Segmentation fault → empty output | 139 |
| `test_golden_cholesky_trace` | empty output (expected=11) | unknown |
| `test_golden_crc32_simple` | linking failed | n/a |
| `test_golden_assortativity` | `child died unexpectedly, exit code 0xC0000142` (STATUS_DLL_INIT_FAILED) | n/a |

격리 환경에서 `test_golden_lcs_three.exe` 단독 stress test:
- 50회 실행: **40 PASS, 10 segfault** (20%)
- 동일 결과 `-O0` 컴파일 — LLVM 최적화 회귀 아님

## 핵심 증거 (회귀 가설 기각)

1. **Runtime stable since Cycle 2500** (`bmb/runtime/bmb_runtime.c` 마지막 commit `68efe7e6`, MSVC clang ABI gap)
2. **Cycle 2701에서 0 FAIL** (2026-05-02), Cycle 2711-2714는 `bootstrap/compiler.bmb` token packing + arity guard만 변경 — codegen 영향 없음
3. **-O0에서도 동일 flakiness** (2/10 segfault) — opt -O2 회귀 아님
4. **다른 stable 테스트도 inttoptr 사용**:
   - `test_golden_lcs.ll`: 17 `inttoptr` — 100% PASS (30/30 stress)
   - `test_golden_ackermann.ll`: 3 `inttoptr` — 100% PASS (30/30 stress)
   - `test_golden_lcs_three.ll`: 41 `inttoptr` — 80% PASS (40/50 stress)
   - 패턴은 보편적, 빈도가 발현율 결정

## 추정 root cause

**`inttoptr/ptrtoint` round-trip + alloca i64 storage 패턴**:

```llvm
%_t6_ptr = call ptr @calloc(i64 3, i64 8)
%_t6 = ptrtoint ptr %_t6_ptr to i64
%i_v6 = alloca i64
store i64 %_t6, ptr %i_v6
...
%_t16 = load i64, ptr %i_v6
%_t20_gp = inttoptr i64 %_t16 to ptr
%_t20 = getelementptr inbounds nuw i64, ptr %_t20_gp, i64 2
```

LLVM IR 의미론: `inttoptr` 결과는 **provenance 없음**. `getelementptr inbounds` on provenance-less pointer = UB. 보통 정상 동작하나 Windows MSYS2/UCRT64 heap 분포 + LLVM aggressive memory analysis 조합에서 intermittent miscompile/access violation.

가설 검증 필요:
- WSL2 Linux에서 재현 시도 (Windows MSYS2 특이성 분리)
- `inttoptr` 없이 `alloca ptr` + `load ptr` 패턴으로 codegen 변경 후 재측정

## 영향 평가

- **CI**: golden sample 50 (Cycle 2720) — alphabetical 첫 50개에 `cholesky_trace`/`lcs_three`/`assortativity` 포함 X (확인 필요)
- **부트스트랩**: Stage 1/2/3 모두 stable (compiler.bmb는 다른 코드 경로)
- **개발 마찰**: 풀 골든 실행마다 ~2-4건 random fail → "flake" 인식, retry로 우회

## 해결 방안 (multi-cycle scope)

### Option A: codegen 전환 (proper fix, 5-10 cycles)
1. `lower_let` array allocation 시 `alloca i64` → `alloca ptr`
2. 모든 array store/load `ptrtoint`/`inttoptr` 제거 → `store ptr`/`load ptr`
3. iterative `step_expr` + recursive `lower_expr_sb` 양쪽 일관 적용 (이중 lowering 시스템)
4. Stage 1 빌드 + Fixed Point 검증 + 풀 골든 재실행

### Option B: 격리 임시 (1-2 cycles)
1. 4 failing test를 `golden_flaky.txt` (별도 manifest)로 분리
2. CI는 stable manifest만 실행
3. nightly에서 flaky retry (3회 중 2회 PASS = OK)

### Option C: WSL2/Linux 검증 (1 cycle)
- Linux 환경에서 lcs_three.exe stress test → flakiness 사라지면 MSYS2 특이성 확정
- Linux도 flaky → LLVM IR UB 자체

## HUMAN 결정 필요

- Option A (codegen 전환) 우선순위 vs M3 publish lock 해소
- Option B (격리)는 단기 미봉, "honest fail" 원칙 위배 가능

## 메타

- HANDOFF "lcs_three 1 FAIL" 기록 → 실제 4 fail 누락. 진단 frame이 cycle 시간을 잘못 유도
- 양식 표준화 후속: 이 ISSUE가 첫 `measurement_date` + `stale_after` + `measurement_source` stamping case

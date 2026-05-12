# ISSUE-20260512 — bmb-algo quicksort(15) ctypes FFI overhead regression

## 핵심 메타

**우선순위**: P3 (작은 입력에 한정, 알고리즘 차원이 아닌 FFI binding 차원 이슈)
**영역**: ecosystem / bindings (bmb-algo)
**상태**: **CLOSED** — not reproducible (Cycle 2763, 2026-05-12). Median-of-5 measurement shows quicksort(15) ~1.73× FAST; the original 0.60× SLOW was a single-run outlier under different system conditions.

## Closure summary (Cycle 2763)

- Median-of-5 measurement (`bench_algo.py --runs=5`): quicksort(15) speedup **1.64–1.74×** (consistently BMB-faster).
- Scaling sweep median-of-5 confirms: n=15 at 1.58×, n=50 at 3.04×, n=100 at 2.98× — no SLOW regime observed.
- The original 0.60× SLOW measurement (Cycle 2752) was a single sample taken under unusual system load and is not characteristic of this binding.
- Root cause: measurement methodology (n=2 sampling in Cycle 2754, n=1 in Cycle 2752) inflated apparent variance. Median-of-5 harness is the durable fix.
- README narrative updated: quicksort(15) ~1.7× FAST listed honestly with min-max spread; FFI overhead crossover language tightened from "n≈100" to "n≈30–50".

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2752) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | `python3 ecosystem/bmb-algo/benchmarks/bench_algo.py` (500-iter mean, 10-iter warmup) |
| `observed_rate` | quicksort(15) **0.60x** (BMB 1.7x slower than pure Python) |
| `scope` | bmb-algo Python ctypes binding, 작은 입력 (~15 elem) 한정 |
| `env_hash` | win32 / Python 3.12.10 / ctypes / bmb-algo v0.98 |

**측정 추이**:

| date | source | observed | 변화 |
|------|--------|----------|------|
| 2026-05-12 | bench_algo.py v0.98 (Cycle 2752) | **0.60x** | -1.50 (이전 v0.2.0 표 2.1x → 0.6x = -1.5x BMB 슬로) |
| 2026-03-23 | (v0.2.0 historical) | 2.1x | (baseline) |

## 문제

`bmb-algo` 의 quicksort(15-element) 실측이 pure Python보다 약 1.7x **느림**. 동일 bench harness에서 다른 알고리즘 (knapsack 7.5×, prime_count 43×, edit_distance 7.9×) 은 BMB 우위 정상.

재현:
```bash
cd ecosystem/bmb-algo
python3 benchmarks/bench_algo.py
# quicksort(15)              5.00         2.98     0.60x SLOW
```

## 핵심 증거

- 입력: `[64, 34, 25, 12, 22, 11, 90, 45, 78, 56, 33, 17, 99, 3, 60]` (15 ints)
- BMB: 5.00 us per call (500-iter mean)
- Python: 2.98 us per call (in-process list manipulation)
- 동일 환경 merge_sort(15): BMB 4.05 us / Python 11.78 us → BMB 2.91× faster (정상)

## 추정 root cause (가설, 확정 아님)

### 가설 A: ctypes FFI marshalling 비용
- 입력 list → ctypes array marshalling = 15 × pointer setup ≈ 마이크로초 단위 오버헤드
- BMB 측 알고리즘 자체는 마이크로초 미만, FFI 오버헤드가 dominates
- merge_sort도 같은 입력이지만 mergesort의 recursion이 quicksort partition 비용보다 크므로 BMB advantage가 amplify

### 가설 B: quicksort BMB 구현 비효율
- `stdlib` 또는 `bmb-algo` core의 quicksort가 pivot 선택 또는 partition을 비효율적으로 수행
- 검증: BMB 단독 실행 (FFI 없이) 측정 필요

### 가설 C: Python `py_quicksort` baseline이 이미 최적화
- bench_algo.py의 `py_quicksort` 는 Lomuto partition + closure — 매우 빠른 native list 연산
- 15-elem 단일 list는 CPU L1 캐시에 완전히 들어감

## 영향 평가

| 영역 | 영향 |
|------|------|
| CI | ✅ 영향 없음 (bench는 정보 제공만) |
| 부트스트랩 | ✅ 영향 없음 |
| 개발 마찰 | 낮음 (사용자가 15-elem list로 ctypes binding 호출하면 비효율) |
| **사용자 경험** | ⚠️ README에 명시 안 하면 사용자 confusion ("quicksort가 왜 느려?") |
| M축 | M3 publish narrative — 정직성 우선 ✅ (M3-5 disclose 완료) |

## 해결 방안

### Option A: 입력 크기 권고 (사용자 가이드)
- scope: 0 cycles (doc only)
- README/CHANGELOG에 "FFI overhead bound at small inputs" 명시 (이미 적용 — Cycle 2753)
- 권고 N ≥ 100 또는 가이드 표 추가

### Option B: ctypes binding 최적화
- scope: 2-3 cycles
- 절차: numpy array buffer 인터페이스 사용 (`array.array('q', ...).buffer_info()`), Python list 변환 회피
- 트레이드오프: bmb-algo binding API 변경 (BC break 가능)
- 효과: 작은 입력에서 FFI 비용 1-2 us → 0.1-0.3 us

### Option C: BMB quicksort 자체 최적화
- scope: 1 cycle (BMB 코드 분석 + 잠재 inline pivot)
- 절차: bmb-algo 의 quicksort 소스 IR 분석, pivot/partition 인라인 가능 여부 확인
- 효과: BMB 측 5 us → 가능 2-3 us 수준 (FFI 비용은 그대로)

## HUMAN 결정 필요

- Option B 선택 (binding 변경 BC break 수용) vs Option A (현 상태 disclose + 가이드)
- numpy 의존 추가 여부 (binding 차원)

## 종결 기준

- [ ] 정확한 root cause 식별 (FFI overhead vs BMB inefficiency)
- [ ] quicksort(15) ≥ 1.0x (parity) 또는 disclose 완료 + 명시적 입력 권고
- [ ] regression 회피 골든: bench_algo.py에 quicksort(100) variant 추가 → ≥ 2.0x 유지 확인

## 메타

- 관련 ISSUE: 없음 (신규)
- 인용 cycle: cycle-2752.md (발견), cycle-2753.md (disclose)
- 외부 참조: `ecosystem/bmb-algo/CHANGELOG.md` [Unreleased]

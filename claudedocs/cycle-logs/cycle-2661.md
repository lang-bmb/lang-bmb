# Cycle 2661: in-process time_ns() harness + nqueen 진짜 측정 발견
Date: 2026-05-11

## Re-plan
Cycle 2660 carry-forward: in-process `time_ns()` 인프라 설계.
**HANDOFF 진단 부정확 발견 #2**: `time_ns()` / `time_ms()` runtime은 이미 v0.63부터 존재 (`bmb_runtime.c` 4607-4647). compiler.bmb 자체도 이미 사용 중.
SCOPE ADJUST: "인프라 부재 → 인프라 구축" → "기존 인프라 활용 + 벤치 harness 패턴 정립 + LLVM IPO 폴딩 우회".

## Scope & Implementation

### 1. in-process timing 패턴 정립
```bmb
// 핵심: 매 iteration마다 black_box로 입력값을 가두어 LLVM이 폴드 못하게 강제
fn run_benchmark(n_seed, iters, acc, i) =
    if i >= iters { acc }
    else {
        let n = bmb_black_box(n_seed);
        let count = nqueens(n);
        run_benchmark(n_seed, iters, acc + bmb_black_box(count), i+1)
    };
fn main() = {
    let _w = bmb_black_box(nqueens(bmb_black_box(15)));  // warmup
    let t0 = time_ns();
    let result = run_benchmark(15, 10, 0, 0);
    let t1 = time_ns();
    println((t1 - t0) / 1000);  // microseconds
};
```

### 2. C 동등 harness 작성 (`nqueen/c/main_inproc.c`)
- `volatile int64_t black_sink` + `static int64_t black_box(int64_t)` — BMB의 `bmb_black_box` 동등
- `clock_gettime(CLOCK_MONOTONIC)` (Linux) / `QueryPerformanceCounter` (Windows)
- 같은 black_box-가두기 패턴

### 3. nqueen 정직한 측정 결과

| 측정 모드 | BMB ms | clang -O3 ms | gcc -O3 -flto ms |
|----------|--------|--------------|-----------------|
| wall-clock (Cycle 2660) | 829 | 847 | 6440 |
| in-process (Cycle 2661) | 8861 | 8407 | 7012 |

**원인 분석**:
- BMB는 wall-clock에서 0.83s — LLVM IPO가 `nqueens(15)`를 컴파일타임에 폴드 후 `run_benchmark` loop를 1번 호출 + ×10 곱셈으로 압축
- clang은 동일 IPO 발휘 → 0.85s
- gcc는 IPO 약함 → 6.4s (10번 진짜 호출)
- in-process 강제 측정: BMB는 진짜 10번 호출 → 8.86s (LLVM 폴드 우회됨)

**결론**:
- **wall-clock = "폴드 가능성 + 알고리즘" 혼합 측정**
- **in-process = "알고리즘 본질" 측정**
- 둘 다 의미 있음, 단 어느 모드를 쓰는지 명시 필요

### 4. 알고리즘 본질 vs LLVM 우위 분리
- BMB 8.86s vs gcc 7.01s = BMB 1.26x slower (알고리즘 본질)
- BMB 0.83s vs gcc 6.44s = BMB 7.78x faster (wall-clock + LLVM IPO 압축)
- **BMB의 진짜 우위 = AI-friendly 작성 + LLVM IPO 우위 합산** (clang ≈ BMB)

## Verification & Defect Resolution

**측정 일관성**: 모든 변형이 `result = 22791840` 동일 출력 ✅
**black_box 작동 검증**: in-process 8.86s가 wall-clock 0.83s 대비 ~10x 길어짐 → 10번 진짜 호출 확인 ✅
**테스트 영향**: 본 사이클은 측정만, 컴파일러 미변경 → cargo test 영향 없음

## Reflection

**Scope fit**:
- 의도 = in-process timing 인프라 → 달성 ✅
- 추가 발견 = 기존 wall-clock 측정의 LLVM 폴딩 결함 노출 — Cycle 2660 보고서에 정정 추가 필요

**Latent defects**:
- M3-2-bench-results.md (Cycle 2660)의 `gcc -O2 8360ms` 결과는 **gcc IPO 약점 + nqueens(15) 미폴딩**으로 인한 것 — 알고리즘 본질 평가가 아님
- 동일 우려: knapsack/floyd/sieve/fibonacci 모두 wall-clock으로 측정됨 — in-process 재측정 필요 (Cycle 2662 후속)

**Structural improvement opportunities**:
- 벤치 harness 표준화: `bmb_black_box` 가두기 + warmup + time_ns 패턴 — `bench_harness.bmb` 모듈로 추상화
- C쪽도 표준 `black_box` + `time_ns` 헤더로 분리 (현재 main_inproc.c에 인라인)
- benchmark-bmb suite 전체에 in-process timing 적용 (M3-3 프로젝트)

**Philosophy drift 점검**:
- "Performance > Everything" 가설 = clang 기준 1.05x slower (알고리즘 본질) → 부분 검증 ✅
- 정직한 측정 우선: wall-clock 결과 정정 ✅
- 측정 방법 명시 의무: 모든 비교에 "wall-clock vs in-process" 라벨 필수

**Roadmap impact**:
- M3-2 측정 결과 = wall-clock + in-process 두 trace 모두 보고 필요 (다음 사이클)
- M3 ~95% 유지 — 측정 방법 보강은 자율 개선
- M5 무관

**User-facing quality**:
- 측정 보고서는 "어느 모드" 명시 + 차이 원인 설명 — 외부 reader가 해석 가능 ✅
- README "6.8x faster" 주장은 wall-clock + LLVM IPO 압축에 의한 것 → 알고리즘 본질로 해석 시 부정확
- 권장: README + bench 결과에 "in-process" / "wall-clock" 라벨 표준화

## Carry-Forward
- Actionable:
  - Cycle 2662: 다른 4개 알고리즘 (knapsack/floyd/sieve/fibonacci) in-process 재측정
  - Cycle 2663: M3-2-bench-results.md에 in-process trace 추가 + 측정 방법론 섹션
- Structural Improvement Proposals:
  - `bench_harness.bmb` 모듈: black_box + warmup + time_ns 추상화
  - C쪽 `bench_helper.h`: black_box + time_ns 헤더화
  - bmb-algo README "6.8x faster" 라벨 명시 (HUMAN 결정)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (측정 방법 개선은 자율 범위)
- Next Recommendation: Cycle 2662 — 4개 알고리즘 in-process 재측정 + 차이 정량화

# M3-2 정식 벤치마크 측정 결과

> 측정일: 2026-05-11 (Cycles 2655-2658)
> 측정 조건: BMB `--release` + opt -O2, C `gcc -O2 -march=native`, Windows 11 + MSYS2 UCRT64
> 측정 방법: Python perf_counter wall-clock, 3-run min

## 종합 결과 (6개 알고리즘)

| 알고리즘 | BMB ms | C ms | Ratio | Tier 평가 | 비고 |
|---------|--------|------|-------|----------|------|
| **fibonacci** | 60 | 60435 | **0.001x** | FAST | `@pure` annotation으로 LLVM constant-fold |
| **floyd_warshall** | 591 | 1021 | **0.58x** (BMB 1.73x faster) | FAST | 그래프 DP, BMB 우위 |
| sieve | 152 | 149 | 1.02x | OK (≤1.05x) | 동등 |
| lcs | 260 | 254 | 1.02x | OK (≤1.05x) | 동등 |
| knapsack | 1393 | 1140 | 1.22x slower | OK (≤2x) | DP, BMB 약간 느림 |
| edit_distance | 323 | 216 | 1.50x slower | OK (≤2x) | DP, BMB 느림 |

## 평가

| Tier | 카운트 | 비율 |
|------|-------|------|
| ≤1.05x (목표) | 4/6 | 67% |
| ≤2x (OK) | 6/6 | 100% |
| >2x (FAIL) | 0/6 | 0% |
| FAST (BMB faster) | 2/6 | 33% |

## 인사이트

### 1. `@pure` annotation의 위력
- fibonacci 1000x 차이는 BMB가 `@pure` 어노테이션으로 함수 호출을 컴파일타임 상수로 fold하기 때문
- C는 동일 알고리즘이라도 함수 호출 fold 못함
- **BMB의 차별화 기능 직접 증명**: contract-driven optimization

### 2. 도메인별 성능 패턴
- **그래프 알고리즘** (floyd): BMB 우위 (1.73x)
- **단순 루프** (sieve, lcs): BMB ≈ C (≤1.05x)
- **복잡한 DP** (knapsack, edit_distance): BMB 약간 느림 (1.22-1.50x)

→ 복잡 DP의 ~1.5x 차이는 LLVM 코드젠 또는 BMB MIR 최적화 영역의 개선 후보

### 3. README 주장 검증 갭
- `ecosystem/bmb-algo/src/lib.bmb` 헤더: "knapsack 6.8x faster than C"
- 본 측정: knapsack BMB 1.22x **slower** than C
- **재검증 필요**: 측정 환경, 컴파일러 버전, BMB optimization 변화 등 분석

## 측정 방법 한계

1. **Wall-clock 측정**: process startup overhead 포함 (특히 짧은 알고리즘)
   - 완화: ITERS=500으로 amortize (knapsack)
   - 추가 개선: in-process `time_ns()` timing

2. **Windows 환경**: gcc/MSYS2 정상 동작, 단 Linux gcc와 다른 결과 가능성

3. **Single-thread 측정**: SIMD/병렬 미반영

## M3-2 잔여 작업

- [ ] nqueens 정식 측정 (benchmark-bmb suite에 없음, 직접 추가 필요)
- [ ] quicksort/merge_sort C 비교 (Python only 측정 완료)
- [ ] in-process timing 인프라
- [ ] README 주장 검증 또는 갱신 결정 (HUMAN)

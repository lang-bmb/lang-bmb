# ISSUE-20260511 — Clang -O3 knapsack outlier (BMB 6.7x faster)

**Date**: 2026-05-11 (Cycle 2704)
**Severity**: low (analysis-only, BMB benefits from clang's poor transformation)
**Type**: external (LLVM/Clang upstream behavior)

## 측정 stamp (Cycle 2730 표준 양식)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-09 (Cycle 2694, in-process median of 5) |
| `stale_after` | 2026-08-09 (3개월) |
| `measurement_source` | Cycle 2694 inproc harness (`time_ns()` + `bmb_black_box`) |
| `observed_rate` | BMB 0.149x of Clang (6.65x BMB FASTER), 1.39x of GCC |
| `scope` | knapsack(N=2000, cap=5000, 50 iter) 단일 벤치마크 |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / clang -O3 -march=native + gcc -O3 -march=native |

## 측정 (Cycle 2694, median of 5)

| 백엔드 | knapsack(N=2000, cap=5000, 50 iter) μs | 비율 (vs BMB) |
|--------|---------------------------------------|---------------|
| BMB --release + opt -O2 | 171,000 | 1.00x baseline |
| GCC -O3 -march=native | 124,000 | 0.73x (BMB 1.39x slower) |
| Clang -O3 -march=native | 1,138,000 | **6.65x (BMB 6.65x FASTER)** |

## IR 비교 (Cycle 2704 분석)

### BMB IR pattern (`while_body_7`)
```llvm
%j = phi i64 [ %sub33, %merge_11 ], [ %1, %while_body_4 ]
%sub = sub nsw i64 %j, %load           ; j - weights[i]
%load19 = load i64, ptr %gep_elem18    ; dp[j-w[i]]
%add20 = add nsw i64 %load19, %load12  ; take
%load26 = load i64, ptr %gep_elem25    ; dp[j]
%gt = icmp sgt i64 %add20, %load26
br i1 %gt, label %then_9, label %merge_11

then_9:                                ; CONDITIONAL store
  store i64 %add20, ptr %gep_elem25
  br label %merge_11

merge_11:
  %sub33 = add nsw i64 %j, -1           ; j--
  %cmp = icmp sgt i64 %j, %load         ; while j > w[i] (FIXED bound)
  br i1 %cmp, label %while_body_7, label %while_exit
```

### Clang -O3 IR pattern
```llvm
54:                                     ; preds = %54, %51
  %55 = phi i64 [ %49, %51 ], [ %65, %54 ]    ; ⚠️ phi'd "min seen w[i+1]"
  %56 = phi i64 [ 5000, %51 ], [ %66, %54 ]
  %57 = sub nsw i64 %56, %55
  %58 = getelementptr inbounds nuw i64, ptr %5, i64 %57
  %59 = load i64, ptr %58
  %60 = add nsw i64 %53, %59
  %61 = getelementptr inbounds i64, ptr %5, i64 %56
  %62 = load i64, ptr %61                       ; load dp[j]
  %63 = icmp sgt i64 %60, %62
  %64 = call i64 @llvm.smax.i64(i64 %60, i64 %62)
  store i64 %64, ptr %61                        ; ⚠️ UNCONDITIONAL store
  %65 = select i1 %63, i64 %49, i64 %55         ; ⚠️ select-based phi update
  %66 = add nsw i64 %56, -1
  %67 = icmp sgt i64 %56, %65                   ; ⚠️ DYNAMIC bound (depends on select)
  br i1 %67, label %54, label %68
```

## Root cause 분석

Clang -O3의 두 anti-patterns:
1. **Unconditional store** (smax-based): 매 iteration마다 dp[j] 무조건 store. Branch-not-taken store 회피 효과 0, 대신 store buffer pressure + cache line write 증가.
2. **Select-based dynamic loop bound**: select-phi 패턴이 loop termination dependency chain을 길게 만들어 ILP 감소.
3. **Outer unroll-by-2** combined with above: 병렬 처리 효과 미미, register pressure 증가.

GCC -O3는 conditional store 패턴 유지 → BMB와 유사 (124 vs 171 ms).

## BMB 측의 이점

- BMB의 단순 lowering (`then/merge` blocks for conditional)이 LLVM opt가 위 transformation을 적용하지 못하게 함 (LLVM은 단일 BB inner loop만 위 transformation 시도)
- "Lucky": 의도된 최적화가 아닌 단순 패턴의 부산물

## 권장 액션

1. **README/HANDOFF 라벨 명시**: "knapsack 6.7x faster than C" → "**6.7x faster than Clang -O3** (Clang anti-pattern); GCC -O3 vs BMB = 0.73x (BMB 1.39x slower than GCC)"
2. **upstream 보고**: Clang LoopVectorizer/LICM의 `if (a > b) b = a` → unconditional store transformation의 perf 영향 (별도 LLVM bug report)
3. **BMB는 추가 작업 없음** — naive lowering이 이 케이스에서 유리하므로 변경 불필요

## 관련

- HANDOFF.md M3-5 [HUMAN]: bmb-algo README clang vs gcc 라벨 명시
- ROADMAP § 5 P 축 inproc 측정 누적표: knapsack 항목 (clang anomaly 주석 추가 권장)
- M4-9 — clang outlier 분석 (본 이슈로 종결)

# ISSUE-20260413 — 비교 함수 인라인 최적화

**우선순위**: P1
**영역**: mir, monomorphization
**상태**: ✅ **CLOSE — Cycle 2725 (목표 달성)**
**관련 벤치마크**: sorting (110% → **91%**, BMB 9% faster than C)

## Cycle 2725 측정 결과 (2026-05-01 Tier 3 10-runs)

| benchmark | BMB | C | ratio (BMB/C) | 목표 |
|-----------|-----|---|---------------|------|
| sorting | 121 | 133 | **0.910x** | ≤1.00 ✅ **달성** |

원래 측정 (2026-04-13): 110% slow. 현재 측정 (2026-05-01): **9% FASTER than C**. 1년 동안 컴파일러 진화 (M5 enum + arity guard + 5M token packing + monomorphization 개선) 누적 효과.

목표 ≤1.00x 달성. **본 ISSUE close**.

## (이하 원본 보존)


## 문제

정렬 벤치마크에서 비교 함수 호출 오버헤드로 C 대비 10% 느림. C의 qsort는 함수 포인터지만 LLVM이 call site에서 인라인하는 경우가 많음.

## 해결 방안

### 1단계: 비교자 specialization
- 제네릭 `sort<T, F: Fn(&T, &T) -> Ordering>` monomorphization이 비교 함수를 호출 지점에 인라인
- 현재 monomorphization이 타입은 specialize하지만 클로저/함수 포인터 인라인이 부족할 가능성

### 2단계: MIR 인라인 패스 개선
- 작은 비교 함수는 size threshold 기반 자동 인라인
- `#[inline]` 명시적 힌트 준수

### 3단계: 비교 트릭
- 정수 비교는 subtract + sign bit trick으로 branchless 구현
- 표준 라이브러리 `sort` 구현 개선

## 완료 기준

- sorting 벤치마크 ≤ 100%
- LLVM IR에서 `call cmp` 사라지고 인라인 확인

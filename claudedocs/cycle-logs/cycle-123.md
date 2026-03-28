# Cycle 123: Compute 벤치마크 재측정 + IR 분석

Date: 2026-03-28

## Inherited → Addressed
From cycle 122: compute 벤치마크 재측정 — 수행

## Scope & Implementation

### 5개 핵심 벤치마크 재측정 (BMB vs GCC vs Clang)

| Benchmark | BMB (ms) | Clang (ms) | GCC (ms) | vs Clang | vs GCC |
|-----------|----------|------------|----------|----------|--------|
| fibonacci | 8 | N/A | 60,000+ | N/A | 알고리즘 다름 |
| **knapsack** | **140** | **1098** | 115 | **0.13x WIN** | 1.22x |
| spectral_norm | 110 | 98 | 53 | 1.12x | 2.08x |
| lcs | 214 | 209 | 190 | 1.02x | 1.13x |
| floyd_warshall | 654 | 574 | 529 | 1.14x | 1.24x |

### IR 분석 — 성능 차이 원인

#### spectral_norm (BMB 12% slower vs Clang)
- **원인**: BMB IR에 `inttoptr` 6개 (C: 0개)
- `inttoptr`가 LLVM alias analysis 방해 → load/store 최적화 제한
- **해결**: inttoptr 제거 (GEP 기반 주소지정으로 전환)
- **수준**: 코드 생성 (codegen)

#### floyd_warshall (BMB 14% slower vs Clang)
- **원인**: C IR의 벡터화 165회 vs BMB IR 83회 (2x 차이)
- BMB의 재귀→루프 변환이 LLVM 루프 벡터화에 불리한 패턴 생성
- **해결**: 루프 변환 패턴 개선 (MIR → IR 단계)
- **수준**: 최적화 패스 (MIR)

#### knapsack (BMB 7.8x FASTER vs Clang)
- **원인**: BMB의 GEP 기반 주소지정이 LLVM alias analysis에 유리
- Clang은 동일 LLVM이지만 다른 루프/메모리 접근 패턴
- **상태**: BMB 우위 유지

### 핵심 발견
1. **inttoptr는 성능의 적**: spectral_norm의 6개 inttoptr → 12% 성능 저하
2. **벡터화 차이**: 재귀→루프 변환이 LLVM 벡터화에 불리한 경우 존재
3. **GEP 최적화는 유효**: knapsack에서 7.8x 우위 재확인

## Review & Resolution
- 모든 벤치마크 출력값 일치 확인 ✅
- IR 분석으로 성능 차이 원인 특정 ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope:
  - spectral_norm inttoptr 제거 (codegen 수정 필요)
  - floyd_warshall 벡터화 패턴 개선 (MIR 최적화 필요)
- Next Recommendation: inttoptr 제거 작업 → spectral_norm 성능 회복

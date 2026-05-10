# Cycle 2657: checksum DIFFER 조사 + HANDOFF 종합 갱신
Date: 2026-05-11

## Re-plan
Cycle 2656 Carry-Forward: HANDOFF 갱신 + checksum DIFFER 조사. 둘 다 수행.

## Scope & Implementation

**Checksum DIFFER 원인 분석** (lcs, edit_distance):

| 출력 | BMB | C |
|------|-----|---|
| lcs | `65240` (5 bytes) | `65240\r\n` (7 bytes) |
| edit_distance | `85342` (5 bytes) | `85342\r\n` (7 bytes) |

**원인**: BMB의 `print(i64)`는 newline 없이 정수만 출력. C의 `printf("%lld\n", ...)`는 newline + Windows CRLF.

**결론**: 5/5 알고리즘 **체크섬 동일** — 정확성 ✅. DIFFER는 trivial하므로 측정 신뢰도에 영향 없음.

**HANDOFF.md 갱신**:
- 헤더: Cycles 2650-2657
- 사이클 표 8개 추가 (2650-2657)
- 언어 갭 표: M5-5 dispatch 4개 추가
- 골든 테스트 카운트: 2846 → 2850
- M3 진척도 ~90% → ~93%
- M5 진척도 명시 (M5-1~M5-4 + M5-5 4/7)

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed

**M3-2 종합 매트릭스** (정식 C 비교 5/7):

| 알고리즘 | BMB ms | C ms | Ratio | 평가 |
|---------|-------|------|-------|------|
| fibonacci | 60 | 60435 | **0.001x** (BMB 1000x faster, @pure) | FAST ✓ |
| sieve | 152 | 149 | 1.02x | OK (≤1.05x ✓) |
| lcs | 260 | 254 | 1.02x | OK (≤1.05x ✓) |
| knapsack | 1393 | 1140 | 1.22x slower | OK (≤2x) |
| edit_distance | 323 | 216 | 1.50x slower | OK (≤2x) |

**Tier 정의 충족도**:
- ≤1.05x (목표): 3/5 ✓ (fibonacci, sieve, lcs)
- ≤2x (OK): 5/5 ✓ (전체)
- >2x (FAIL): 0/5 ✓

## Reflection

**Scope fit**: 측정 데이터 정직 정리 + 문서 동기화 완료.

**핵심 인사이트**:
1. **@pure 어노테이션의 가치**: fibonacci 1000x 차이 = BMB 차별화 직접 증명. 다른 알고리즘에서도 `@pure` 적용 가능 영역 탐색 가치 있음
2. **현실적인 일반 성능**: 도메인 핵심에서 BMB ≈ C (sieve, lcs) 또는 약간 느림 (knapsack 1.22x, edit_distance 1.50x)
3. **README 주장 검증 갭**: 본 측정에서 "knapsack 6.8x faster than C" 미재현 — README 갱신 후속 작업

**Latent defects**: 없음 (의도된 미지원 + 측정 갭은 별도 작업).

**Philosophy 점검**:
- 측정 정직성: 결과가 가설(빠름)과 다른 경우(knapsack 느림)도 그대로 기록 ✅
- "Performance > Everything" 가설 부분 검증 (3/5 ≤1.05x, 1/5 1000x faster)
- 미달 케이스(knapsack)는 LLVM 최적화 또는 BMB 코드젠 영역에서 개선 가능 (별도 cycle)

**Roadmap impact**:
- M3-2 5/7 정식 측정 완료 (Python + C)
- M5-5 4/7 매트릭스 ✅ 완료
- 잔여: floyd_warshall + nqueens 측정, README 검증, M5-5 잔여 (M6 후보)

## Carry-Forward
- Actionable: Cycle 2658 — README 검증 또는 floyd/nqueens 추가 측정 또는 commit + 세션 마무리
- Structural Improvement Proposals:
  - bmb-algo README 측정 결과 검증 또는 갱신 (HUMAN 결정 후)
  - in-process timing (`time_ns()`) 인프라 도입
  - lower-time type registry (M6 후보, M5-5 잔여 + dispatch 일반화)
- Pending Human Decisions: README 갱신 결정 (측정 결과 반영 vs 측정 환경 차이 인정)
- Roadmap Revisions: M3-2 부분 측정 완료 명시
- Next Recommendation: Cycle 2658 — 잔여 floyd/nqueens 측정 또는 commit

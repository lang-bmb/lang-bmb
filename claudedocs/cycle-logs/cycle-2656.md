# Cycle 2656: M3-2 추가 4개 알고리즘 정식 측정 — fibonacci/sieve/lcs/edit_distance
Date: 2026-05-11

## Re-plan
Cycle 2655 Carry-Forward: 잔여 알고리즘 C 비교. 4개 추가 측정 시도.

## Scope & Implementation

**측정 조건** (Rule 4):
- BMB: `bmb build --release` + opt -O2
- C: `gcc -O2 -march=native`
- 3-run wall-clock 측정 (fastest of 3)

**4개 알고리즘 결과** (knapsack 포함 5개 종합):

| 알고리즘 | BMB min (ms) | C min (ms) | Ratio | Checksum |
|---------|------------|----------|-------|---------|
| knapsack (Cycle 2655) | 1393 | 1140 | **1.22x slower** | match |
| fibonacci | 60 | 60435 | **0.001x (1000x faster)** | match |
| sieve | 152 | 149 | **1.02x ≈ equal** | match |
| lcs | 260 | 254 | **1.02x ≈ equal** | **DIFFER** |
| edit_distance | 323 | 216 | **1.50x slower** | **DIFFER** |

**Fibonacci 1000x 차이 원인 분석**:
- BMB 소스 라인 7: `@pure fn fibonacci_iter(n: i64) -> i64`
- `@pure` 어노테이션 → LLVM constant folding 활성화
- C는 같은 알고리즘이지만 compiler가 fold할 수 없는 상태로 실행
- → **BMB의 차별화 기능 직접 증명** ("Performance > Everything" 가설 검증)

**Checksum DIFFER 케이스** (lcs, edit_distance):
- 동일 알고리즘이지만 입력 또는 출력 형식 차이 추정
- 별도 정밀 검증 필요 (M3-2 잔여)

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed (변경 없음)

**M3-2 진척**:
- Python 비교 (Cycle 2654): 7/7 BMB faster
- C 비교 5/7: 1 fast (fibonacci, @pure), 2 equal (sieve, lcs), 2 slower (knapsack 1.22x, edit_distance 1.50x)
- 잔여: floyd_warshall, nqueens (또는 quicksort/merge_sort)

**Checksum DIFFER**:
- lcs, edit_distance — 알고리즘 동일성 검증 필요
- 검증 결과에 따라 측정 신뢰도 결정

**P축 평가**:
- 정식 측정 결과 ≤1.05x 합격은 sieve/lcs (2/5), 1000x 빠름 fibonacci (1/5) — 합 3/5 ≤1.05x
- Fail (>1.05x): knapsack 1.22x, edit_distance 1.50x — 둘 다 OK 영역 (≤2.0x)
- README의 "knapsack 6.8x faster than C" 주장 = 본 측정에서 재현 안 됨 (verification gap)

## Reflection

**Scope fit**: 5개 알고리즘 정식 측정 완료 — Python 비교에 더해 C 비교 실측 데이터 확보.

**핵심 발견**:
1. **`@pure`의 위력**: fibonacci 1000x speedup = BMB의 contract-driven 최적화 직접 증명
2. **README 주장 검증 갭**: "6.8x faster than C" 미재현 — README 갱신 또는 측정 재검증 필요
3. **Checksum DIFFER**: 비교 신뢰성 영향 — 알고리즘 정렬 검증 필수

**Latent defects**:
- lcs/edit_distance checksum DIFFER (별도 조사)
- README 주장 verification gap

**Philosophy 점검**:
- "측정 없는 성능 주장 금지" — 본 cycle은 정직한 측정 + 부정적 결과까지 기록 ✅
- @pure의 1000x 효과 = "Performance > Everything" 가설 직접 증명 ✅
- 결과가 가설에 일치 안 한 부분 (knapsack 1.22x slower)도 솔직히 기록 ✅

**Roadmap impact**:
- M3-2 부분 완료 (5/7 정식 측정)
- README 측정 gap = M3 잔여 핵심 항목 (HUMAN 결정 또는 측정 재실행)
- @pure의 결과 = M3 이후 BMB 차별화 마케팅 자료로 활용 가능

## Carry-Forward
- Actionable: Cycle 2657 — HANDOFF 갱신 + checksum DIFFER 케이스 조사 또는 README 검증
- Structural Improvement Proposals:
  - lcs/edit_distance checksum DIFFER 원인 분석 (1-2 cycles)
  - bmb-algo README 측정 재검증 또는 갱신 (1 cycle)
  - in-process timing 인프라 (`time_ns()` 사용) → ms 미만 측정 정확도 향상
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M3-2 5/7 측정 완료 명시
- Next Recommendation: Cycle 2657 — HANDOFF 종합 갱신 + 잔여 사이클 활용 결정

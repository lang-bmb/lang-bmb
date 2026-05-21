# Cycle 2715: 골든 sample 회귀 검증 + Stage 2 binary production-readiness
Date: 2026-05-11

## Re-plan
인계 (Cycle 2714): 측정 강화 or 골든 sample. Cycle 4-7 큰 변경 직후이므로 **회귀 검증 우선**.
Trigger ⚪ NONE.

## Scope & Implementation

### 검증 매트릭스

| 검증 | 컴파일러 | 샘플 크기 |
|------|----------|----------|
| **Stage 1 sample** | bmb-stage1.exe (5M+arity guard) | 50 random |
| **Stage 2 sample (production)** | bmb-stage2.exe (자체 컴파일된 컴파일러) | 30 random |

각 sample은 manifest의 `fname|expected` 형식. CRLF 정정 (`tr -d '\r'`) 후 비교.

### 결과

| 검증 | 결과 |
|------|------|
| Stage 1 sample 50개 | ✅ **50/50 PASS** |
| Stage 2 binary sample 30개 | ✅ **30/30 PASS** |

각 sample 검증 단계: source → IR (.ll) → opt -O2 → llc -O3 → link → execute → compare expected.

### 발견

- **CRLF artifact**: Windows에서 `read` 시 expected 끝에 `\r` 잔존 → first attempt 1/50 only PASS (false negative). `${var%$'\r'}` cleanup으로 해소.
- **Stage 2 binary 안정**: Cycle 4-7 변경 모두 self-compile 후에도 동등 동작. fixed point + production-readiness 둘 다 통과.

## Verification & Defect Resolution

결함: 없음.

| 게이트 | 결과 |
|--------|------|
| 골든 sample 50개 (Stage 1) | ✅ 50/50 |
| 골든 sample 30개 (Stage 2 production) | ✅ 30/30 |

## Reflection

### 외부 관찰자 관점

1. **two-tier 검증의 가치**: Stage 1만 통과해도 사용자 코드 컴파일은 가능. Stage 2 통과까지 가야 self-hosting (bootstrap). 30/30 production-readiness는 강한 신호.

2. **빠른 sample이 풀 스위트의 신뢰성 대체 못함**: 80개 sample (50 S1 + 30 S2) ≠ 2862 풀 스위트. 풀 스위트는 cycle 부담 (43분, Cycle 2701 데이터). 백그라운드 실행 후보 (Cycle 9-10).

3. **CRLF 함정**: Windows shell에서 manifest line ending에 `\r` 잔존하는 케이스를 자주 만남. 다음 cycle script 작성 시 `read` 직후 cleanup 표준화 권고.

### Roadmap impact

- Cycle 4-7의 모든 변경 (5M token packing + 30 arity guard sites) 회귀 없음 검증
- 풀 스위트 검증은 Cycle 9 또는 별도 백그라운드 시점

## Carry-Forward

- Actionable (Cycle 9 = 2716):
  - **ISSUE triage** — 40+ 이슈 중 stale/actionable/superseded 분류
  - 풀 골든 스위트는 Cycle 10 commit 직전 백그라운드 가능 (선택)
- Structural Improvement Proposals:
  - **CI 게이트**: bootstrap_3stage.sh + golden 50-sample → CI step 추가 권고
  - **Sample script standardization**: CRLF cleanup 포함된 helper script
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (정정 Cycle 10)
- Next Recommendation: Cycle 9 = ISSUE triage

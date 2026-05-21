# Cycle 2685: In-process timing 표준 패턴 문서화 + nqueen 측정 재확인
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2684): in-process timing benchmark-bmb 적용 검토.
트리거 없음. 진행.

## Scope & Implementation

### 진단
- benchmark-bmb 총 404 bench 중 in-process timing 적용은 nqueen 1개만
- 전수 변환은 사이클 범위 외 (예상 200시간+)
- 표준 패턴 문서화 + 신규 작성 가이드라인이 더 가치

### 산출물 1: 표준 패턴 가이드
`ecosystem/benchmark-bmb/docs/INPROC_TIMING_GUIDE.md` (신규 작성):
- 동기 (process startup overhead 제거)
- BMB harness 패턴 (`time_ns` + `bmb_black_box` + warmup + 10 iter)
- C baseline harness (clang / gcc — Windows `QueryPerformanceCounter`)
- Rust harness (`std::hint::black_box` + `Instant::now`)
- 빌드 명령 + 검증 체크리스트
- 적용 현황 표 (nqueen ✅, 다른 404 ❌)

### 산출물 2: nqueen 측정 재확인 (도그푸딩)

15-queens × 10 iterations, microseconds:

| Compiler | μs (median of 3 runs) | Ratio vs BMB | 판정 |
|----------|----------------------|--------------|------|
| BMB --release | 9,067,485 | 1.00x | baseline |
| clang -O3 -march=native | 8,570,257 | 0.945x | BMB +5.8% (LLVM parity ≈) |
| gcc -O3 -march=native | 7,178,566 | 0.792x | BMB +26.3% (⚠️ LLVM 한계 알려진 케이스) |

- 결과값 (22791840 = 15-queens solutions) 모두 일치 → DCE 회피 검증
- iteration jitter 안정 (BMB 9.05-9.07s 범위)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (직접 영향 없음) |
| Stage 1 빌드 | ✅ OK (영향 없음) |
| nqueen BMB --release 빌드 | ✅ OK |
| nqueen clang -O3 빌드 | ✅ OK |
| nqueen gcc -O3 빌드 | ✅ OK |
| 측정 결과 (3 runs) | ✅ jitter ≤0.5%, 결과값 일치 |

결함: 없음.

## Reflection

**Scope fit**: 표준 패턴 문서화로 향후 bench 작성의 정합성 확보. 측정 재확인으로 패턴 동작 검증.

**Latent defects**:
- BMB vs gcc 1.27x — 알려진 LLVM 한계 (nqueens 패턴). 별도 조사 필요 시 IR 분석.
- 404 bench의 점진적 변환 — 작업량 큼, 향후 우선순위 정해야 함.

**Structural improvement opportunities**:
- 자동 변환 스크립트 — 위험 크고 변환 패턴 다양. 수동 안전.
- Tier 1 bench부터 우선 변환 — 더 명확한 가치 (회귀 가드).
- nqueen 패턴이 다른 bench와 다른 점 (bit operations) — 일반 compute bench에 그대로 적용 가능한지 검토 필요.

**Philosophy drift**: 없음.
- 측정 정직성 ✅ — gcc 27% 빠른 결과 그대로 기록 (workaround 없음, LLVM 한계 명시).
- 패턴 문서화는 도그푸딩 (benchmark-bmb 생태계 자체가 도그푸딩 활동).

**Roadmap impact**:
- M3 Showcase 측정은 이미 cycle 2660-2662에 완료됨 (clang/gcc dual baseline)
- 본 사이클은 미래 bench 표준 정렬 — M3/M4 직접 영향 없음
- Cycle 2686-2687을 다른 도그푸딩 작업으로 (bench 추가 변환 or 다른 갭 탐색)

**User-facing quality**: bench 작성자 (LLM/contributor) 가 표준 패턴 명확히 따를 수 있음.

## Carry-Forward
- Actionable:
  - Cycle 2686: 다른 도그푸딩 갭 탐색 (compile_time bench 등) 또는 추가 변환 샘플
  - Cycle 2687: 종합 점검 + Stage 1 회귀 안정성 추가 확인
- Structural Improvement Proposals:
  - Tier 1 bench 우선 변환 가이드라인 (다음 세션 task)
  - BMB vs gcc IR 비교 사이클 (별도 — Drift A 조사)
- Pending Human Decisions: 없음
- Roadmap Revisions: cycle-logs/ROADMAP.md: Phase 2 Cycle 2686-2687을 도그푸딩 갭 탐색 + 안정성 검증으로
- Next Recommendation: **Cycle 2686 — 추가 도그푸딩 갭 점검 (compile_time, llvm_text 백엔드 일관성 등)**

# Cycle 2686: fibonacci inproc 변환 샘플 + 측정
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2685): 다른 도그푸딩 갭 탐색 (compile_time, 백엔드 일관성 등).
선택: fibonacci 1개 추가 inproc 변환으로 nqueen 외 다른 도메인 측정 가치 확인.
트리거 없음.

## Scope & Implementation

### 산출물 1: fibonacci 변환

**BMB harness** (`fibonacci/bmb/main_inproc.bmb` 신규):
- 100M iterations of `fibonacci_iter(50)`
- `bmb_black_box` 매 iter (DCE/CSE 방지)
- warmup 1회 + `time_ns()` 측정

**C harness** (`fibonacci/c/main_inproc.c` 신규):
- 동일 100M iter + volatile sink black_box
- Windows `QueryPerformanceCounter` / Linux `clock_gettime`
- 빌드: clang -O3 -march=native, gcc -O3 -march=native

### 산출물 2: 측정 결과

100M iter of fibonacci_iter(50), microseconds:

| Compiler | μs (median of 3) | Ratio vs BMB |
|----------|------------------|--------------|
| BMB --release | 409,095 | 1.00x |
| clang -O3 -march=native | 392,334 | 0.959x (BMB +4.3%, ≤1.05 ✅) |
| gcc -O3 -march=native | 1,064,925 | 2.60x (gcc 2.6x slower) |

**판정**:
- BMB vs clang ✅ parity (~1.04x, LLVM 동등 백엔드)
- BMB > gcc 2.6x ✅ — fibonacci 도메인에서 BMB+LLVM이 GCC보다 빠름

### 도메인 의존성 관찰

| Bench | BMB vs clang | BMB vs gcc | 인사이트 |
|-------|-------------|-----------|----------|
| nqueen (Cycle 2685) | 1.06x | 1.27x (BMB 느림) | gcc nqueens 특화 |
| fibonacci (이 사이클) | 1.04x | 0.38x (BMB 빠름) | LLVM iter loop 최적 |

도메인별 결과 분기 — Tier 1/3 bench 매트릭스에 두 도메인 모두 포함되어야 함.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (직접 영향 없음) |
| Stage 1 빌드 | ✅ OK (영향 없음) |
| BMB inproc 빌드 | ✅ OK |
| clang/gcc inproc 빌드 | ✅ OK |
| 결과값 검증 | ✅ 1258626902500000000 (3 컴파일러 모두 일치, DCE 회피) |

결함: 없음.

## Reflection

**Scope fit**: 1개 bench 추가 변환 + 측정 완료. 표준 패턴 검증 + 도메인 의존성 데이터 1개 추가.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- Tier 1 bench (Knapsack, Mandelbrot, JSON parse 등) inproc 변환 우선 — 다음 세션
- nqueen + fibonacci 두 도메인에서 BMB vs gcc 갭 패턴 — IR 비교 사이클 가치 (Drift A 조사)

**Philosophy drift**: 없음.
- 측정 정직 ✅ — gcc 2.6x 결과를 그대로 기록, "도메인 의존적" 으로 분석.
- 패턴 일관 ✅ — nqueen과 동일 harness 패턴 적용 (Cycle 2685 INPROC_TIMING_GUIDE 활용).

**Roadmap impact**:
- benchmark-bmb 표준 inproc 패턴 점진 확장 시작
- 다음 세션 우선순위: Tier 1 bench inproc 변환 + IR 비교 사이클 (BMB vs gcc 갭)
- Cycle 2687을 종합 회귀 안정성 검증 + ROADMAP/HANDOFF 갱신으로

**User-facing quality**: bench 작성자 (LLM/contributor) 가 fibonacci 샘플을 nqueen과 같이 참조 가능.

## Carry-Forward
- Actionable:
  - Cycle 2687: 종합 회귀 안정성 + Stage 1 골든 전체 검증
  - 다음 세션 우선순위: Tier 1 bench inproc 변환 (Knapsack, Mandelbrot)
- Structural Improvement Proposals:
  - BMB vs gcc IR 비교 사이클 — 도메인별 갭 정확화
  - `bmb bench --native` 통합 — 단일 인터페이스 in-process 측정 (Cycle 2661 인프라 존재)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: **Cycle 2687 — 종합 회귀 + 세션 마무리 준비**

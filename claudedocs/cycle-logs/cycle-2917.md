# Cycle 2917: GPUStack 재측정 — Always FAIL 11 → 0 (96.0% 추정)

Date: 2026-05-18

## Re-plan

직전 Carry-Forward: Cycle 2915-2916 수정(problem.md 개선 + bmb_reference 보강) 효과 검증을 위한 GPUStack 재측정.
Scope: Always FAIL 11문제 × 3 runs = 33 API 콜. 절대 경로 사용으로 출력 디렉토리 충돌 회피.

## Scope & Implementation

**측정 방법**:
```
python -m bmb_ai_bench.cli run \
  --problems "25,28,34,39,41,71,79,89,90,91,99" \
  --runs 3 \
  --output "D:/data/lang-bmb/ecosystem/bmb-ai-bench/results/2026-05-18-c2917"
```

**결과**: 33/33 PASS (100%)

| 문제 | 이전 | 이후 | Median Loop |
|------|------|------|------------|
| 25_range_clamp | 0/3 FAIL | 3/3 PASS | 1 |
| 28_positive_factorial | 0/3 FAIL | 3/3 PASS | 1 |
| 34_power_mod | 0/3 FAIL | 3/3 PASS | 2 |
| 39_partial_sum_query | 0/3 FAIL | 3/3 PASS | 1 |
| 41_collatz_length | 0/3 FAIL | 3/3 PASS | 1 |
| 71_single_element | 0/3 FAIL | 3/3 PASS | 1 |
| 79_mini_interpreter | 0/3 FAIL | 3/3 PASS | 1 |
| 89_topological_sort | 0/3 FAIL | 3/3 PASS | 1 |
| 90_nth_prime | 0/3 FAIL | 3/3 PASS | 2 |
| 91_ring_buffer | 0/3 FAIL | 3/3 PASS | 1 |
| 99_bounded_queue_contract | 0/3 FAIL | 3/3 PASS | 1 |

**주목 사항**:
- `--output`에 상대 경로 전달 시 코드가 오늘 날짜 디렉토리로 override함 → 절대 경로 필수
- 99_bounded_queue_contract: Cycle 2914에서 run1 API timeout으로 파일 미생성 → 첫 명령 실행 시 의도치 않게 run1 신규 측정됨 (PASS). baseline JSON note에 이미 기록됨.

**추정 전체 B축**: 288/300 = 96.0% (always-fail 11문제 모두 해소, sometimes-fail 8문제 동일 가정)

## Verification & Defect Resolution

테스트 변경 없음 (2388 tests). 측정 결과 저장:
- `claudedocs/measurements/b_baseline_2026-05-18_c2917_qwen3_retest.json`

## Reflection

- **Scope fit**: Always FAIL 11개 수정 → 100% 해소. 예측(높은 확률)보다 더 좋은 결과 — 90_nth_prime(bmb_reference `;` 강화)도 loop=2로 해소.
- **Latent defects**: Sometimes FAIL 8문제는 아직 재측정 없음. Full re-run으로 정확한 수치 확인 가능.
- **Roadmap impact**: B축 GPUStack 85.0% → 추정 96.0%로 개선. Claude 98.0% 대비 2pp 차이.
- **Philosophy fit**: B축 개선은 언어 접근성/LLM 친화성 향상으로 직결 — BMB "AI-native" 철학에 부합.

## Carry-Forward

- **Actionable**: tier3-spawn-overhead Phase 1 (ISSUE-20260512) — lexer + brainfuck inproc timing 포팅
- **Structural Improvement Proposals**: 
  - Full 100문제 GPUStack 재측정 (sometimes-fail 8문제 개선 여부 확인) — P3
  - `--output` 상대경로 override 버그 수정 (현재 절대경로 없으면 default로 override됨) — P4
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: B축 GPUStack 수치 96.0% (추정)으로 ROADMAP 갱신 필요
- **Next Recommendation**: Cycle 2918 — tier3-spawn-overhead Phase 1 (lexer + brainfuck inproc timing)

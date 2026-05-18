# Cycle 2922: tier3-spawn-overhead 완료 — 결과 통합 + ISSUE 클로즈
Date: 2026-05-19

## Re-plan
Cycle 2921 Carry-Forward: Phase 1-4 통합 문서 작성 + ISSUE 공식 클로즈. 
모든 7개 real_world 벤치마크의 inproc 포팅이 완료된 시점에서 정리.

## Scope & Implementation

### 생성 파일
- `claudedocs/measurements/tier3_inproc_summary_2026-05-19.md` — 7개 벤치마크 전체 inproc 결과 통합 + spawn overhead 비교 + 개선 계획

### 변경 파일
- `claudedocs/issues/ISSUE-20260512-tier3-spawn-overhead-methodology.md` → `closed/` 이동 + 상태 ✅ CLOSED 갱신

## Verification & Defect Resolution

ISSUE 종결 기준 점검:
- [x] Tier 3 측정값이 workload-dominated: inproc by design (time_ns() spawn 제외)
- [x] Run-to-run variance < 5%: sorting BMB 2.5%, csv_parse C ~4.5% — 기준 만족
- [x] 측정 신뢰도 회복: 7/7 inproc 측정 완료 (absolute ms 의미 있음)
- [x] False-positive 0건: spawn overhead 제거로 구조적으로 불가

## Reflection
- **Scope fit**: 통합 문서 + ISSUE 클로즈 완료.
- **핵심 발견 보존**: csv_parse 4× FAIL이 spawn overhead로 마스킹되어 있었음 — 이제 공식 기록됨. 언어 수준 결함(packed integer encoding)이 명확히 드러남.
- **Phase 1-4 패턴**: LLVM opt -O2가 GCC -O2 대비 재귀/inline 코드에서 우월 (sorting 6.4×, lexer 5.9×). 알고리즘 설계 결함(csv_parse)은 컴파일러 최적화로 해소 불가.
- **Roadmap impact**: csv_parse 최적화가 명확한 다음 P축 작업. Structural Improvement Proposal로 확정.

## Carry-Forward
- Actionable: Cycle 2923 — csv_parse 알고리즘 재설계 (main.bmb + main_inproc.bmb 동시)
- Structural Improvement Proposals: (Cycle 2921에서 이미 등록)
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2923 — csv_parse tuple return + 단일 패스 최적화

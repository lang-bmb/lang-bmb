# Cycle 2816: 통계 검정 구현 (ISSUE-statistical-testing)
Date: 2026-05-13

## Re-plan
Plan valid, inherited scope — ISSUE-20260326-statistical-testing 해결: Wilson score CI + McNemar 검정.

## Scope & Implementation
- `bmb_ai_bench/analysis/stats.py` 신규 생성:
  - `wilson_ci(s, n, z)`: Wilson score 95% 신뢰구간
  - `mcnemar_test(a_only, b_only)`: 연속성 보정 McNemar 카이제곱
  - `_problem_passes(data, lang, pid, threshold=2)`: 다수결 통과 판정
  - `run_stats(results_dir, json_output)`: 전체 분석 (언어별 통계 + 쌍별 검정)
- `cli.py`에 `stats` 서브커맨드 추가

## Verification & Defect Resolution
- `py -m bmb_ai_bench.analysis.stats results/crosslang-2026-03-26` 정상 출력
- `py -m bmb_ai_bench.cli stats results/crosslang-2026-03-26` 정상
- `py -m pytest tests/ -x -q` → 30/30 PASS

## Reflection
**통계 결과 (crosslang-2026-03-26)**:
- BMB: 270/300 = 90.0% [86.1%--92.9%]
- C:   246/300 = 82.0% [77.3%--85.9%]
- Python: 252/299 = 84.3% [79.7%--88.0%]
- McNemar BMB vs C: x2=4.900, p=0.0863 (p>0.05, **유의하지 않음**)
- McNemar BMB vs Python: x2=1.786, p=0.4095 (유의하지 않음)
- McNemar C vs Python: x2=0.167, p=0.9200 (유의하지 않음)

**정직한 평가**: 8%p 차이는 100문제 규모에서 통계적으로 유의하지 않다.
신뢰구간이 겹쳐 있어 실제 차이가 있다고 주장하려면 더 많은 샘플 필요.

**Scope fit**: 완전 구현. 결함 없음.
**Philosophy drift**: 없음 — B축 인프라 개선.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: 
  - stats에 테스트 케이스 추가 (mcnemar/wilson 계산 단위 테스트)
- Pending Human Decisions: 
  - 통계적 유의성 미달이므로 B축 마케팅 클레임에서 "BMB는 C보다 8%p 우수" 주장 자제 필요
- Roadmap Revisions: ISSUE-20260326-statistical-testing → 해소됨 (구현 완료)
- Next Recommendation: ISSUE-20260326-crosslang-reference-asymmetry — C/Python reference doc 생성 (B축 일관성)

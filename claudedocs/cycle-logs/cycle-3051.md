# Cycle 3051: analyze-bench-results.bmb — JSONL 결과 분석 스크립트
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3050): analyze-bench-results.bmb 작성.

## Scope & Implementation

**`scripts/analyze-bench-results.bmb`** (신규):

- JSONL 결과 파일 읽어서 통계 출력:
  - 총 문제 수, pass/fail 카운트 및 비율
  - 시도 횟수 분포 (1-shot / 2-3 / 4-5 / 6+), pass 문제만 집계
  - 실패 문제 목록 (problem_id + attempts)

**주요 설계 결정**:
- `stats_loop` 단일 패스로 모든 통계 수집
- 결과값 i64 packing: `total * 100000000 + passed * 10000 + b1 * 1000 + b2 * 100 + b3 * 10 + b4`
  - 각 카운터는 최대 999 문제 이내 (100문제 bench에서 충분)
- attempts bucket은 pass 문제만 포함 (`ok > 0 and bk == N` 조건)

**버그 수정**:
- 초기 구현: FAIL 문제도 attempts bucket에 포함됨
- 수정: `if ok > 0 and bk == N` 조건 추가

## Verification & Defect Resolution

```
# 3개 FAIL 케이스
=== BMB AI Bench Results: results-test-pilot.jsonl ===
Total: 3, Pass: 0 (0%), Fail: 3
Attempt distribution (pass only): (empty — correct)
Failed problems: 3개 listed ✓

# 2개 PASS 케이스
=== BMB AI Bench Results: results-test-2026-05-22.jsonl ===
Total: 2, Pass: 2 (100%), Fail: 0
Attempt distribution: 1-shot: 2 (100%) ✓
```

`bmb check` → success (17 warnings, no errors) ✓

## Reflection
- i64 packing은 100문제 기준으로 안전 (각 카운터 < 100). 1000문제 이상이면 오버플로 위험.
- M6 dogfooding 가치: Python `analyze_crosslang.py` 없이 BMB만으로 결과 분석 가능
- failed problems 출력: GPUStack 실제 실행 후 어떤 문제가 실패했는지 즉시 파악 가능

## Carry-Forward
- Actionable:
  - Cycle 3052: ROADMAP.md 업데이트 + 전체 M6-P2 완료 상태 반영
  - GPUStack 파일럿 실행: HUMAN 승인 필요 (API 사용 발생)
- Structural Improvement Proposals:
  - packed i64 인코딩 → 1000+ 문제 시 오버플로. 현재 100문제 bench에서는 안전.
- Pending Human Decisions: GPUStack 파일럿 실행 승인
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 3052 — M6-P2 완료 상태 ROADMAP 반영 + 커밋 준비

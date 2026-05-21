# Cycle 2778: D5-B — verify_bench_outputs.py --epsilon 플래그 추가
Date: 2026-05-12

## Re-plan
Plan valid. D5-B: verify_bench_outputs.py epsilon 플래그 추가 — D1 완료 후 네 번째 우선순위. ⚪ NONE.

## Scope & Implementation

`scripts/verify_bench_outputs.py`에 `--epsilon FLOAT` 인수 추가:
- `_tokens_approx_equal(a, b, eps)`: 토큰 쌍 비교. float 파싱 성공 시 `|a-b| <= eps * max(|a|, |b|, 1.0)` (상대 허용), 실패 시 exact match.
- `outputs_match(a, b, epsilon)`: epsilon=None이면 exact match (기존 동작 보존). epsilon 있으면 라인/토큰별 approx 비교.
- `verify_bench()` signature에 `epsilon: Optional[float] = None` 추가.
- `main()`: `--epsilon EPSILON` argparse 인수 + JSON 리포트에 `epsilon` 필드 포함.

## Verification & Defect Resolution

- `python3 scripts/verify_bench_outputs.py --help` → `--epsilon EPSILON` 표시 ✅
- 단위 테스트 (7 cases):
  - exact match no eps: ✅
  - exact mismatch no eps: ✅
  - float eps match (1e-5): ✅
  - float eps fail (1e-6): ✅
  - int mismatch (1e-6): ✅
  - n_body style multi-float: ✅
  - line count mismatch: ✅
  - non-parseable token: ✅
- 부가 검증: D1 fix로 json_serialize bench output이 이제 PASS 확인

## Reflection

Scope fit: ✅. 최소 변경, 기존 동작 완전 보존.
Philosophy drift: 없음.
Roadmap impact: D5-B complete. D5-A 다음.

## Carry-Forward

- Actionable: D5-A — GitHub Actions verify workflow step (Cycle 2779)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2779 — D5-A GitHub Actions step

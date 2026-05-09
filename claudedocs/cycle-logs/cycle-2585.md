# Cycle 2585: Track R Phase 3 — `run` 서브커맨드 구현
Date: 2026-05-09

## Re-plan
Plan valid. Inherited scope: Track R Phase 3 — `run` 서브커맨드 구현. `scripts/run_experiment.py`의 로직을 포터블 모듈로 추출.

## Scope & Implementation
- `ecosystem/bmb-ai-bench/bmb_ai_bench/run_cmd.py` 신규 생성
  - `_select_problems()`: pilot/category/nums 필터링
  - `_run_one_problem()`: generate→check→build→test 루프 (10 iterations max)
  - `run_run()`: CLI entry point, dry-run + JSON 출력 지원
  - `_load_env_file()`: `.env.local` 환경변수 로딩 (포터블, hardcoded path 제거)
  - Rule 8 준수: 진행상황은 stderr, 결과(summary JSON)는 stdout
- `ecosystem/bmb-ai-bench/bmb_ai_bench/cli.py` 업데이트
  - `run` 서브커맨드 stub → 실제 `run_run()` 호출로 교체
  - `--model`/`--api-base`/`--api-key` 선택적으로 변경 (env 폴백 지원)
  - `--pilot`, `--problems`, `--max-loops`, `--dry-run`, `--json` 추가
- `ecosystem/bmb-ai-bench/tests/test_run_cmd.py` 신규 (11 tests)
  - `_select_problems()` 4개 (all/pilot/numbers/category)
  - dry-run human/JSON/category 3개
  - 에러 케이스 2개 (no model)
  - `run_run()` 직접 API 2개

## Verification & Defect Resolution
- `pytest tests/test_run_cmd.py`: ✅ 11/11 passed
- `pytest tests/`: ✅ 26/26 passed (no regressions)

## Reflection
- Scope fit: `run_experiment.py`의 로직을 포터블하게 추출. CLI 통합 완료.
- Latent defects: `_run_one_problem()` 내 `_fake_proc` helper는 내부용으로만 쓰임 — 공개 API 아님.
- Philosophy drift: Rule 8 (machine-first output) 준수. progress→stderr, summary→stdout.
- Roadmap impact: Track R ~82% → ~90%. `run` 서브커맨드 완성.

## Carry-Forward
- Actionable: Track R Phase 3 추가 — `analyze` 서브커맨드 연결 + `report.py` results.json format 일치 확인
- Structural Improvement Proposals: `_fake_proc` inner class → NamedTuple으로 단순화 가능
- Pending Human Decisions: npm publish, v0.100, M3 showcase
- Roadmap Revisions: Track R ~90% 업데이트
- Next Recommendation: Cycle 2586 — `analyze` 서브커맨드 연결 + results.json format 검증

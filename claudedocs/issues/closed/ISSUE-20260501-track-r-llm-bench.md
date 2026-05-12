# ISSUE: Track R — LLM Bench Tracking (bmb-ai-bench)

> **트랙**: R (LLM Bench Tracking)
> **마일스톤**: M2 (AI-Ready Infrastructure)
> **현 상태**: ~40% (`ecosystem/bmb-ai-bench/` 활성)
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 4.2

## 현 상태

- ✅ `ecosystem/bmb-ai-bench/` Python 패키지
  - `bmb_ai_bench/` (`cli.py`, `registry.py`, `doctor.py`, `validate.py`)
  - `runner/` (`bmb_runner.py`, `llm_client.py`, `perf.py`, `base.py`)
  - `analysis/`, `diagnostics/`, `protocol/`
  - `problems/`, `results/`, `tests/`
  - `scripts/run_crosslang.py`, `analyze_crosslang.py`, `run_experiment.py`
- ✅ `ecosystem/ai-proof/` (precursor — `runners/`, `orchestrator/`, `analysis/`)
- ✅ Cross-language experiment 지원
- ⚠️ 50-task 표준 suite 공식화 미점검
- ⚠️ tracking dashboard / 누적 결과 노출 미점검
- ⚠️ "추적만, 합격선 X" 정책 (spec § 6) 코드/문서에 반영 미점검

## 잔여 작업

1. **Phase 1 — 인벤토리**
   - `bmb-ai-bench/problems/` 현재 task 목록 + 카운트
   - `ai-proof` vs `bmb-ai-bench` 관계 (deprecate? 공존?)
   - 결과 storage (`results/`) 구조

2. **Phase 2 — 50-task 표준 선정**
   - 도메인 정합 우선 (컴파일러·DSL·검증기 task)
   - 난이도 계층 (easy/medium/hard, 각 ~17개)
   - LLM 모델별 결과 (Claude/GPT/Llama 등) 누적

3. **Phase 3 — Tracking Dashboard**
   - `bmb llm-bench` 명령 또는 별도 스크립트
   - 누적 결과 시각화 (시간 경과별 모델별 통과율)
   - "합격선 X" 정책 명시 (단순 추적 — 외부 모델 학습 수준은 통제 영역 외)

4. **Phase 4 — CI 통합 (optional)**
   - 새 BMB 버전 릴리스 시 자동 LLM bench 실행
   - 회귀 감지 (BMB 문법 변경이 LLM 정답률 급락 → 알림)

## 완료 조건 (M2 정합)

- [ ] 50-task 표준 suite 명시 (`bmb-ai-bench/STANDARD_TASKS.md` 또는 등가)
- [ ] tracking dashboard 또는 결과 누적 형식 명시
- [ ] "합격선 X" 정책 명시 (README 또는 docs)
- [ ] 옛 ai-proof 정리 (deprecate or merge)

## 추정 사이클

3-4 cycles. Phase 1 = 1 cycle, Phase 2 = 2 cycles (선정 + 검증), Phase 3 = 1 cycle.

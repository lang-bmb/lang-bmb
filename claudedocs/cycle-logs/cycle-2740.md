# Cycle 2740: bmb-ai-bench `run_cmd.py` production code path 동일 truncation 적용

Date: 2026-05-11

## Re-plan

인계: Cycle 2739 carry-forward "추가 자율 작업 발굴". Trigger: 🟠 **RE-PLAN** — Cycle 2739 fix가 `scripts/run_experiment.py` (standalone) 만 다뤘으나, **production code path `bmb_ai_bench/run_cmd.py`** 도 동일 결함 보유 발견. 더 높은 leverage.

## Scope & Implementation

### 진단

`bmb-ai-bench run` 명령의 production 구현 `bmb_ai_bench/run_cmd.py::_run_one_problem`도 messages.append 무한 누적 (line 128-129, 150-151, 183-184). HTTP 413 위험 동일.

Grep `truncat|len(messages)|413` → no matches confirms production path lacks the fix.

스크립트 비교:
- `scripts/run_experiment.py` (Cycle 2739 fix)
- `scripts/run_crosslang.py` (이미 fix, line 199-201)
- `bmb_ai_bench/run_cmd.py` (**미적용 — 본 cycle**)

### Fix

`run_cmd.py::_run_one_problem` for loop 시작 부분에 truncation 추가:

```python
for attempt_num in range(1, max_loops + 1):
    # Context truncation (HTTP 413 prevention): keep initial prompt + last 2 assistant/user pairs
    if len(messages) > 5:
        messages = [messages[0]] + messages[-4:]
    response = llm.generate(sys_instruction, messages)
    code = llm.extract_code(response, "bmb")
```

5-message cap (1 initial + 2 assistant/user pairs). 동일 패턴 3 위치 통일.

### Verification

| 검증 | 결과 |
|------|------|
| Python syntax check (`py_compile`) | ✅ OK |
| `pytest tests/test_run_cmd.py` | ✅ 11/11 PASS |
| `pytest` 전체 ai-bench | ✅ 30/30 PASS |
| `attempts` list 독립성 (data preservation) | ✅ 동일 구조 (run_experiment.py와 같음) |
| bench 진행 영향 | ✅ 없음 (Python script only) |

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Production code path 동일 결함 발견 | ✅ Cycle 2739 후 grep 검증으로 발견 |
| 3 스크립트 정합성 확보 | ✅ run_experiment.py / run_crosslang.py / run_cmd.py 모두 동일 패턴 |
| ai-bench 전체 테스트 통과 | ✅ 30/30 |

결함: 없음.

## Reflection

### Cycle 2739 leverage 확장

Cycle 2739에서 "scripts/run_experiment.py만 fix"하고 close 한 가설이 부분적으로 맞았음. ISSUE 본문에는 `run_experiment.py` 와 `run_crosslang.py` 만 언급되었고, production `run_cmd.py`는 누락. ISSUE 자체가 stale (2026-03-26) — production refactor가 ISSUE 작성 후 일어났을 가능성.

→ "ISSUE close 시 같은 패턴의 더 깊은 위치 추가 grep" 알고리즘 도출 (Cycle 2741+ 활용 후보)

### close 후 추가 leverage 발견 패턴

| Cycle | 패턴 |
|-------|------|
| 2733 | roadmap-sync close 시 v0.98 측정 확인 → 추가 2건 발견 |
| 2735 | if-else/recursive close 시 5/5 deterministic 확인 |
| **2740** | **context-overflow close 시 production code path grep → 누락 발견** |

→ close는 "끝"이 아닌 "다음 leverage 시작점". 양식 표준화 saturation에도 불구하고 새 leverage 영역 발견.

### Active 백로그 변화

| 상태 | 카운트 |
|------|-------|
| Cycle 2737 시작 | 19 active |
| Cycle 2739 종료 | 18 active (-1) |
| **Cycle 2740 (현재)** | **18 active** (동일 — 이미 close된 ISSUE의 production 확장 fix) |

context-overflow-prevention ISSUE는 Cycle 2739에 close 되었지만, **Cycle 2740이 leverage 확장** — production code path까지 fix.

closed/ ISSUE 본문 갱신 권고 (다음 cycle):
- "Implementation Location" 섹션에 `run_cmd.py` 추가

## Carry-Forward

- Actionable:
  - **closed/ISSUE-20260326-context-overflow-prevention.md** Implementation Location 섹션 갱신 (run_cmd.py 추가) — light edit
- Structural Improvement Proposals:
  - 3개 스크립트의 message 관리 패턴 통합 유틸 (예: `bmb_ai_bench/runner/messages.py::truncate_messages(messages, max=5)`) — 현재 5-line snippet 명확하므로 ROI 낮음, defer
  - `bmb-ai-bench run` 명령에 `--max-context-pairs N` CLI 옵션 추가 — 가변 truncation, optional
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: **Cycle 2741** — closed ISSUE 본문 갱신 (light) + 백그라운드 bench 상태 확인 + 자율 발굴 계속

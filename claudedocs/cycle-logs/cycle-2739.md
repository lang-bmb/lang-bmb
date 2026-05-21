# Cycle 2739: context-overflow-prevention 스크립트 fix + close

Date: 2026-05-11

## Re-plan

인계: Cycle 2738 carry-forward "context-overflow-prevention 스크립트 수정 (`run_experiment.py` 슬라이딩 윈도우, 자율, ~30 LOC, bench 무관)". Trigger: ⚪ NONE.

## Scope & Implementation

### 진단

`scripts/run_experiment.py`: feedback loop이 `messages.append` 무한 누적 (3 위치: line 125-126, 145-146, 173-174). HTTP 413 (Payload Too Large) 위험.

`scripts/run_crosslang.py`: 이미 truncation 보유 (line 199-201, 작성 시기 미상). 양 스크립트 정합성 깨짐.

### Fix

`run_experiment.py` for loop 시작 부분에 truncation 추가 (3 위치 모두에 추가하는 대신 단일 진입점):

```python
for attempt_num in range(1, MAX_LOOPS + 1):
    # Context truncation (HTTP 413 prevention): keep initial prompt + last 2 assistant/user pairs
    if len(messages) > 5:
        messages = [messages[0]] + messages[-4:]
    # Generate
    response = llm.generate(sys_instruction, messages)
```

`run_crosslang.py` 패턴과 동일. 5 message cap (initial + 2 pairs).

### Data preservation 검증

`attempts: list[AttemptRecord]`이 `messages`와 독립적으로 누적됨 (line 117, 143, 171). 모든 시도가 결과 JSON에 보존. 따라서 message truncation은 LLM context만 줄이고 분석 데이터는 손실 없음.

### ISSUE close

`ISSUE-20260326-context-overflow-prevention.md`:
- closed/로 이동 + Resolution 섹션 추가
- `_b_track_methodology_stamp.md` reference table 갱신 (~~strikethrough + CLOSED 표기~~)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Python syntax check | ✅ `python -m py_compile` OK |
| Truncation 패턴 run_crosslang.py와 동일 | ✅ |
| Data preservation (attempts list 독립) | ✅ 코드 분석 |
| ISSUE closed/ 이동 + reference 갱신 | ✅ |
| bench 진행 중 영향 | ✅ 없음 (Python script only) |

결함: 없음.

## Reflection

### "이미 fix 되어 있다" 시그널의 가치

`run_crosslang.py`는 이미 fix 있었다. ISSUE 작성 시점 (Cycle 2306-2325, 약 2026-03-26)에 양 스크립트가 정합성이 깨졌고, 한 쪽만 patch되어 있었던 상태. ISSUE는 "정합성 깨짐"을 발견했지만 close 트리거가 없어 1.5개월 stale.

→ 양식 표준화 + v0.98 verify-or-close 알고리즘 (Cycle 2735 확립)이 이런 mismatched-fix-state를 발견할 leverage. 직접 적용 결과 1 추가 close.

### close 후보 saturation 정정

Cycle 2738 결론 "추가 close 후보 부재" 일부 정정: B-track methodology category 안에서도 **인프라 단계 fix** (스크립트 단순 수정)는 자율 close 가능. methodology 자체 (statistical testing / multi-model validation) 는 M4-1 종속이지만, 인프라 layer 일부는 분리 처리 가능.

### Active 백로그 변화

| 상태 | 카운트 |
|------|-------|
| 세션 시작 (Cycle 2737) | 19 |
| Cycle 2739 종료 | **18** (-1 net: context-overflow-prevention close) |
| Closed 누적 | 41 (+1) |

## Carry-Forward

- Actionable: 없음 (Cycle 2739 자체 완결)
- Structural Improvement Proposals: 양 스크립트 (`run_experiment.py` / `run_crosslang.py`) 의 message 관리 패턴 통합 유틸 (e.g., `truncate_messages()` helper) — 단 현재 5-line snippet은 충분히 명확하므로 ROI 낮음, defer
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음
- Next Recommendation: **Cycle 2740** — 백그라운드 bench 상태 확인 + 추가 자율 작업 발굴 (run_experiment.py 외 ai-bench 스크립트의 다른 작은 결함, doc, etc.)

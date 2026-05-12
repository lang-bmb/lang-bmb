# LLM Context Overflow in Feedback Loop

**Status: CLOSED (Cycle 2739, 2026-05-11)** — sliding window truncation 적용
**Priority: MEDIUM (at time of filing)**
**Category: Experiment Infrastructure**

## Resolution

Cycle 2739에서 `scripts/run_experiment.py`에 sliding window truncation 추가:

```python
for attempt_num in range(1, MAX_LOOPS + 1):
    # Context truncation (HTTP 413 prevention): keep initial prompt + last 2 assistant/user pairs
    if len(messages) > 5:
        messages = [messages[0]] + messages[-4:]
    # Generate
    response = llm.generate(sys_instruction, messages)
```

`scripts/run_crosslang.py`는 이미 동일 패턴 보유 (lines 199-201) — 양 스크립트 정합성 확보.

### 측정 stamp (Cycle 2739 close)

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-11 (Cycle 2739 fix) |
| `stale_after` | n/a (CLOSED) |
| `measurement_source` | `scripts/run_experiment.py` line 91-94 |
| `observed_rate` | n/a (preventive fix, M4-1 실행 후 검증 가능) |
| `scope` | `run_experiment.py` 단독 — `run_crosslang.py`는 사전 fix |
| `env_hash` | python 3.12 / 모든 LLM 백엔드 |

### Acceptance Criteria

- [x] Context truncation implemented (keep last N turns) — 5 messages max (1 initial + 2 pairs)
- [x] Data preservation: `attempts` list independent from `messages` list → 결과 JSON 손실 없음
- [⏳] Problem 79 10 loops 무사 완주: M4-1 baseline 실행 시 검증 (HUMAN gate)

## Summary
Problem 79_mini_interpreter hit HTTP 413 (Payload Too Large) during the feedback loop. After 5-6 failed attempts, the accumulated conversation history exceeds API limits, preventing further correction attempts.

## Impact
- 1 problem completely skipped in initial experiment
- Any problem that fails 5+ times risks context overflow
- Biases results: problems that need many corrections are more likely to hit this limit

## Proposed Fix (구현 1번)
1. **Context truncation**: Keep only the last 3 conversation turns (system + last attempt + feedback) — **applied: keep initial + last 2 assistant/user pairs (5 messages max)**
2. Sliding window: Remove oldest assistant/user message pairs when context exceeds threshold — **applied**
3. Summary compression: Replace old attempts with a summary line ("Previous attempts failed with: ...") — not applied (data is preserved in `attempts` list)
4. Max tokens check: Estimate token count before API call, truncate if near limit — not applied (5-message cap is sufficient lower bound)

## Implementation Location
- `scripts/run_experiment.py` lines 91-94 (Cycle 2739 추가)
- `scripts/run_crosslang.py` line 199-201 (사전 fix, 작성 시기 미상)
- `bmb_ai_bench/run_cmd.py::_run_one_problem` line 97-99 (**Cycle 2740 추가 — production code path**)

→ `bmb-ai-bench run` 명령 production 경로 포함 3 위치 모두 정합성 확보 (5-message cap: 1 initial + 2 assistant/user pairs).

## Context
Discovered during first LLM experiment run (Cycle 2306-2325). Fixed Cycle 2739.

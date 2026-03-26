# LLM Context Overflow in Feedback Loop

**Status: OPEN**
**Priority: MEDIUM**
**Category: Experiment Infrastructure**

## Summary
Problem 79_mini_interpreter hit HTTP 413 (Payload Too Large) during the feedback loop. After 5-6 failed attempts, the accumulated conversation history exceeds API limits, preventing further correction attempts.

## Impact
- 1 problem completely skipped in initial experiment
- Any problem that fails 5+ times risks context overflow
- Biases results: problems that need many corrections are more likely to hit this limit

## Proposed Fix
1. **Context truncation**: Keep only the last 3 conversation turns (system + last attempt + feedback)
2. **Sliding window**: Remove oldest assistant/user message pairs when context exceeds threshold
3. **Summary compression**: Replace old attempts with a summary line ("Previous attempts failed with: ...")
4. **Max tokens check**: Estimate token count before API call, truncate if near limit

## Implementation Location
`scripts/run_experiment.py` and `scripts/run_crosslang.py` — the `messages` list grows unboundedly.

## Acceptance Criteria
- [ ] Context truncation implemented (keep last N turns)
- [ ] Problem 79 can complete 10 loops without 413 error
- [ ] No data loss: truncated attempts still recorded in result JSON

## Context
Discovered during first LLM experiment run (Cycle 2306-2325).

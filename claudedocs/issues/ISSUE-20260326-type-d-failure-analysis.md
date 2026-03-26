# Type D (Test Failure) Dominates — Root Cause Analysis Needed

**Status: OPEN**
**Priority: HIGH**
**Category: Compiler / Error Messages**

## Summary
144 out of 164 total error loops (88%) are Type D (test failures — code compiles but produces wrong output). The LLM consistently generates syntactically valid BMB code but fails on algorithmic correctness. The current test failure feedback ("expected X, got Y") is insufficient for the LLM to self-correct.

## Evidence
- Type A (Contract): 0 — contracts never cause issues
- Type B (Syntax): 15 — always self-corrected
- Type C (Semantic): 5 — always self-corrected
- **Type D (Test): 144** — often NOT self-corrected (9 problems fail after 10 loops)

## Root Cause
Test failure feedback is minimal:
```
test_failure: Test 4: expected '20\n', got '0\n'
Fix the error. Output ONLY the complete corrected code.
```

The LLM sees expected vs actual but not:
- Which test input caused the failure
- What the intermediate values should be
- A diff of expected vs actual for all tests
- A hint about which function/logic path is wrong

## Proposed Fixes
1. **Enhanced test feedback**: Include the stdin that caused the failure
   ```
   Test 4 FAILED:
     Input: "5 0 2 0 3 1 4 3"
     Expected: "20\n"
     Got: "0\n"
   ```

2. **Multiple test results**: Show first 3 failures, not just the first one
3. **Diff hints**: If output is partially correct (e.g., "20\n30\n" vs "20\n0\n"), highlight where divergence starts
4. **Trace suggestion**: "Consider adding println() to trace intermediate values"

## Acceptance Criteria
- [ ] Test feedback includes stdin for failed test case
- [ ] First 3 failures shown (not just first)
- [ ] Type D self-correction rate improves (measured pre/post)
- [ ] At least 2 of the 9 currently-failing problems become passing

## Context
This is the most impactful improvement for BMB's AI-friendliness. Reducing Type D failures by even 20% would push overall success from 90% to ~94%.

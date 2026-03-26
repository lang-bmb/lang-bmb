# BMB 1-shot Success Rate Lower Than C/Python (56% vs 65%/67%)

**Status: OPEN**
**Priority: MEDIUM**
**Category: Compiler / Error Messages**

## Summary
BMB's first-attempt success rate (56.3%) is 9-11%p lower than C (64.7%) and Python (67.2%). The error feedback loop compensates (final rate 90%), but in single-shot generation scenarios (no feedback), BMB would underperform.

## Impact
- Use cases without feedback loops (one-shot code generation, IDE completion) would see BMB as harder
- The 56% → 90% recovery (+34%p) depends entirely on enriched error messages
- If error message quality degrades, BMB's advantage collapses

## Root Cause Analysis
BMB 1-shot failures by error type (from loop type distribution):
- **B (Syntax): 15 occurrences** — unfamiliar syntax (`else { () }`, `;` placement)
- **C (Semantic): 5 occurrences** — type mismatches, unknown builtins
- **D (Test): 144 occurrences** — correct syntax but wrong logic (algorithm errors)

Most 1-shot failures are Type D (logic errors), same across all languages. The BMB-specific gap (~10%p) comes from B+C errors that C/Python don't have.

## Proposed Fixes
1. **Expand BMB Reference**: Add more patterns (conditional patterns, loop idioms, common mistakes)
2. **PatternBank enhancement**: Pre-emptively catch common LLM mistakes in the reference doc
3. **Syntax simplification**: Reduce mandatory `else { () }` boilerplate (language spec change)
4. **Better code examples**: Add 5-10 complete working programs to the reference

## Acceptance Criteria
- [ ] 1-shot rate ≥ 65% (matching C baseline)
- [ ] Reference document expanded with common mistake patterns
- [ ] Measure 1-shot rate before/after reference improvement

## Context
While BMB ultimately wins on final success rate, the 1-shot gap matters for developer experience and tooling integration.

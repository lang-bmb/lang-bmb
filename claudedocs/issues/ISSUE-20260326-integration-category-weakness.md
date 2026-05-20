# BMB Integration Category Weakness (63% vs Python 76%)

**Status: PARTIALLY RESOLVED** — Cycle 2983 (2026-05-20): B-axis 차원에서는 ALL integration 문제 GPUStack 1-shot 통과. Cycle 2991 재검토: crosslang 측정은 여전히 stale, HUMAN-blocked.
**Priority: LOW** (B-axis 해소됨, crosslang만 남음 — HUMAN-blocked)
**Category: Language Design / Compiler**

## Summary
BMB scores 63.3% on Integration problems vs Python 76% and C 67%. This is the only category where BMB loses to both competitors. Multi-function composition and complex state management are harder in BMB's expression-based style.

## Failed Problems Analysis
| Problem | BMB | C | Python | Root Cause |
|---------|-----|---|--------|------------|
| 76_multi_function | FAIL | FAIL | FAIL | 5+ function composition with abs/sign/sum |
| 79_mini_interpreter | FAIL | FAIL | FAIL | Context overflow (payload too large) |
| 81_dispatch_table | FAIL | FAIL | FAIL | Op-code based dispatch |
| 83_pipeline | FAIL | FAIL | FAIL | Sequential array transforms |
| 85_registry_pattern | FAIL | FAIL | PASS | Key-value store with update |

## Root Causes
1. **Expression-based `if/else` verbosity**: Every branch must have matching types and explicit `else { () }`, making multi-step logic harder to read/write
2. **No method syntax**: `vec_set(v, i, vec_get(v, i) + 1)` vs Python `v[i] += 1`
3. **No destructuring**: Complex return values require manual unpacking
4. **Context accumulation**: Failed attempts add to conversation length → API payload limits

## Proposed Fixes (Decision Framework: Language Spec → Compiler)
1. **Language spec**: Consider `v[i]` sugar for `vec_get(v, i)` (syntactic improvement)
2. **Language spec**: Consider `if cond { body }` without mandatory `else` for statement position
3. **Compiler**: Better error messages for multi-function problems (which function failed)
4. **Experiment**: Add context truncation to prevent payload overflow

## Acceptance Criteria
- [ ] Root cause analysis for each failed integration problem
- [ ] At least 2 integration problems fixed by language/compiler improvement
- [ ] Integration success rate ≥ 75% (matching Python)

## Context
This is the only category where BMB underperforms. Per CLAUDE.md: "언어 한계는 답이 아니다 — 언어를 바꾼다."

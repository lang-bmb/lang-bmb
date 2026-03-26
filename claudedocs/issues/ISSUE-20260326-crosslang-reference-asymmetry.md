# Cross-Language Experiment: Reference Document Asymmetry

**Status: OPEN**
**Priority: HIGH**
**Category: Experiment Methodology**

## Summary
In the cross-language comparison (BMB vs C vs Python), BMB receives a 117-line Quick Reference document while C and Python receive none. This is the single largest methodological weakness — it makes the comparison unfair and prevents claiming "BMB is more AI-friendly than C/Python" without qualification.

## Impact
- Current results (BMB 90% vs C 82% vs Python 84%) may be partially inflated by the reference advantage
- Cannot quantify how much of the 8%p gap is from language design vs reference document
- Academic reviewers would flag this as confounding variable

## Proposed Fix
1. **Create equivalent reference documents** for C and Python (~100 lines each, covering stdin/stdout patterns, array handling, common pitfalls)
2. **Run 4 conditions**: BMB+ref, BMB-ref, C+ref, C-ref (factorial design)
3. **Isolate reference effect**: measure `(BMB+ref - BMB-ref)` vs `(C+ref - C-ref)`
4. If BMB still wins after removing reference advantage → claim is fully supported

## Acceptance Criteria
- [ ] C Quick Reference document (~100 lines)
- [ ] Python Quick Reference document (~100 lines)
- [ ] 4-condition experiment (at least 30 problems × 3 runs × 4 conditions = 360 runs)
- [ ] Quantified reference effect size

## Context
Discovered during AI-Bench cross-language experiment (Cycle 2306-2325).

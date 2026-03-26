# Statistical Significance Testing Missing

**Status: OPEN**
**Priority: MEDIUM**
**Category: Experiment Analysis**

## Summary
The experiment reports raw percentages and paired win counts but no formal statistical tests. Without p-values and confidence intervals, the observed differences (BMB 90% vs C 82%) cannot be formally distinguished from chance variation.

## Impact
- 8%p difference may or may not be statistically significant at α=0.05
- No confidence intervals for success rates
- Academic publication requires formal testing

## Proposed Fix
1. **Wilcoxon signed-rank test** for paired BMB vs C and BMB vs Python (per-problem success rates)
2. **McNemar's test** for paired binary outcomes (pass/fail per problem)
3. **95% confidence intervals** for each language's success rate (Wilson score interval)
4. **Effect size**: rank-biserial correlation for Wilcoxon, odds ratio for McNemar
5. **Bonferroni correction** for multiple comparisons (3 pairwise tests)

## Data Available
- 100 problems × 3 runs per language
- Per-problem success rate computable (0, 0.33, 0.67, 1.0)
- Paired data: each problem tested in all 3 languages

## Acceptance Criteria
- [ ] Wilcoxon signed-rank: BMB vs C, BMB vs Python (p-value, effect size)
- [ ] McNemar's test on aggregated pass/fail
- [ ] 95% CI for each language success rate
- [ ] Bonferroni-corrected significance threshold
- [ ] Summary: "BMB > C is significant at p=X" or "not significant"

## Dependencies
Requires `scipy` (optional dependency in pyproject.toml, already listed).

## Context
Identified in objectivity review of cross-language experiment results.

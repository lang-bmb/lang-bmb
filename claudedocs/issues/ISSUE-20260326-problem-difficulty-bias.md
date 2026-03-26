# Problem Suite Difficulty Bias Toward Easy/Medium

**Status: OPEN**
**Priority: LOW**
**Category: Experiment Methodology**

## Summary
The 100-problem suite is dominated by easy/medium problems. Hard problems (LIS, knapsack, topological sort) are a small minority. This inflates success rates for all languages and reduces the experiment's ability to differentiate language quality on complex tasks.

## Current Distribution (estimated)
- Easy: ~40 problems (basic I/O, simple loops, single function)
- Medium: ~45 problems (multiple functions, data structures, moderate logic)
- Hard: ~15 problems (DP, graph algorithms, complex state management)

## Impact
- 90% success looks impressive but may not reflect real-world BMB usage
- Real systems programming tasks (compiler writing, runtime implementation) are much harder
- Easy problems show all languages at 100% — no differentiation

## Proposed Fix
1. **Add 20 hard problems** (number 101-120):
   - Graph algorithms (Dijkstra, BFS/DFS, SCC)
   - Dynamic programming (edit distance, longest common subsequence)
   - Complex data structures (balanced BST, hash table)
   - Multi-module programs (import system testing)
   - File I/O intensive tasks
2. **Difficulty-weighted scoring**: Hard problems worth 3x, medium 2x, easy 1x
3. **Real-world problems**: Port actual BMB compiler tests as benchmarks

## Acceptance Criteria
- [ ] At least 10 new hard problems added
- [ ] Success rate reported separately by difficulty
- [ ] Difficulty-weighted score calculated

## Context
Identified in objectivity review. The current suite proves BMB works for algorithmic problems but not for systems programming complexity.

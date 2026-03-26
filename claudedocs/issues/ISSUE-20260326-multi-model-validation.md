# Multi-Model Validation Required

**Status: OPEN**
**Priority: HIGH**
**Category: Experiment Methodology**

## Summary
All 899 experiment runs used a single LLM (claude-text). Results may not generalize to other models. The "AI-friendly" claim requires demonstration across multiple LLMs.

## Impact
- claude-text may have specific strengths parsing BMB reference documents
- Other models (GPT-4o, Qwen, Gemini) may show different patterns
- Single-model results are a pilot, not a generalizable finding

## Available Endpoints (.env.local)
- **GPUStack Qwen**: `http://122.35.14.215:8021` (qwen3-vl-30b-a3b-thinking-awq)
- **OpenAI GPT-4o**: `https://api.openai.com/v1` (API key available)
- **Anthropic Claude**: `https://api.anthropic.com/v1` (API key available)

## Proposed Fix
1. Run the same 100 problems × 3 languages × 3 runs on at least 2 additional models
2. Compute Kruskal-Wallis test for model × language interaction
3. If BMB leads across 3+ models → generalization supported

## Acceptance Criteria
- [ ] GPT-4o experiment (100 × 3 × 3 = 900 runs)
- [ ] Qwen experiment (100 × 3 × 3 = 900 runs)
- [ ] Cross-model comparison table
- [ ] Kruskal-Wallis / Friedman statistical test
- [ ] "BMB leads on N/M models" statement with evidence

## Context
Discovered during AI-Bench objectivity review (Cycle 2306-2325).

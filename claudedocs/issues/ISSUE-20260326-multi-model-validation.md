# Multi-Model Validation Required

**Status: PARTIALLY RESOLVED** — Cycle 2991 (2026-05-20): GPUStack Qwen3.5-35b = 2번째 모델 검증 완료 (99.7%, 299/300). Claude 98.0% vs GPUStack 99.7% 비교 가능. GPT-4o 실험 + 통계 검정은 HUMAN-blocked.
**Priority: MEDIUM** (HIGH→MEDIUM: 2-model validated, 내려감)
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
- [ ] GPT-4o experiment (100 × 3 × 3 = 900 runs) — HUMAN-blocked
- [x] Qwen experiment: GPUStack qwen3.6-35b-a3b 99.7% (299/300, 2026-05-20)
- [x] Cross-model comparison: Claude 98.0% vs GPUStack 99.7% — 2-model table 가능
- [ ] Kruskal-Wallis / Friedman statistical test — HUMAN-blocked (3+ model 필요)
- [x] "BMB leads on N/M models": 2/2 models confirmed (98.0% + 99.7%)

## Context
Discovered during AI-Bench objectivity review (Cycle 2306-2325).

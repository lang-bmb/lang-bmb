# Cycle 2837: str_replace + str_repeat Builtins
Date: 2026-05-14

## Re-plan
Plan valid. ROADMAP ① next: str_replace, str_repeat. str_starts_with/str_ends_with already existed (Cycle 2828). Pattern-following: interpreter-only builtins, identical pattern to Cycles 2828/2833/2836.

## Scope & Implementation
- eval.rs: Added builtin_str_replace (replace-all), builtin_str_repeat (n-times repetition)
- register_builtins: Registered str_replace + str_repeat
- types/mod.rs: str_replace(String,String,String)->String, str_repeat(String,i64)->String
- bmb_reference.md: Updated String section with str_replace + str_repeat entries
- tests/integration.rs: Added test_interp_str_replace_repeat (5 cases)

## Verification & Defect Resolution
- test_interp_str_replace_repeat: PASS (5/5 cases)
- No defects. str_replace delegates to Rust String::replace (replace-all), str_repeat to String::repeat.
- n<=0 guard returns empty string correctly.

## Reflection
- Scope fit: 100%
- Latent defects: None
- Structural improvements: None
- Philosophy drift: None
- Roadmap impact: ① language gap list continues to shrink. Remaining major item: for-in-vec (고복잡도 — deferred).

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: ROADMAP ① updated (Cycle 2837 str_replace/str_repeat ✅)
- Next Recommendation: Cycle 2838 — additional bmb_reference algorithmic patterns (graph algorithms, DP patterns, string algorithms), OR attempt for-in-vec (high complexity), OR more utility builtins (str_join, vec_contains, etc.)

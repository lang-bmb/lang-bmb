# Cycle 2775: D6 — Submodule revert (lexer/brainfuck workload amp)
Date: 2026-05-12

## Re-plan
Plan valid. D6 첫 번째 우선순위 확인 (HANDOFF ordering). ⚪ NONE.

## Scope & Implementation
HANDOFF D6: Cycle 2765에서 추가한 lexer/brainfuck workload amplification 변경 revert.
- `ecosystem/benchmark-bmb/benches/real_world/lexer/c/main.c` — 1000x → 100x 복원
- `ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` — 1000x → 100x 복원
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/c/main.c` — 99 → 9 outer loop 복원
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main.bmb` — 99 → 9 outer loop 복원

명령: `git checkout -- benches/real_world/lexer/ benches/real_world/brainfuck/`

## Verification & Defect Resolution
- `git status --short`: untracked 2건 (mandelbrot inproc — prior session, 그대로 유지) ✅
- parent repo `git diff ecosystem/benchmark-bmb`: 출력 없음 (submodule pointer 변경 없음) ✅
- `lexer/c/main.c` `generate_large_source(100)` 확인 ✅

## Reflection
Scope fit: ✅. 효과 없는 변경 제거 — Principle 2 (Workaround 금지) + BMB 철학 정렬.
Philosophy drift: 없음.
Roadmap impact: D6 complete, D4 다음.

## Carry-Forward
- Actionable: D4 `.gitignore` 정책 (Cycle 2776)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2776 D4 — .gitignore 예외 추가

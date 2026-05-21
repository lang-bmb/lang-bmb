# Cycle 2802: bootstrap stack overflow P3 fix (hash_table)
Date: 2026-05-13

## Re-plan
Final cycle of this session (2793-2802). Checked remaining open ISSUEs:
- B-track ISSUEs: blocked (need API access)
- tier3-spawn-overhead: P2, HUMAN decision needed
- playground-wasm: P2, multi-cycle
- Carry-forward from Cycle 2800 (Rule 13 false positive): moot — `has_todo_call`
  already has `is_ident_char` prefix guard at line 490 of lint.bmb

Actionable P3: ISSUE-20260512-bootstrap-parser-stack-overflow. Single-cycle option c
(Windows linker --stack) already implemented in Rust pipeline (Cycle 2780 D2), but
`bootstrap/compiler.exe` was built before that patch. Plan: rebuild.

## Scope & Implementation

**Root cause investigation:**
- `bootstrap/compiler.exe` PE header: `SizeOfStackReserve = 2097152` (2MB)
- Rust compiler's build pipeline adds `-Wl,--stack,67108864` (64MB) since Cycle 2780 D2
- Current exe was built before that patch → stale binary

**Fix:**
- `./target/release/bmb build bootstrap/compiler.bmb -o bootstrap/compiler.exe --release`
- New exe: `SizeOfStackReserve = 67108864` (64MB) ✅

**Files changed:**
- `bootstrap/compiler.exe` (binary rebuild — not tracked in git)

## Verification & Defect Resolution

- `bootstrap/compiler.exe build ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb`
  → `{"type":"build_success"}` ✅ (previously: STATUS_STACK_OVERFLOW 0xC00000FD)
- hash_table exe runs correctly (output: 95259, 100000, 46445) ✅
- `cargo test --release --all`: 2377/2377 PASS ✅
- `verify_bench_outputs.py --tier 1`: 9/10 PASS (1 mismatch = pre-existing n_body fp precision) ✅

## Reflection

- **Scope fit**: Minimal targeted fix. No code changes required — only a binary rebuild.
- **Latent defects**: n_body precision mismatch is pre-existing (fp representation difference
  between BMB and C baselines, not a regression). Stage 2/3 Fixed Point unchanged (no
  source modification, Cycle 2792 still authoritative).
- **Philosophy drift**: None. The fix leverages an already-landed compiler improvement (D2,
  Cycle 2780) that just hadn't been applied to the stored binary.
- **Roadmap impact**: Active ISSUE 13 → 12. P3 cleared.
- **User-facing quality**: hash_table bench is now buildable via bootstrap path, enabling
  proper bootstrap-path performance comparisons for that bench.

## Carry-Forward
- Actionable: None.
- Structural Improvement Proposals:
  - The bootstrap/compiler.exe binary is not tracked in git, so rebuilds are manual.
    A CI step or script that auto-rebuilds compiler.exe after Rust compiler updates
    would prevent stale-stack issues from reappearing. P4, low urgency.
- Pending Human Decisions:
  - ISSUE-20260512-tier3-spawn-overhead-methodology (P2): Option A/B/C choice needed
  - B-track ISSUEs: API access required
- Roadmap Revisions: ROADMAP updated, Active ISSUE 12. Cycle 2802 갱신 섹션 추가.
- Next Recommendation: Cycle 2803 — P2 work depends on HUMAN decisions.
  If no P1/P2 unblocked, focus on lint quality (Rule 13 already has guard),
  playground-wasm Phase 1 scoping, or CI improvements.

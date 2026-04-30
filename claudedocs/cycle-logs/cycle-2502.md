# Cycle 2502: G.4 phi_load_map dedup — root-cause fix
Date: 2026-04-30

## Re-plan
Plan valid. With Cycle 2500 push (CI in_progress), Z3 unavailable locally
(blocking G.1), proceed with G.4 latent fix from HANDOFF priority 4.

## Scope & Implementation

**Latent risk** (HANDOFF Cycle 2494 audit):
- `phi_load_map` keyed by `(dest_block, local_name, pred_block) → load_temp`.
- `load_temp` constructed as `format!("{}.phi.{}", p.name, pred_label)` —
  determined by **(name, pred) only**, not dest_block.
- Iteration at `llvm_text.rs:2454` filters by `pred_block == &block.label`
  but does NOT filter by dest_block.
- **Failure mode**: when two distinct phi destination blocks reference the
  same `(local, pred)`, the map has two entries with **identical load_temp
  values**. Both match the iteration filter → two `%X.phi.Y = load ...`
  instructions emitted in the same block → LLVM IR redefinition error.
- Currently latent — no test triggers this branch shape (single predecessor
  branching to multiple successors that each have a phi for the same local).

**Decision Framework analysis** — Level 2 (Compiler structure). The data
structure was never honest: dest_block was redundant in the key. Two
options:
1. **Defensive dedup in iteration** (HashSet check)
2. **Honest key** — drop dest_block; insert dedupes naturally

Per CLAUDE.md "근본 해결" principle, chose option 2.

**Verification of safety** before change:
- Searched all `phi_load_map` consumers (5 sites + signatures).
- Iteration site `llvm_text.rs:2454` destructures `(_dest_block, ...)` — never used.
- Lookup site `llvm_text.rs:2493` filters by `(_, ln, pb)` — never used.
- Function param site `llvm_text.rs:6525-6546` regenerates the name from
  `(p.name, label)` only — confirms dest_block is irrelevant.

**Fix** (`bmb/src/codegen/llvm_text.rs`):
- Map type: `HashMap<(String, String, String), String>` →
  `HashMap<(String, String), String>` (5 sites: declaration, 2 fn signatures,
  iteration destructure, find-predicate).
- Insert key: `(block.label.clone(), p.name.clone(), pred_label.clone())` →
  `(p.name.clone(), pred_label.clone())`.
- Comment updated to explain the deliberate omission of dest_block.

**Files changed**:
- `bmb/src/codegen/llvm_text.rs` — 5 hunks

## Verification & Defect Resolution

| Check | Result |
|-------|--------|
| `cargo build --release` | ✅ 4m 42s |
| `cargo test --release --lib` | ✅ **3,772 pass** / 0 fail |
| `cargo test --release --lib --features llvm --target x86_64-pc-windows-gnu` | ✅ **3,953 pass** / 0 fail |
| `cargo clippy --all-targets -- -D warnings` | ✅ clean |
| `bash scripts/bootstrap.sh --stage1-only` | ✅ 22.5s |

No defects found.

## Reflection

**Scope fit**: ✅ Surgical change. The new key shape is also marginally
faster (HashMap with 2-tuple key vs 3-tuple) — though this isn't
meaningful given the small size of the map.

**Latent defects discovered**: None. Cycle 2494 audit had also flagged
phi_string_map and phi_coerce_map as similar shapes; on examination
those use **counter-based** temp names (`_str_phi_N`, `_phi_sext_N`),
so distinct keys yield distinct counter values → no collision risk.
Only phi_load_map had the formula-based naming that allowed value collision.

**Philosophy drift**: None. Level 2 honest data structure.

**Roadmap impact**: None — defensive cleanup, no observable change. The
"latent until trigger appears" risk in HANDOFF G.4 is now structurally
eliminated rather than monitored.

## Carry-Forward
- **Actionable**: Commit + push. Continue observing Cycle 2500 CI.
- **Pending Human Decisions**: Z3 install (G.1), TestPyPI token (B'.2).
- **Roadmap Revisions**: None.
- **Next Recommendation**: Cycle 2503 = commit/push; check CI; if Cycle
  2500 windows-latest CI green → mark B'.1 complete; consider H tier C
  (Bootstrap+Benchmark PR-only) or session early-terminate.

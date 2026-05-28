# Cycle 3249: Bootstrap Bare-Filename SIGSEGV — Investigation & Documentation
Date: 2026-05-28

## Re-plan

Initial scope (inherited): Continue M11-C Phase 14 — investigate SIGSEGV crash in bootstrap compiler
triggered by `--` comment before compound const declarations.

**Trigger**: 🟠 RE-PLAN fired mid-cycle. Initial hypothesis (comment triggers crash) was wrong.
Actual trigger: bare filename with no path separator. After extensive investigation (50+ tool calls)
with root cause not found, advisor intervened: document as ISSUE, do NOT implement workaround.

Adjusted scope: characterize crash fully, document as ISSUE, clean up temp files, no code change.

## Scope & Implementation

### Investigation findings

**False hypothesis**: `--` comment before compound const → SIGSEGV  
**Actual trigger**: any bare filename (no `/`, `\`, or `.` prefix) → SIGSEGV

Pattern fully characterized:
- `bare.bmb` → exit 139 (SIGSEGV)
- `./bare.bmb` → OK
- `/absolute/path.bmb` → OK
- `dir/file.bmb` → OK

**Root cause (partial)**:
- `include_dirname("bare.bmb")` returns `""` (empty string) — no separator found
- `include_dirname("./bare.bmb")` returns `"."` — works correctly
- Compile itself succeeds (`{"type":"build_success",...}` printed)
- SIGSEGV occurs AFTER compilation, likely in `bmb_arena_destroy()` during cleanup
- Exact connection from `src_dir = ""` to arena corruption NOT found

**Pre-existing bug**: Verified with `bootstrap/compiler.exe` from before Cycle 3248.
Bug was not introduced by recent cycles.

### Actions taken

1. Characterized crash pattern with minimal reproduction cases
2. Verified pre-existing status with pre-Cycle-3248 binary
3. Identified `include_dirname` as the source of `src_dir = ""` behavior
4. Advisor guidance: ISSUE documentation, no workaround per Principle 2
5. Created: `claudedocs/issues/bootstrap-bare-filename-sigsegv.md`
6. Cleaned up: 9 temp `.bmb`/`.ll`/`.exe` files from CWD + `/tmp/` investigation artifacts
7. Removed: duplicate `.out` files from `tests/bootstrap/` (identical copies of `tests/golden/`)
8. Restored: `bootstrap/compiler.exe` to committed state via `git checkout`

### Files changed

- **Created**: `claudedocs/issues/bootstrap-bare-filename-sigsegv.md`
- **No code changes** (correct decision per advisor + Principle 2)

## Verification & Defect Resolution

- `bootstrap/compiler.exe` verified working with absolute paths (golden test IR generation: exit 0)
- No regressions (no code modified)
- `git status` clean (only expected untracked: ecosystem/*, new ISSUE file)

## Reflection

**Scope fit**: Investigation scope was appropriate once hypothesis was corrected. ISSUE documentation
is the correct deliverable — implementing a fix without understanding the arena corruption mechanism
would be a workaround, violating Principle 2.

**Latent defects**: The bare-filename SIGSEGV is a latent defect (P3, non-blocking). All real
workflows use `./file.bmb` or absolute paths. The golden test runner uses absolute paths. Not urgent.

**Philosophy drift**: None. Correctly avoided workaround per Principle 2.

**Roadmap impact**: None. M11-C Phase 13 (Cycle 3248) remains valid and complete. The SIGSEGV
investigation was a detour — M11-C work can resume normally.

**User-facing quality**: N/A (no output changes).

## Carry-Forward

- **Actionable**: None — ISSUE documented, cleanup complete, no deferred defects
- **Structural Improvement Proposals**:
  - Fix `include_dirname` to return `"."` instead of `""` for bare filenames (POSIX `dirname` semantics)
  - OR normalize input path at `emit_ir`/`build_file_ex` entry points
  - Root-cause analysis of arena corruption trigger needed before implementing either
- **Pending Human Decisions**: None
- **Roadmap Revisions**: None — M11-C Phase 14 remains next planned work
- **Next Recommendation**: Resume M11-C — next Phase 14 cycle should tackle the next compound
  const / stack array improvement on the M11-C roadmap. The bare-filename SIGSEGV (ISSUE filed)
  can be addressed separately as a P3 cleanup task when root cause becomes clear.

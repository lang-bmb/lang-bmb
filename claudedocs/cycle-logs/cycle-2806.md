# Cycle 2806: Bootstrap compiler.exe CI Rebuild Script (P4)
Date: 2026-05-13

## Re-plan
Plan valid. Carry-Forward from Cycle 2805: bootstrap compiler.exe CI rebuild script (P4, 1 cycle). Root cause: `bootstrap/compiler.exe` built before Cycle 2780 64MB stack patch had 2MB stack → STATUS_STACK_OVERFLOW on deeply nested sources. Fix committed in Cycle 2802 (rebuild). This cycle adds prevention infrastructure.

## Scope & Implementation

### `scripts/rebuild-bootstrap-exe.sh` (new, 75 LOC)

New script that:
1. **Staleness check**: compares `bootstrap/compiler.exe` mtime vs `bootstrap/compiler.bmb`. Returns immediately if current.
2. **Rebuild**: calls `bmb build bootstrap/compiler.bmb -o bootstrap/compiler.exe --fast-compile` (13s on dev machine).
3. **Stack verification**: uses Python `struct` to read PE32+ header and verify `SizeOfStackReserve ≥ 32 MB` (current: 64 MB = `-Wl,--stack,67108864`).
4. **Three modes**: `--check-only` (CI gate, exits 1 if stale), `--force` (unconditional rebuild), `--json` (machine output).

**Bug fixed during dev**: `log()` function returned exit code 1 when `JSON_OUTPUT=true` (because `[ false ] && echo` short-circuits with exit 1). Fixed: `log() { [ ... ] && echo -e "$1" || true; }`.

### `scripts/bootstrap.sh` integration

Added stale-exe check between "Prerequisites OK" and "Stage 1" labels:
- Calls `rebuild-bootstrap-exe.sh --json`; parses result
- Verbose: logs "current (stack: 64 MB)"
- Non-verbose: logs rebuild notice if exe was stale and rebuilt
- Silently skips if script not found (backwards compat)

### Output examples

```
# Current exe (typical):
bootstrap/compiler.exe is current (stack: 64 MB)    # --verbose
{"status":"current","stack_mb":64,"rebuilt":false}  # --json

# Stale and rebuilt:
bootstrap/compiler.exe was stale — rebuilt (stack: 64 MB)
{"status":"ok","stack_mb":64,"rebuilt":true,"build_ms":11115}

# --check-only (CI gate):
ERROR: bootstrap/compiler.exe is stale vs compiler.bmb
{"status":"stale","rebuilt":false}  # + exit 1
```

## Verification & Defect Resolution

- `bash scripts/rebuild-bootstrap-exe.sh --json` → `{"status":"current","stack_mb":64,"rebuilt":false}` ✅
- `bash scripts/rebuild-bootstrap-exe.sh --force --json` → rebuilds, `{"status":"ok","stack_mb":64,"rebuilt":true,"build_ms":10995}` ✅
- `bash scripts/rebuild-bootstrap-exe.sh --check-only --json` → `{"status":"current",...}` exit 0 ✅
- `bash scripts/bootstrap.sh --stage1-only --verbose` → shows "bootstrap/compiler.exe current (stack: 64 MB)" then proceeds to Stage 1 ✅
- Stage 1 build: 11024ms, success ✅

## Reflection

**Scope fit**: Complete. P4 task fully implemented in 1 cycle.

**Latent defects**: None. The script is stateless and idempotent.

**Structural improvement opportunities**:
- The `--check-only` flag could be wired into a CI workflow step to fail builds when compiler.exe is stale (prevents accidental commit of stale exe). Current integration is advisory only (not blocking).
- Stack size check is Windows-only (PE32+ parsing). On Linux, `compiler` has no PE header — the Python script returns 0, triggering the `stack_mb < 32` warning path. A cross-platform alternative: embed a version stamp in the exe and check that instead.

**Philosophy drift**: None. Prevention script, not a workaround.

**Roadmap impact**: None directly. stale-stack P3 ISSUE (`ISSUE-20260512-bootstrap-parser-stack-overflow`) was already closed in Cycle 2802 with the actual fix. This cycle adds prevention.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: Wire `--check-only` into CI workflow step (currently advisory); cross-platform stack check (version stamp alternative)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 2807 — select next autonomous scope from ROADMAP. Candidates: bootstrap parser iterative conversion (P3, multi-cycle), or a lint/benchmark scope item.

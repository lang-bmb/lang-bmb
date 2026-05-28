# ISSUE: Bootstrap compiler SIGSEGV on bare filename input

**Discovered:** Cycle 3249 (2026-05-28)  
**Status:** Open (P3 — non-blocking)  
**Affects:** `bootstrap/compiler.exe` (all subcommands: `emit-ir`, `build`, etc.)

---

## Symptom

Bootstrap compiler crashes with SIGSEGV (exit code 139) when input file is specified as a **bare filename** (no path separator).

```
$ cd D:/data/lang-bmb
$ ./bootstrap/compiler.exe emit-ir simple_test.bmb /tmp/out.ll
# → exit 139 (SIGSEGV)
```

## Reproduction

```powershell
# Create a minimal BMB file in CWD
echo 'fn main() -> i64 = 42;' > simple_test.bmb

# Bare filename → CRASH
./bootstrap/compiler.exe emit-ir simple_test.bmb /tmp/out.ll
# exit 139

# With ./ prefix → OK
./bootstrap/compiler.exe emit-ir ./simple_test.bmb /tmp/out.ll
# exit 0

# Absolute path → OK
./bootstrap/compiler.exe emit-ir D:/data/lang-bmb/simple_test.bmb /tmp/out.ll
# exit 0
```

## Pattern

| Input form | Result |
|------------|--------|
| `bare.bmb` | **SIGSEGV** (exit 139) |
| `./bare.bmb` | OK |
| `../dir/bare.bmb` | OK |
| `/absolute/path.bmb` | OK |
| `relative/dir/file.bmb` | OK |

**Trigger**: Any input path containing NO path separator (`/`, `\`, or `.` prefix).

## Root Cause (Partial)

`include_dirname` in `bootstrap/compiler.bmb` (lines ~245–275):
- For `"bare.bmb"`: scans for last `/` or `\`, finds none → returns `""` (empty string)
- For `"./bare.bmb"`: finds `.` separator → returns `"."`

This gives `src_dir = ""` for bare filenames. The compile step itself succeeds (output `{"type":"build_success",...}` is printed), but the crash occurs after compilation — likely during arena cleanup (`bmb_arena_destroy` in `bmb_runtime.c:2165`) when freeing arena blocks. The exact mechanism connecting `src_dir = ""` to memory corruption was not found.

## Pre-existing Bug

Verified with `bootstrap/compiler.exe` from before Cycle 3248 — same crash pattern. This bug was NOT introduced by any recent cycle.

## Workaround (for users)

Always invoke the bootstrap compiler with an explicit path separator:
- `./myfile.bmb` instead of `myfile.bmb`
- Or use an absolute path

## Impact

**Non-blocking**: 
- Golden test runner uses `${TESTS_DIR}/${FILENAME}` (absolute path) — unaffected
- Normal user workflow uses `./file.bmb` or full paths — unaffected  
- No production code path hits bare filename without separator

## Fix Direction

Fix `include_dirname` to return `"."` instead of `""` when no separator is found (matching POSIX `dirname` semantics), OR normalize the input path at the top of `emit_ir`/`build_file_ex` entry points. Root-cause analysis of why `src_dir = ""` causes arena corruption is needed before implementing either fix.

**Note**: Per CLAUDE.md Principle 2, do NOT add a workaround (e.g., silently prepending `./`) without understanding the arena corruption mechanism.

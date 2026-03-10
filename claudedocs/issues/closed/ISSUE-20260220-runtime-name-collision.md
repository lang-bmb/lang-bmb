# Runtime Function Name Collision in Bootstrap Compiler

**Status: RESOLVED — Fixed in Cycle 1475 (user functions shadow builtins in interp + text codegen)**

## Summary
The bootstrap compiler prefixes all user-defined functions with `bmb_` when generating LLVM IR. This creates namespace collisions with functions defined in the BMB runtime (`bmb/runtime/bmb_runtime.c`).

## Severity: HIGH

## Reproduction
1. Define a function named `parse_int` in user code
2. Compile with `bmb_stage1.exe build`
3. The user function becomes `bmb_parse_int` in LLVM IR
4. `bmb_parse_int` already exists in `bmb_runtime.c:833`
5. Linker silently resolves to the wrong function

## Symptoms
- Function appears to return wrong values (typically 0)
- Only manifests when called from deep if/else chains (8+ branches)
- Internal debug prints within the function don't execute
- Extremely difficult to diagnose — looks like a codegen bug

## Root Cause
`bmb/runtime/bmb_runtime.c` defines many `bmb_*` functions:
- `bmb_parse_int` (line 833)
- `bmb_string_new`, `bmb_string_len`, `bmb_string_concat`, etc.
- `bmb_array_new`, `bmb_array_push`, `bmb_array_get`, etc.
- `bmb_sb_new`, `bmb_sb_push`, `bmb_sb_build`, etc.

Any user function matching `<runtime_function_name>` after `bmb_` prefix will collide.

## Known Collision-Prone Names
- `parse_int` → `bmb_parse_int`
- `string_new` → `bmb_string_new`
- `string_len` → `bmb_string_len`
- `array_new` → `bmb_array_new`
- `array_push` → `bmb_array_push`
- `sb_new` → `bmb_sb_new`
- And many more

## Proposed Solutions

### Option A: Mangle with module path (Recommended)
Prefix user functions with module path: `bmb_<module>_<function>`
e.g., `bmb_json__parse_int` or `bmb_mod_json_parse_int`

### Option B: Use different prefix for runtime
Change runtime functions from `bmb_*` to `__bmb_rt_*` to avoid user namespace.

### Option C: Detect and error on collision
At link time or IR generation, check for collisions with known runtime symbols.

## Workaround
Prefix all user functions with a unique module name (e.g., `json_parse_int` instead of `parse_int`).

## Discovered During
Cycle 1293 — bmb-json parser development. Took 23 debug test files to isolate.

## Related Files
- `bmb/runtime/bmb_runtime.c` — Runtime function definitions
- `bootstrap/llvm_ir.bmb` — LLVM IR generation (name mangling)

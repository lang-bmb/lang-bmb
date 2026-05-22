# Cycle 3034: M6 Prerequisites Audit + Stale Docs Fix
Date: 2026-05-22

## Re-plan
HANDOFF said M6-P1 needs "HTTP + JSON". Reading server.py directly revealed the MCP server uses **stdio JSON-RPC** (FastMCP default transport), not HTTP. SCOPE ADJUST: audit focus shifted from HTTP stdlib assessment to stdio framing + JSON stdlib sufficiency.

## Scope & Implementation

**Audit-only cycle** — no new language/compiler changes. One defect fixed in server.py.

### A) JSON stdlib assessment
`stdlib/json/mod.bmb` (337 lines): full recursive JSON parser + serializer. Public API:
- `jdoc_new() -> i64`, `jparse(doc, src) -> i64`, `jstringify(doc, n, src) -> String`
- `jobj_get/str/num`, `jarr_len/get`, `jtype/jbool/jnum/jstr`
- **Limitation**: integer numbers only (no floats). JSON-RPC 2.0 integer IDs are fine.
- **VERDICT**: ✅ Sufficient for JSON-RPC 2.0

### B) stdio framing assessment
MCP protocol uses LSP-style `Content-Length: N\r\n\r\n{body}` framing.
- `read_line() -> String` — reads one line (strips newline), native ✅
- `read_bytes(n: i64) -> String` — reads exactly N bytes, native ✅ (Cycle 3033 fix)
- `write_stdout(s: String) -> ()` — raw stdout write (no newline), native ✅
- **VERDICT**: ✅ Full stdio framing implementable in BMB

### C) Subprocess execution assessment
Tools in bmb-mcp invoke the `bmb` CLI and return results.
- `exec_output(cmd, args) -> String` — captures stdout only
- `system_capture(cmd) -> String` — popen-based, stdout only
- **Gap**: No stderr capture. Python bmb_cli.py captures both. Mitigation: append ` 2>&1` to system_capture calls.
- **VERDICT**: ✅ Feasible (stderr merged with stdout via 2>&1)

### D) Scripts/ port assessment
scripts/ contains 40+ files, many 200-700 lines. Most use pipes/grep/awk/sed/bc — shell primitives not available in BMB as language features. Only simple scripts (rebuild-runtime.sh ~130 lines, rebuild-bootstrap-exe.sh ~130 lines) are realistic targets. Complex orchestration scripts (benchmark.sh, full-cycle.sh) are not portable without significant new BMB stdlib.
- **VERDICT**: Scripts/ port is lower-priority than HANDOFF suggested. 2-3 scripts max, not "1-2 cycles."

### E) Defect fix: stale _QUICK_REFERENCE in server.py
Lines 806-808 said tuple destructuring, `_` wildcard, and `::` static calls are NOT supported — all three are wrong (added Cycles 2620-2621, wildcard always supported, enum variants Cycle 2633). Fixed with accurate information.

## Verification & Defect Resolution
No code compilation changes — no cargo test needed. Verified the edit looks correct.

## Reflection
- **Scope fit**: Audit complete, all four prerequisite areas assessed.
- **Latent defects**: The stale _QUICK_REFERENCE was actively harming AI-assisted BMB development — fixed.
- **Structural improvements**: The `stdlib/json/mod.bmb` pre-allocated capacity (arrays: 64, objects: 32) may be limiting for large JSON responses, but MCP tool responses are bounded.
- **Philosophy drift**: None — pure discovery work aligned with M6 goal.
- **Roadmap impact**: HANDOFF estimate "2-3 cycles" for bmb-mcp is optimistic given 13 tools. Revise to 4-6 cycles. Scripts/ re-scoped to 2-3 simple scripts only.

## Carry-Forward
- Actionable: None (audit complete, defect fixed)
- Structural Improvement Proposals: stdlib/json may need larger pre-allocated capacities for nested MCP responses — monitor during implementation
- Pending Human Decisions: None
- Roadmap Revisions: bmb-mcp estimate revised to 4-6 cycles; scripts/ reduced to "simple subset only (2-3 cycles)"
- Next Recommendation: Cycle 3035 — start M6-P1 bmb-mcp BMB scaffold (stdio loop + JSON-RPC dispatch + 2-3 core tools)

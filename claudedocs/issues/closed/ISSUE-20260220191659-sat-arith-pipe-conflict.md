# Saturating Arithmetic Pipe Delimiter Conflict

**Status: RESOLVED — Separator already uses byte 31 (\x1F), not byte 124 (|). Confirmed in Cycle 1549.**

## Summary
The MIR optimizer uses `|` (pipe, char 124) as a line separator throughout its pipe-separated MIR storage format. The saturating arithmetic operators `+|`, `-|`, `*|` contain `|` in their operator names, causing the optimizer to incorrectly split MIR lines containing these operators.

## Impact
- **Severity**: Medium — affects any BMB function using saturating arithmetic when compiled with the bootstrap compiler
- **Scope**: Latent bug — compiler.bmb doesn't use saturating arithmetic, so it doesn't currently manifest in bootstrapping
- **Manifestation**: MIR line `  %_t0 = +| %a, %b` is split into `  %_t0 = +` and ` %a, %b`, corrupting the MIR representation

## Affected Files
- `bootstrap/optimize.bmb` — All functions that process pipe-separated MIR (100+ uses of `find_char(mir, 124, pos)`)
- `bootstrap/llvm_ir.bmb` — MIR-to-LLVM-IR translation also uses pipe-separated format

## Root Cause
- MIR text format uses `+|` for saturating add (defined in `bmb/src/mir/mod.rs:1551`)
- Optimizer uses `|` as line separator (established early in optimizer development)
- These two uses of `|` conflict

## Possible Solutions

### Option 1: Change Optimizer Line Separator (Preferred)
- Change from `|` to a safe character (e.g., `\x01`, `\x1F` unit separator, or `\n` newline)
- **Pros**: Clean fix, no changes to MIR format
- **Cons**: Requires updating 100+ occurrences across optimize.bmb and llvm_ir.bmb

### Option 2: Change MIR Operator Names
- Use different names for saturating ops (e.g., `sadd`, `ssub`, `smul`)
- **Pros**: Simple change in Rust MIR module + bootstrap
- **Cons**: Violates Rust freeze policy (requires bmb/src/mir/mod.rs change)

### Option 3: Escape Pipe in MIR Lines
- Escape `|` in operator names when serializing to pipe-separated format
- **Pros**: No format changes
- **Cons**: Complex, error-prone, performance impact from escaping/unescaping

## Recommendation
Option 1 is the cleanest fix. Use `\x1F` (unit separator, char 31) as line separator — this ASCII control character cannot appear in any MIR text content.

## Related
- Wrapping arithmetic (`+%`, `-%`, `*%`) had a similar but less severe issue — `%` in operator names confused operand extraction. Fixed in Cycle 1316 with dedicated `extract_wrap_left`/`extract_wrap_right`.
- Discovered during Cycle 1318 investigation of saturating arithmetic constant folding.

# Issue: String Escape Sequences Needed for Optimal String Building

## Summary
BMB lacks string escape sequences (`\n`, `\t`, `\"`, `\\`), forcing inefficient runtime patterns for string building with special characters.

## Current Impact
- `json_serialize` benchmark: 165% of C (was 191%, improved with `sb_push_char`)
- Cannot write `"\"name\":"` directly - must use `chr(34) + "name" + chr(34) + ":"`
- Each `chr()` call creates a new BmbString allocation
- Even `sb_push_char` allocates 2 bytes per character

## Comparison

### C (optimal)
```c
buf_str("\"name\":");  // Single operation, zero allocation
```

### BMB (current)
```bmb
let u1 = sb_push_char(sb, 34);  // " - allocates 2 bytes
let u2 = sb_push(sb, "name");
let u3 = sb_push_char(sb, 34);  // " - allocates 2 bytes
let u4 = sb_push_char(sb, 58);  // : - allocates 2 bytes
// 4 allocations + 4 function calls for what C does in 1
```

### BMB (with escape sequences)
```bmb
let u = sb_push(sb, "\"name\":");  // 1 function call, string interned
```

## Proposed Solution

### 1. Add Escape Sequences to Lexer/Parser
Support standard escape sequences in string literals:
- `\"` - double quote (34)
- `\\` - backslash (92)
- `\n` - newline (10)
- `\r` - carriage return (13)
- `\t` - tab (9)
- `\0` - null (0)
- `\xHH` - hex escape

### 2. Files to Modify
- `bmb/src/lexer/mod.rs` - Handle escape sequences during tokenization
- `bmb/src/parser/grammar.lalrpop` - No changes needed if lexer handles it
- `docs/LANGUAGE_REFERENCE.md` - Document the feature

### 3. Expected Performance Impact
With escape sequences:
- `json_serialize` should reach ~100-110% of C
- Eliminates per-character allocations for known strings
- Enables string interning for commonly used patterns

## Current Workarounds Applied
- Added `sb_push_char(handle, char_code)` to runtime and compiler
- Used batched string slices for escape-free portions
- **Current: 239% of C** (with proper escape testing workload)

## Root Cause Analysis

The performance gap is due to multiple fundamental issues:

1. **No native loops** - BMB uses recursion instead of loops
   - Each iteration: function call overhead + stack frame
   - C: `while (*s) { buffer[pos++] = c; }` - zero overhead

2. **No mutable buffers** - Every character write allocates
   - `sb_push_char` does `malloc(2)` per character
   - C: Direct index write to static buffer

3. **No string escape sequences** - Runtime overhead for special chars
   - Must build `"Alice " + chr(34) + "..."` at runtime
   - C: `"Alice \"The Great\""` - compile-time constant

## Required Language Changes (Choose at least one)

| Change | Impact | Complexity |
|--------|--------|------------|
| Add string escape sequences | ~50% improvement | Medium |
| Add while/loop construct | ~30% improvement | High (parser, AST, MIR, codegen) |
| Add mutable byte buffer type | ~40% improvement | High (type system, codegen) |

## Priority
**P0** - Blocking BMB from matching C on string-heavy workloads
- Affects all string-heavy code
- Prevents BMB from matching C performance on string operations
- Already documented in LANGUAGE_REFERENCE.md as a known limitation

## Related
- `docs/LANGUAGE_REFERENCE.md:122` - Documents the limitation
- `ecosystem/benchmark-bmb/benches/real_world/json_serialize/` - Affected benchmark

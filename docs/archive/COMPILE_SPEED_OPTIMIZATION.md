# BMB Bootstrap Compiler Speed Optimization Plan

## Current Performance (v0.60.246)

| Compiler | Before | After | Improvement |
|----------|--------|-------|-------------|
| Rust BMB | 0.168s | 0.169s | baseline |
| Stage 2 (Bootstrap→Bootstrap) | 3.51s | **1.15s** | **3.0x faster** |

**Target**: ~~Reduce Stage 2 compilation time to < 1.2s~~ **ACHIEVED** (1.15s)

### Key Optimizations Applied (v0.60.246)

1. **escape_field/unescape_field**: Converted from `acc + chr(c)` to StringBuilder
   - Impact: 63% reduction in Stage 2 time (3.08s → 1.17s)

2. **escape_parens/unescape_parens**: Same StringBuilder conversion
   - Impact: 31% reduction in Stage 3 time (3.79s → 2.62s)

3. **parse_program**: StringBuilder for function accumulation
   - Impact: Minimal (not the bottleneck)

4. **collect_strings**: Avoid sb_build per iteration
   - Impact: Minimal

5. **keyword_or_ident**: Length-based dispatch
   - Impact: Minimal

6. **Runtime strmap support**: Added string-key hashmap to runtime
   - Available for future O(1) marker lookups (Task #11)

---

## Completed Tasks

| Task | Status | Impact |
|------|--------|--------|
| #9 Parser StringBuilder | ✓ Completed | Minimal |
| #10 String collection | ✓ Completed | Minimal |
| #12 Trampoline optimization | ✓ Completed | escape_field fix was key |
| #13 Keyword dispatch | ✓ Completed | Minimal |
| #14 Runtime hashmap (strmap) | ✓ Completed | Infrastructure ready |

## Remaining Tasks

| Task | Status | Priority | Notes |
|------|--------|----------|-------|
| #11 Marker O(1) lookup | Pending | P2 | Requires ~30 function signature changes |

---

## Root Cause Analysis

### Resolved: escape_field O(n²)

The **actual** bottleneck was the `escape_field` and `unescape_field` functions:

```bmb
// OLD: O(n²) - acc + chr(c) creates new string each iteration
fn escape_field(s: String, pos: i64, acc: String) -> String =
    if pos >= s.len() { acc }
    else { escape_field(s, pos + 1, acc + chr(c)) }

// NEW: O(n) - StringBuilder accumulates in-place
fn escape_field(s: String) -> String =
    let sb = sb_new();
    let _ = escape_field_sb(s, 0, sb);
    sb_build(sb);
```

This single change resulted in 63% reduction in compile time.

### P2: Marker Lookup (Deferred)

**Location**: `find_marker_from_end()`, `is_string_var_fast()`

Currently scans up to 100 comma-separated entries per lookup. With `strmap` runtime support now available, this could be converted to O(1) lookup, but requires changing ~30 function signatures.

**Decision**: Deferred due to:
1. Current performance (1.15s) already meets target (<1.2s)
2. High refactoring complexity
3. Diminishing returns expected

---

## Verification Results (v0.60.246)

```
======================================
Bootstrap Status Summary
======================================
Stage 1 (Rust BMB → BMB₁):     true (2011ms)
Stage 2 (BMB₁ → LLVM IR):      true (1154ms)
Stage 3 (BMB₂ → LLVM IR):      true (2630ms)
Fixed Point (S2 == S3):        true
Total Time:                    6152ms
```

All bootstrap tests pass. Fixed point verified.

---

## Success Metrics (Final)

| Metric | Before | Target | Achieved |
|--------|--------|--------|----------|
| Stage 2 time | 3.51s | < 1.2s | **1.15s** ✓ |
| Rust BMB ratio | 21x | < 7x | **6.8x** ✓ |
| Fixed point | ✓ | ✓ | ✓ |

---

## Technical Notes

### StringBuilder Pattern

The key insight: BMB's string concatenation creates a new string each time. StringBuilder amortizes this to O(1) per append.

```bmb
// Pattern for converting O(n²) accumulation to O(n)
fn old_function(s: String, pos: i64, acc: String) -> String =
    if pos >= s.len() { acc }
    else { old_function(s, pos + 1, acc + something) }

// Convert to:
fn new_function(s: String) -> String =
    let sb = sb_new();
    let _ = new_function_sb(s, 0, sb);
    sb_build(sb);

fn new_function_sb(s: String, pos: i64, sb: i64) -> i64 =
    if pos >= s.len() { 0 }
    else {
        let _ = sb_push(sb, something);
        new_function_sb(s, pos + 1, sb)
    };
```

### Runtime strmap Functions (v0.60.246)

Added string-key hashmap support:

```bmb
let map = strmap_new();              // Create new map
let _ = strmap_insert(map, "key", 100);  // Insert key-value
let v = strmap_get(map, "key");      // Get value (-1 if not found)
let has = strmap_contains(map, "key"); // Check existence (0/1)
let sz = strmap_size(map);           // Get size
```

Uses FNV-1a hashing with chained collision resolution.

---

## References

- `bootstrap/compiler.bmb` - Main compiler source
- `bmb/runtime/bmb_runtime.c` - Runtime library with strmap
- `docs/BOOTSTRAP_BENCHMARK.md` - Bootstrap process documentation

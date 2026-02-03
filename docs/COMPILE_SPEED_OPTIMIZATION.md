# BMB Bootstrap Compiler Speed Optimization Plan

## Current Performance (v0.60.245)

| Compiler | Before | After | Improvement |
|----------|--------|-------|-------------|
| Rust BMB | 0.168s | 0.167s | baseline |
| Stage 1 (Rust→Bootstrap) | 3.05s | **1.12s** | **2.7x faster** |
| Stage 2 (Bootstrap→Bootstrap) | 3.51s | **1.17s** | **3.0x faster** |

**Target**: ~~Reduce Stage 1/2 compilation time to < 1s (< 6x of Rust)~~ **ACHIEVED** (6.7x)

### Key Optimizations Applied (v0.60.245)

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

---

## Root Cause Analysis

### P0: Critical Bottlenecks (Must Fix)

#### 1. Parser O(N²) Accumulation
**Location**: `parse_program()` (line ~745)

```bmb
// Current: O(N²) string concatenation
fn parse_program(src, ..., fns: String) =
    parse_program(src, ..., fns + " " + unpack_ast(rf))
```

**Impact**: 462 functions → 213,444 string allocations
**Solution**: Use StringBuilder for O(1) amortized accumulation

#### 2. String Collection O(n²) Scanning
**Location**: `find_string_in_mir()` (line ~2650)

```bmb
// Current: O(n) scan per string literal
fn find_string_in_mir(mir: String, pos: i64) -> i64 =
    if matches_string_pattern(mir, pos) { pos }
    else { find_string_in_mir(mir, pos + 1) }
```

**Impact**: O(n²) for MIR with many string literals
**Solution**: Single-pass collection of all string positions

---

### P1: Medium Priority

#### 3. Marker Lookup O(100)
**Location**: `find_marker_from_end()` (line ~3092)

Currently scans up to 100 comma-separated entries per lookup.

**Solution**:
- Option A: Bloom filter for fast negative check
- Option B: Binary search on sorted list
- Option C: Runtime hashmap (requires runtime support)

#### 4. Trampoline Work Stack Parsing
**Location**: `trampoline_v2()` (line ~1054)

String-encoded work stack requires parsing on every step.

**Solution**: Binary/integer encoding or runtime stack support

---

### P2: Low Priority (Nice to Have)

#### 5. Keyword Matching
**Location**: `keyword_or_ident()` (line ~171)

25-way if-else chain for keyword matching.

**Solution**: Length-based dispatch to reduce average comparisons

---

## Implementation Roadmap

### Phase 1: Quick Wins (Est. 2-3x improvement)

| Task | Effort | Impact | Dependencies |
|------|--------|--------|--------------|
| #9 Parser StringBuilder | Medium | High | None |
| #10 Single-pass string collection | Medium | High | None |

**Expected result**: Stage 1 time 3.0s → 1.0-1.5s

### Phase 2: Runtime Enhancement (Est. 3-5x improvement)

| Task | Effort | Impact | Dependencies |
|------|--------|--------|--------------|
| #14 Runtime hashmap | High | Very High | None |
| #11 Marker lookup with hashmap | Low | Medium | #14 |

**Expected result**: Stage 1 time 1.0s → 0.3-0.5s

### Phase 3: Polish (Est. 10-20% improvement)

| Task | Effort | Impact | Dependencies |
|------|--------|--------|--------------|
| #12 Trampoline optimization | Medium | Medium | #14 optional |
| #13 Keyword dispatch | Low | Low | None |

**Expected result**: Stage 1 time 0.3s → 0.25-0.3s

---

## Verification Plan

### After Each Change

```bash
# 1. Correctness check
./target/x86_64-pc-windows-gnu/release/bmb.exe run bootstrap/selfhost_test.bmb
# Expected: 999

# 2. 3-stage bootstrap
bash scripts/bootstrap.sh --stage1-only
# Expected: Stage 1 success

# 3. Performance measurement
time ./target/bootstrap/bmb-stage1.exe bootstrap/compiler.bmb /dev/null
# Record: real time
```

### Before Merge

```bash
# Full 3-stage with fixed point
bash scripts/bootstrap.sh
# Expected: Fixed Point (S2 == S3): true
```

---

## Technical Notes

### StringBuilder in BMB

The bootstrap compiler already uses StringBuilder for LLVM IR generation:

```bmb
let sb = sb_new();
let _ = sb_push(sb, "define i64 @");
let _ = sb_push(sb, fn_name);
// ...
let result = sb_build(sb);
```

This pattern should be extended to parser and string collection.

### Hashmap Considerations

BMB doesn't have native hashmap support. Options:

1. **Runtime C function**: Add to `bmb_runtime.c`
2. **String-encoded trie**: Pure BMB, but complex
3. **Sorted list + binary search**: Simpler, O(log n)

Recommendation: Add runtime hashmap for maximum impact.

### Work Stack Encoding

Current string-based encoding:
```
"ITEM1\tITEM2\tITEM3"
```

Possible integer-based encoding:
```
// Encode: type * 1000000 + position
// Decode: type = val / 1000000, pos = val % 1000000
```

This eliminates string parsing overhead in the hot loop.

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| StringBuilder breaks parser | Incremental refactoring with tests |
| Hashmap memory leaks | Explicit cleanup in compiler main |
| Performance regression | Benchmark before/after each change |
| Bootstrap breakage | 3-stage verification after each change |

---

## Success Metrics

| Metric | Current | Target | Stretch |
|--------|---------|--------|---------|
| Stage 1 time | 3.05s | < 1.0s | < 0.5s |
| Stage 2 time | 3.51s | < 1.2s | < 0.6s |
| Rust BMB ratio | 18-21x | < 6x | < 3x |
| Fixed point | ✓ | ✓ | ✓ |

---

## References

- `bootstrap/compiler.bmb` - Main compiler source
- `bmb/runtime/bmb_runtime.c` - Runtime library
- `docs/BOOTSTRAP_BENCHMARK.md` - Bootstrap process documentation
- Previous optimizations: v0.60.230 (trampoline), v0.60.238 (parser depth)

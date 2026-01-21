# BMB Performance Issues - v0.50.72

## Critical Issues (P1)

### ISSUE-001: syscall_overhead 3.76x slower than C
- **Benchmark**: `benches/syscall/syscall_overhead`
- **Current**: BMB 618ms vs C 164ms
- **Root Cause**: BmbString wrapper overhead for every file_exists call
- **Impact**: All FFI-heavy code with string parameters

**Technical Details**:
```c
// C - direct char* to stat()
int64_t check_exists(const char* path) {
    struct stat st;
    return (stat(path, &st) == 0) ? 1 : 0;
}

// BMB - BmbString wrapper overhead
fn file_exists(path: String) -> i64  // String = BmbString*
  // 1. Load path->data (indirection)
  // 2. Call stat(path->data, &st)
```

**Fix Options**:
| Option | Complexity | Impact | Recommendation |
|--------|------------|--------|----------------|
| A. Inline bmb_string_from_cstr for literals | Low | Medium | ✅ Do first |
| B. Add `cstr` type for FFI | Medium | High | For v0.51 |
| C. Small String Optimization | High | Medium | For v0.52 |

---

### ISSUE-002: fannkuch 1.76x slower than C
- **Benchmark**: `benches/compute/fannkuch`
- **Current**: BMB 187ms vs C 106ms
- **Root Cause**: Recursive permutation algorithm vs C's iterative

**Technical Details**:
```
C uses nested for loops:
  for (int i = 0; i < n; i++)
    for (int j = 0; j < k; j++)

BMB uses recursion:
  fn permute(arr, k) = if k == 0 { ... } else { permute(arr, k-1) }
```

**Fix Options**:
| Option | Complexity | Impact | Recommendation |
|--------|------------|--------|----------------|
| A. Rewrite benchmark with while loops | Low | High | ✅ Do first |
| B. Add function inlining pass | Medium | Medium | For v0.51 |
| C. Improve TCO for this pattern | High | Medium | Already done |

---

## Major Issues (P2)

### ISSUE-003: http_parse 1.56x slower than C
- **Benchmark**: `benches/real_world/http_parse`
- **Current**: BMB 23ms vs C 15ms
- **Root Cause**: String concatenation creates allocations

**Technical Details**:
```bmb
// Each + creates a new allocation
let request = "GET " + path + " HTTP/1.1" + crlf() +
              "Host: " + host + crlf() + crlf();
```

**Fix**: Implement string interpolation or optimize concat chain

---

### ISSUE-004: matrix_multiply 1.44x slower than C
- **Benchmark**: `benches/surpass/matrix_multiply`
- **Current**: BMB 5.8ms vs C 4.0ms
- **Root Cause**: Array indexing overhead in triple-nested loop

**Fix**: Better bounds check elimination for provably-safe indices

---

### ISSUE-005: json_serialize 1.35x slower than C
- **Benchmark**: `benches/real_world/json_serialize`
- **Current**: BMB 29ms vs C 21ms
- **Root Cause**: O(n²) string building pattern

```bmb
fn escape_loop(s, pos, acc) =
  if pos >= len(s) { acc }
  else { escape_loop(s, pos+1, acc + escape_char(s[pos])) }
  //                          ^^ creates new string each iteration!
```

**Fix**: Add StringBuilder or use array buffer

---

## Minor Issues (P3)

### ISSUE-006: fibonacci 1.33x
- **Expected**: Non-tail recursive algorithm, cannot optimize
- **Action**: Document as algorithm limitation, not compiler issue

### ISSUE-007: null_elim 1.23x
- **Root Cause**: Contract checking not fully eliminated
- **Fix**: Improve ContractUnreachableElimination pass

### ISSUE-008: json_parse 1.16x
- **Root Cause**: String tokenization overhead
- **Fix**: Same as http_parse - string optimization

### ISSUE-009: reverse-complement 1.16x
- **Root Cause**: Bioinformatics string processing
- **Fix**: String buffer optimization

### ISSUE-010: branch_elim (contract_opt) 1.15x
- **Root Cause**: Branch hints not optimal
- **Fix**: Improve SimplifyBranches pass

### ISSUE-011: pointer_chase 1.15x
- **Root Cause**: Pointer indirection in linked list
- **Fix**: May require profile-guided optimization

### ISSUE-012: cache_stride 1.11x
- **Root Cause**: Cache-unfriendly access pattern
- **Fix**: Loop tiling or prefetch hints

---

## Improvement Roadmap

### v0.50.73 Analysis Results

**ISSUE-002: fannkuch while loops**
- ❌ BLOCKED: While loop grammar too restrictive
- Pattern `while cond { { let x = ...; assignment; () } }` fails to parse
- Root cause: grammar.lalrpop while body accepts only SpannedExpr, not BlockStmt sequence
- **Action**: Grammar fix required (v0.51 scope)
- **Current**: 1.73x C (acceptable)

**ISSUE-001: syscall_overhead FFI**
- ⚠️ ROOT CAUSE: BmbString* wrapper overhead
- String literal created once before loop (correct)
- Overhead is `file_exists(BmbString*)` vs C's `stat(char*)`
- **Fix needed**: Direct char* FFI for string literal arguments
- **Complexity**: Medium (codegen change)
- **Action**: Defer to v0.51

### v0.51.x (Short-term)
- [ ] **ISSUE-001**: Direct char* FFI for string literals (syscall_overhead)
- [ ] **ISSUE-002**: While loop grammar extension (fannkuch)
- [ ] ISSUE-003: String interpolation
- [ ] ISSUE-004: Bounds check elimination improvement
- [ ] ISSUE-005: StringBuilder optimization
- [ ] ISSUE-007: Contract elimination improvement

### v0.52.x (Long-term)
- [ ] ISSUE-001: Add `cstr` FFI type
- [ ] ISSUE-010: Branch prediction hints
- [ ] ISSUE-011: Profile-guided optimization

---

## Regression Watchlist

These benchmarks are close to the 1.10x threshold:

| Benchmark | Ratio | Risk |
|-----------|-------|------|
| graph_traversal | 1.10x | May regress |
| sort_presorted | 1.10x | May regress |
| mandelbrot | 1.08x | Low risk |
| aliasing_proof | 1.07x | Low risk |

---

*Generated: 2026-01-21*

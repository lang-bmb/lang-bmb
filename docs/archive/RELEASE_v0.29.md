# BMB v0.29 "Velocity" Release Notes

**Release Date**: 2026-01-04
**Theme**: Performance optimization and code quality

## Overview

v0.29 "Velocity" focused on MIR optimization and bootstrap code quality improvements. This release establishes a solid foundation for the upcoming v0.30 "Pure" self-hosting milestone.

## Major Features

### v0.29.1: MIR Optimization Framework

Implemented 6 optimization passes in the bootstrap:

| Pass | Description | Benefit |
|------|-------------|---------|
| Constant Folding | Evaluate compile-time constants | Reduced runtime computation |
| Dead Code Elimination | Remove unreachable code | Smaller binaries |
| Copy Propagation | Eliminate redundant copies | Fewer instructions |
| Common Subexpression | Reuse computed values | Reduced redundancy |
| Strength Reduction | Replace expensive ops | Faster execution |
| Block Merging | Combine basic blocks | Simpler control flow |

### v0.29.2: Bug Fixes

- Fixed separator conflict in optimize.bmb pattern matching
- Improved pattern stability in optimization passes

### v0.29.3-v0.29.6: Bootstrap Utilities Consolidation

Created `utils.bmb` with 10 sections of shared utilities:

1. **Character Classification** (6 functions)
   - `is_digit`, `is_alpha`, `is_alnum`, `is_ident_start`, `is_whitespace`, `is_alnum_or_underscore`

2. **Character Conversion** (6 functions)
   - `digit_to_int`, `digit_char`, `int_to_string`, `char_upper`, `char_lower`, `char_to_string`

3. **Integer Parsing** (4 functions)
   - `parse_int`, `parse_int_acc`, `parse_int_signed`, `parse_int_end`

4. **String Searching** (10 functions)
   - `find_char`, `find_pipe`, `find_colon`, `find_semicolon`, `find_comma`, `find_space`, `find_newline`
   - `skip_ws`, `skip_spaces`, `skip_to_eol`, `find_ident_end`, `find_number_end`

5. **String Matching** (3 functions)
   - `starts_with`, `starts_with_at`, `find_pattern`

6. **String Extraction** (2 functions)
   - `read_until_ws`, `strip_trailing_colon`

7. **Comment Handling** (3 functions)
   - `skip_comment`, `skip_to_eol`, `skip_all`

8. **Error Handling** (4 functions)
   - `is_error`, `make_error`, `get_error_msg`, `is_error_loose`

9. **Result Packing** (6 functions)
   - `pack_result`, `unpack_pos`, `unpack_ast`
   - `pack_values`, `pack_values3`, `unpack_first`, `unpack_rest`

10. **Token Encoding** (5 functions)
    - `tok_encode`, `tok_kind`, `tok_end`, `tok_is_kind`, `tok_is_eof`

## Code Duplication Reduction

| Function | Files Consolidated |
|----------|-------------------|
| `is_digit` | 11 files |
| `is_alpha` | 10 files |
| `parse_int` | 10 files |
| `pack_result` | 8 files |
| `unpack_pos/ast` | 7 files |
| `is_error` | 6 files |
| `tok_kind/tok_end` | 6 files |
| `find_ident_end` | 6 files |
| `starts_with` | 6 files |

## Statistics

### Bootstrap Growth
| Metric | v0.28 | v0.29 | Change |
|--------|-------|-------|--------|
| Total LOC | ~9,200 | 9,924 | +724 |
| Coverage | 42% | 46% | +4% |
| Test Count | ~320 | 353 | +33 |

### utils.bmb
- **Size**: 21KB, 521 LOC
- **Tests**: 33 (all passing)
- **Sections**: 10

## API Changes

### New APIs in utils.bmb

```bmb
-- Error handling (standard format: "ERR:message")
fn is_error(s: String) -> bool
fn make_error(msg: String) -> String
fn get_error_msg(s: String) -> String

-- Result packing (format: "pos:value")
fn pack_result(pos: i64, value: String) -> String
fn unpack_pos(result: String) -> i64
fn unpack_ast(result: String) -> String

-- Token encoding (format: kind * 1000000 + end_pos)
fn tok_encode(kind: i64, end_pos: i64) -> i64
fn tok_kind(tok: i64) -> i64
fn tok_end(tok: i64) -> i64
```

## Testing

All tests pass:
```
✅ utils.bmb: 33 tests
✅ llvm_ir.bmb: 119 assertions
✅ lowering.bmb: 52 assertions
✅ mir.bmb: 46 assertions
✅ types.bmb: 45 assertions
✅ optimize.bmb: 9 tests
✅ selfhost_equiv.bmb: 19 tests
✅ parser_test.bmb: 15 categories
✅ pipeline.bmb: 14 tests
✅ selfhost_test.bmb: 8 tests
✅ compiler.bmb: 8 tests
```

## Breaking Changes

None. v0.29 is fully backward compatible.

## Known Limitations

- utils.bmb functions must be copied to other modules (no import system yet)
- Generic types not supported in bootstrap
- No trait implementation in bootstrap

## Migration Guide

No migration required. Existing code continues to work.

For new code, prefer using the standardized patterns from utils.bmb:
- Use `is_error()`/`make_error()` instead of custom error checks
- Use `pack_result()`/`unpack_pos()`/`unpack_ast()` for result handling
- Use `tok_kind()`/`tok_end()` for token operations

## Next Steps (v0.30)

v0.30 "Pure" will focus on self-hosting completion:
1. Add generics to bootstrap type checker
2. Add trait support to bootstrap
3. Implement bootstrap interpreter
4. Port remaining Rust code to BMB

---

**Contributors**: BMB Development Team
**Documentation**: See [GAP_ANALYSIS.md](./GAP_ANALYSIS.md) for v0.30 planning

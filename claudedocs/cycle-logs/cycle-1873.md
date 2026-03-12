# Cycle 1873: Bootstrap nonnull + GEP nuw Completion

Date: 2026-03-12

## Inherited → Addressed
Cycle 1872: GEP `inbounds` added. Missing `nuw` and `nonnull` identified as remaining gaps.

## Scope & Implementation

### GEP `nuw` flag
- Added `nuw` to all `getelementptr inbounds` instructions (Cycle 1872 only added `inbounds`)
- Updated both `compiler.bmb` (10 sites) and `llvm_ir.bmb` (9 sites)
- 2,109/2,109 GEPs now have `inbounds nuw` (100% coverage)

### nonnull on allocating function returns
- Added `nonnull` to all string function return types (bmb_string_*)
- Added `nonnull` to conversion functions (bmb_chr, bmb_int_to_string, bmb_f64_to_string, bmb_to_hex/binary/octal, bmb_fast_i2s)
- Added `nonnull` to allocation functions (vec_new, vec_with_capacity, hashmap_new, str_hashmap_new)
- Added `nonnull` to I/O functions (bmb_read_file, bmb_read_line, bmb_getenv, bmb_getcwd)
- Added `nonnull` to builder functions (bmb_sb_build)
- Total: 40 nonnull annotations in bootstrap IR

### Files Changed
- `bootstrap/compiler.bmb` — GEP `nuw` + `nonnull` on 25+ runtime declarations
- `bootstrap/llvm_ir.bmb` — GEP `nuw` + test assertion updates

## Review & Resolution
- **Rust tests**: 6,186/6,186 PASS
- **Bootstrap**: 3-Stage Fixed Point VERIFIED (108,519 lines, S2 == S3)
- **IR quality**: 2,109 inbounds nuw GEPs, 40 nonnull, 1,450 noundef

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Early termination assessment — codegen is comprehensive, no benchmark suite available for validation

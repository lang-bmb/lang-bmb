# Cycle 1940: gotgan transitive deps + lock file E2E
Date: 2026-03-22

## Inherited → Addressed
- Cycle 1939: gotgan dep tree showed only 1 dep (cosmetic) → fixed

## Scope & Implementation
- Fixed `ProjectContext::find()` to use `resolver.build_order()` for transitive deps
  - Before: only direct deps from manifest
  - After: full transitive dependency chain in topological order
- Verified:
  - `gotgan tree --all`: shows pkg-base + pkg-mid (2 deps, correct order) ✅
  - `gotgan check`: resolves transitive deps through -I flags ✅
  - `gotgan update`: generates lock file with both deps ✅
  - Circular dependency detection: "Circular dependency detected: pkg-b" ✅

## Review & Resolution
- `cargo test --release`: 6,186 pass, 0 fail ✅
- pkg-top → pkg-mid → pkg-base chain fully working ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Cycle 1941 — documentation: Ownership & Borrowing tutorial

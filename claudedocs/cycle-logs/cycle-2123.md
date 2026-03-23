# Cycle 2123: End-to-end pip install test
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2121-2122: Version management complete, next was E2E test.

## Scope & Implementation
- Created fresh virtual environment
- Installed all 5 libraries via `pip install -e`
- Verified import + function calls work for all 5 libraries

### Bug Fixed
pyproject.toml had conflicting `[tool.setuptools.packages.find]` and `[tool.setuptools.package-dir]` directives. The `find` directive with `where = ["bindings/python"]` caused setuptools to not map the package correctly in editable installs.

**Fix**: Replaced `packages.find` with explicit `packages = ["bmb_<name>"]` in all 5 pyproject.toml files.

### E2E Results
```
bmb_algo: 0.2.0 - knapsack: 9
bmb_compute: 0.1.0 - sqrt(144): 12
bmb_crypto: 0.2.0 - sha256: 2cf24dba5fb0a30e...
bmb_text: 0.1.0 - kmp: 6
bmb_json: 0.1.0 - validate: True

All 5 libraries imported and working!
```

## Review & Resolution
- Fixed pyproject.toml package mapping for all 5 libraries
- Verified E2E: fresh venv → pip install -e → import → function calls all working

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Final summary + ROADMAP update (cycle 2124)

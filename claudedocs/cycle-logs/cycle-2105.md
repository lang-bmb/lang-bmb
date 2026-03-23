# Cycle 2105: Standardize pyproject.toml for all 5 libraries
Date: 2026-03-23

## Inherited -> Addressed
Cycle 2097-2104: Next recommendation was "PyPI wheel builds, Linux/macOS cross-platform, WASM bindings" — starting with packaging foundation.

## Scope & Implementation
- Created `pyproject.toml` (PEP 621) for all 5 libraries: bmb-algo, bmb-compute, bmb-crypto, bmb-text, bmb-json
- Created missing `setup.py` for bmb-compute and bmb-json (previously only had bindings, no packaging)
- Removed duplicate `setup.py` from bmb-algo/bindings/python/ (root-level is canonical)
- Fixed PEP 639 compliance: removed `License :: OSI Approved` classifiers (superseded by `license = "MIT"` in pyproject.toml)
- Removed duplicate `license` field from setup.py (pyproject.toml takes precedence)

## Review & Resolution
- All 5 libraries verified: `setup.py --version` returns correct version with no warnings
- Consistent structure: pyproject.toml + setup.py at root, bindings/python/ for Python code

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Add per-library test suites (cycle 2106)

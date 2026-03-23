# Cycles 2105-2124 Roadmap: Binding Ecosystem Maturation
Date: 2026-03-23

## Goal
Mature the 5 BMB binding libraries from "working prototype" to "pip-installable, well-tested, well-documented packages."

## Phase 1: Packaging & Testing (Cycles 2105-2110)
- **2105**: Standardize pyproject.toml for all 5 libraries (modern Python packaging)
- **2106**: Add per-library test suites (pytest-based, self-contained)
- **2107**: Add per-library benchmark scripts (vs Python stdlib)
- **2108**: Build script to compile all 5 DLLs from BMB source
- **2109**: Run all tests, fix any issues discovered
- **2110**: Add type stubs (.pyi) for all 5 libraries

## Phase 2: Documentation & Quality (Cycles 2111-2116)
- **2111**: Professional READMEs with full API docs for bmb-algo, bmb-compute
- **2112**: Professional READMEs for bmb-crypto, bmb-text, bmb-json
- **2113**: Add CHANGELOG.md + LICENSE for all 5 libraries
- **2114**: Add examples/ directory with usage examples for each library
- **2115**: Unified documentation: docs/BINDING_GUIDE.md (how to use BMB libraries from Python)
- **2116**: Quality audit — review all Python wrappers for consistency, error handling

## Phase 3: Build Infrastructure (Cycles 2117-2122)
- **2117**: Cross-platform build script (build.py) for Windows/Linux/macOS
- **2118**: GitHub Actions CI workflow for building and testing all libraries
- **2119**: Symbol filtering — ensure only @export symbols are visible
- **2120**: DLL size optimization — strip debug info, minimize runtime
- **2121**: Version management — sync versions across all libraries
- **2122**: Monorepo vs separate repo evaluation + packaging final pass

## Phase 4: Integration & Verification (Cycles 2123-2124)
- **2123**: End-to-end: fresh venv → pip install → import → test all 5 libs
- **2124**: Final BINDING_ROADMAP.md update + ROADMAP.md update + summary

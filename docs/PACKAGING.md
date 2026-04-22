# BMB Python Wheel Packaging

> Maintainer guide for building and publishing the five BMB binding libraries
> (`bmb-algo`, `bmb-compute`, `bmb-crypto`, `bmb-text`, `bmb-json`) to PyPI.
>
> End-user install docs live in `BINDING_GUIDE.md`. This document covers the
> publication side: how the wheels are built, why they're tagged the way they
> are, and how to run a full publish.

> **Status (2026-04-22, post-Cycle 2423)**: Defect 5 resolved and MinGW
> runtime dependency eliminated. `bmb build --shared` produces platform
> shared libraries on both backends (inkwell + text), and Windows output
> is now free of `libgcc_s_seh-1.dll` / `libwinpthread-1.dll` — only
> Windows 10+ system DLLs (`kernel32`, `ws2_32`, UCRT forwarders) are
> required. End-to-end wheel build + isolated-venv import verified
> locally on Windows. First `workflow_dispatch` of `pypi-publish.yml`
> will also validate Linux and macOS.

---

## What we ship

Each binding library is a Python package wrapping a **pre-built native shared
library** (`.dll` / `.so` / `.dylib`) loaded via `ctypes`. This means:

- The wheel is **platform-specific** — a Windows user cannot use a Linux `.so`.
- The wheel is **Python-version independent** — `ctypes` does not use the
  CPython ABI, so any Python 3.x on the matching OS works.

Wheel tag: `py3-none-<platform>` (e.g., `py3-none-win_amd64`,
`py3-none-linux_x86_64`, `py3-none-macosx_10_9_x86_64`).

### Why not `py3-none-any`?

A pure-Python wheel is platform-agnostic, but our wheels ship a `.dll` or `.so`
in `package_data`. If pip found a `py3-none-any` wheel first, a Linux user
would receive a Windows DLL — broken install. The `setup.py` shim forces the
correct platform tag; see next section.

### Why not `cp3XX-cp3XX-<platform>`?

`has_ext_modules=True` alone would tag the wheel as CPython-ABI-specific
(`cp312-cp312-win_amd64`). A user on Python 3.13 would not match. Since the
binary is `ctypes`-loaded and independent of the Python ABI, we override
the tag to `py3-none-<platform>`.

---

## Architecture

```
ecosystem/<lib>/
├── pyproject.toml      ← single source of truth: name, version, deps, metadata
├── setup.py            ← minimal shim: distclass + bdist_wheel tag override
├── bindings/python/    ← Python ctypes wrapper + .pyi stubs + bundled .dll/.so
├── src/lib.bmb         ← BMB source compiled to shared library
└── MANIFEST.in         ← wheel inclusion rules
```

### The `setup.py` shim

Every library's `setup.py` is ~30 lines with the same pattern:

```python
from setuptools import setup
from setuptools.dist import Distribution

try:
    from setuptools.command.bdist_wheel import bdist_wheel as _bdist_wheel
except ImportError:
    from wheel.bdist_wheel import bdist_wheel as _bdist_wheel


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


class bdist_wheel_platform(_bdist_wheel):
    def finalize_options(self):
        super().finalize_options()
        self.root_is_pure = False

    def get_tag(self):
        _, _, plat = super().get_tag()
        return "py3", "none", plat


setup(
    distclass=BinaryDistribution,
    cmdclass={"bdist_wheel": bdist_wheel_platform},
)
```

All metadata (name, version, description, classifiers, URLs, keywords) lives
in `pyproject.toml`. **Do not duplicate metadata in `setup.py`** — dual
source-of-truth has burned us before (see Cycle 2411 audit).

---

## Local build

```bash
# Full pipeline: rebuild compiler if needed → build .dll/.so/.dylib → build wheels
./scripts/build-wheel.sh

# Build one library only
./scripts/build-wheel.sh --lib bmb-algo

# Skip expensive steps when iterating on the shim itself
./scripts/build-wheel.sh --skip-compiler --skip-libs

# Run the full verification pipeline (twine check + pip install + import)
./scripts/build-wheel.sh --verify

# Preview without building anything
./scripts/build-wheel.sh --dry-run
```

Artifacts land in `dist/wheels/` (gitignored). The script exits non-zero if any
wheel is tagged `py3-none-any` — that would indicate the shim is not being
picked up (most likely someone edited `setup.py` away).

### Verification gate

`--verify` runs:

1. `twine check dist/wheels/*.whl` — rejects broken metadata, malformed
   long-description, missing README, etc.
2. `pip install` into an ephemeral venv (not the host Python).
3. `python -c "import <module>"` and print the public-function count.

If any step fails, the script exits 6 and the failing library is named in
the error message.

---

## CI workflow: `.github/workflows/pypi-publish.yml`

**Trigger**: `workflow_dispatch` only. No on-tag or on-push.

### Inputs

| Input | Default | Purpose |
|-------|---------|---------|
| `libraries` | `"all"` | Space-separated library names, or `"all"` |
| `publish` | `false` | Actually upload to PyPI (gated) |
| `repository` | `testpypi` | `testpypi` or `pypi` |

### Jobs

1. **`build-wheels`** — 3-way matrix (windows-latest, ubuntu-latest,
   macos-latest). Each runner installs the matching LLVM toolchain, builds the
   BMB compiler, invokes `scripts/build-wheel.sh --skip-compiler`, then runs
   the validation gate (tag check + `twine check` + install-import smoke
   test). Uploads platform-tagged artifacts.

2. **`publish`** (`if: inputs.publish == true`) — downloads all platform
   artifacts, merges into a single `dist/` directory, runs `twine upload`
   against the chosen repository.

### Publishing flow (first-time)

Before the first publish, configure the repository:

1. **Create deployment environments** in GitHub repo settings:
   - `testpypi` (no protection rules needed initially)
   - `pypi` (add reviewer-approval requirement recommended)
2. **Register PyPI tokens** (choose one per environment):
   - Set `TEST_PYPI_API_TOKEN` secret in `testpypi` environment
     (token from https://test.pypi.org/manage/account/token/).
   - Set `PYPI_API_TOKEN` secret in `pypi` environment
     (token from https://pypi.org/manage/account/token/).
   - Or configure trusted-publishing OIDC — the workflow already declares
     `permissions: id-token: write` in the publish job.
3. **Rehearsal run (strongly recommended)**:
   - Trigger `workflow_dispatch` with `publish=false`, `libraries=all`.
   - Inspect the artifact list; download one wheel per platform and confirm
     it unzips cleanly with the correct binary inside.
4. **TestPyPI dispatch**:
   - Trigger with `publish=true`, `repository=testpypi`.
   - Verify each library installs: `pip install --index-url
     https://test.pypi.org/simple/ bmb-algo` on a clean machine.
5. **PyPI dispatch** (production):
   - Only after (4) succeeds end-to-end.
   - Trigger with `publish=true`, `repository=pypi`.
   - `twine upload --skip-existing` is used, so re-running the same
     version is a no-op.

### Why `--skip-existing` on twine upload

A partial failure (e.g. one platform's wheel fails metadata check) should not
block a re-run from re-publishing the already-uploaded platforms. Combined
with the per-runner validation gate, the only way a bad wheel reaches PyPI
is if all three runners agree on a bad build — unlikely.

---

## Versioning

- Each library's version lives in its own `pyproject.toml` `[project].version`.
- Bump by editing that field; `setup.py` does not need changes.
- `bmb-algo` and `bmb-crypto` are at `0.3.0`; `bmb-compute`, `bmb-text`,
  `bmb-json` are at `0.2.0` (as of 2026-04-22).
- **Do not add version strings to `setup.py`** — it was a bug that caused
  Cycle 2411 to catch a `0.2.0` / `0.3.0` drift; the shim now has no version
  field at all.

---

## Troubleshooting

### "setup.py not found" / "no `pyproject.toml`"

`pip wheel .` must run inside `ecosystem/<lib>/`, not `ecosystem/`. Use
`scripts/build-wheel.sh` which handles this.

### Wheel tagged `py3-none-any`

The shim is not being picked up. Check that `ecosystem/<lib>/setup.py`
contains the `BinaryDistribution` + `bdist_wheel_platform` classes. A
stray `setup.cfg` overriding the cmdclass could also cause this — we
do not use `setup.cfg`.

### Wheel tagged `cp3XX-cp3XX-<plat>`

`has_ext_modules=True` is active but `get_tag()` override is not. Check
the `setup.py` shim — both classes must be present and passed via
`distclass=` and `cmdclass=`.

### `twine check` fails with "long_description has syntax errors"

The `README.md` referenced by `pyproject.toml` contains something `twine`
cannot render. Common cause: leading `#!` shebang line, or malformed
table markdown. Fix the README — `twine check` uses the same renderer as
PyPI would.

### Install succeeds but `import` fails

Most likely the `.dll` / `.so` was not built before `pip wheel`. The
script runs `python ecosystem/build_all.py` first, but if you used
`--skip-libs`, make sure the binary exists in `ecosystem/<lib>/bindings/python/`.

---

## Cross-platform notes

| Platform | Runner | Toolchain | Binary | Tag |
|----------|--------|-----------|--------|-----|
| Windows x86_64 | `windows-latest` | MinGW-w64 + LLVM via MSYS2 UCRT64 | `.dll` | `py3-none-win_amd64` |
| Linux x86_64 | `ubuntu-latest` | `llvm-21` apt package | `.so` | `py3-none-linux_x86_64` |
| macOS x86_64 (Intel) | `macos-13` | `brew install llvm` | `.dylib` | `py3-none-macosx_10_9_x86_64` |
| macOS arm64 (Apple Silicon) | `macos-latest` | `brew install llvm` | `.dylib` | `py3-none-macosx_11_0_arm64` |

> **Why `macos-13` for x86_64 and `macos-latest` for arm64**: GitHub's
> `macos-latest` / `macos-14` runners are Apple Silicon. Brew installs the
> arch-native LLVM, so cross-compiling `--target x86_64-apple-darwin` from
> arm64 breaks when `--features llvm` tries to link against arm64 llvm-sys.
> Pinning each arch to a native-arch runner avoids this class of failure.
> `macos-13` is the last Intel runner GitHub provides — if it is deprecated
> without a replacement, Intel macOS wheels may need to move to
> cross-compilation in a Docker container or drop off the matrix.

> As of 2026-04-22 only Windows has been exercised end-to-end locally; the
> Linux and both macOS runs will first occur via the CI matrix on
> `workflow_dispatch`.

### Windows runtime dependencies (resolved 2026-04-22, Cycle 2423)

Fresh `bmb build --shared` output on Windows previously depended on
MinGW runtime DLLs that ship only with MSYS2:

- `libgcc_s_seh-1.dll` — SEH exception unwinder
- `libwinpthread-1.dll` — POSIX threads shim (pulled transitively by msvcrt)

Cycle 2423 added `-static -static-libgcc` to both link paths
(`bmb/src/build/mod.rs` inkwell `link_native` + text-backend clang
block). After this change `objdump -p <lib>.dll` shows only Windows
system DLLs:

```
KERNEL32.dll
WS2_32.dll
api-ms-win-crt-*.dll   (UCRT forwarders, Windows 10+)
```

End-user `pip install bmb-*` on a stock Windows 10+ machine now works
without needing MSYS2 / MinGW pre-installed. Binary size delta:
+30-60 KB per .dll. No wheel bundling of MinGW DLLs required.

### Linux manylinux compatibility

PyPI rejects generic `linux_x86_64` wheels for broad compatibility —
production uploads should use `manylinux_2_28` or similar. The current
workflow uploads raw `linux_x86_64` as an interim; wheel repair with
`auditwheel repair` is a follow-up task.

---

## History

| Cycle | Change |
|-------|--------|
| ~1951 | Initial packaging: `pyproject.toml`, `.pyi` stubs, `MANIFEST.in` |
| 2411 | Platform-wheel tagging fix; `setup.py` → minimal shim; `setup.py` ↔ `pyproject.toml` version drift collapsed |
| 2412 | `scripts/build-wheel.sh` + `.github/workflows/pypi-publish.yml` |
| 2414 | `twine check` + install-import smoke test gate (CI + local parity via `--verify`) |
| 2415 | This document |
| 2416 | CI matrix: split macOS into `macos-13` (x86_64) + `macos-latest` (arm64) to avoid `--features llvm` cross-compile breakage |
| 2417 | `bindings-ci.yml` wheel gate — tag + `twine check` on every PR |
| 2418 | 🔴 Defect 5 discovered — `bmb build --shared` broken in all backends; prior wheel builds relied on stale pre-built `.dll` files from previous sessions. Cycle log `claudedocs/cycle-logs/cycle-2418.md`. |
| 2419 | ✅ Defect 5 fix pt.1 — user-side `@export` rename to avoid runtime symbol collision (`bmb_is_power_of_two` → `bmb_c_is_power_of_two`, `bmb_next_power_of_two` → `bmb_c_next_power_of_two`, `bmb_is_prime` → `bmb_algo_is_prime`). |
| 2420 | ✅ Defect 5 fix pt.2 — inkwell SharedLib link path, `@export` dllexport storage class, linkage priority (`@export` beats `always_inline` → `Private`). `bmb build --shared` works end-to-end. |

Subsequent cycle logs under `claudedocs/cycle-logs/` for rationale and
verification detail.

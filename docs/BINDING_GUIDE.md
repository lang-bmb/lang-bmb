# BMB Python Binding Libraries — User Guide

> Use blazing-fast BMB-compiled algorithms, crypto, text, JSON, and numeric functions from Python.

## Overview

BMB provides 5 Python libraries compiled from BMB source code. Each library is a shared library (.dll/.so/.dylib) with Python ctypes bindings.

| Library | Functions | Description |
|---------|-----------|-------------|
| **bmb-algo** | 55 | Algorithms: DP, graph, sort, search, number theory |
| **bmb-compute** | 33 | Math, statistics, random, vector operations |
| **bmb-crypto** | 14 | SHA-256, MD5, CRC32, HMAC, Base64/32, ROT13, Hex |
| **bmb-text** | 23 | String search (KMP), find/replace, case, analysis |
| **bmb-json** | 12 | JSON validate, parse, stringify, access, object |

## Installation

### From PyPI (when published)

```bash
pip install bmb-algo bmb-compute bmb-crypto bmb-text bmb-json
```

### From Source

```bash
# 1. Clone the repository
git clone https://github.com/iyulab/lang-bmb
cd lang-bmb

# 2. Build the BMB compiler
cargo build --release --features llvm --target x86_64-pc-windows-gnu  # Windows
cargo build --release --features llvm                                  # Linux/macOS

# 3. Build all binding libraries
python ecosystem/build_all.py

# 4. Install a library in development mode
cd ecosystem/bmb-algo
pip install -e .
```

### Requirements

- Python 3.8+
- Windows: MSYS2/UCRT64 runtime (gcc runtime DLLs)
- Linux: glibc 2.17+
- macOS: 11.0+ (ARM64 or x86_64)

## Quick Start

```python
import bmb_algo
import bmb_crypto
import bmb_text
import bmb_json
import bmb_compute

# Algorithms
bmb_algo.knapsack([2, 3, 4], [3, 4, 5], 7)           # 9
bmb_algo.dijkstra([[0, 4, -1], [-1, 0, 2], [-1, -1, 0]], 0)  # [0, 4, 6]

# Crypto
bmb_crypto.sha256("hello world")                       # hex string
bmb_crypto.base64_encode("hello")                      # 'aGVsbG8='

# Text
bmb_text.kmp_search("hello world", "world")            # 6
bmb_text.str_replace_all("abcabc", "abc", "X")         # "XX"

# JSON
bmb_json.validate('{"name": "BMB"}')                   # True
bmb_json.get_string('{"name": "BMB"}', "name")         # "BMB"

# Compute
bmb_compute.dot_product([1, 2, 3], [4, 5, 6])          # 32
bmb_compute.factorial(10)                               # 3628800
```

## Error Handling

BMB libraries use setjmp/longjmp-based error handling. When a contract violation occurs (e.g., invalid arguments), the function returns a safe default value instead of crashing the host process.

```python
# Contract violations return safe defaults
bmb_algo.knapsack([], [], -1)  # Returns 0 (contract: capacity >= 0)

# The FFI safety layer ensures the Python process never crashes
```

## Data Types

### Arrays

Array-based functions accept Python lists. The library converts them to C arrays internally.

```python
# Lists of integers
bmb_algo.quicksort([5, 3, 1, 4, 2])    # [1, 2, 3, 4, 5]
bmb_compute.sum([10, 20, 30])           # 60
```

### Matrices

Matrix functions accept lists of lists (2D arrays).

```python
adj_matrix = [
    [0, 4, -1],
    [-1, 0, 2],
    [-1, -1, 0],
]
bmb_algo.dijkstra(adj_matrix, source=0)  # [0, 4, 6]
```

Use `-1` to represent "no edge" (infinity) in adjacency matrices.

### Strings

String functions accept Python strings. The library handles UTF-8 encoding.

```python
bmb_text.kmp_search("hello world", "world")  # 6
bmb_crypto.sha256("hello")                    # hex string
```

### Scaled Values

Some bmb-compute functions return scaled integers to avoid floating-point:

```python
bmb_compute.mean_scaled([10, 20, 30])    # 20000 = 20.000 (scaled x1000)
bmb_compute.lerp_scaled(0, 100, 500)     # 50 (t=500 means t=0.5)
```

## Building from Source

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable | Build BMB compiler |
| LLVM | 21+ | IR optimization |
| GCC | MinGW-w64 | Linking (Windows) |
| Python | 3.8+ | Bindings |

### Build Commands

```bash
# Build all 5 libraries
python ecosystem/build_all.py

# Build one library
python ecosystem/build_all.py bmb-algo

# Build + run tests
python ecosystem/build_all.py --test

# Build + generate C headers
python ecosystem/build_all.py --headers

# Generate C headers only
python ecosystem/gen_headers.py

# Debug build (no optimization)
python ecosystem/build_all.py --debug
```

### C/C++ Integration

Each library includes a C header in `include/`:
```c
#include "bmb_algo.h"  // 49 function declarations

int64_t result = bmb_knapsack(weights_ptr, values_ptr, n, capacity);
int64_t fib = bmb_fibonacci(10);  // 55
```

Compile: `gcc example.c -I<lib>/include -L<lib> -l<module> -o example`

### Running Tests

```bash
# Per-library tests (pytest)
cd ecosystem/bmb-algo && python -m pytest tests/ -v

# All libraries at once
python ecosystem/test_all_bindings.py
python ecosystem/test_edge_cases.py
```

### Running Benchmarks

```bash
python ecosystem/bmb-algo/benchmarks/bench_algo.py
python ecosystem/bmb-crypto/benchmarks/bench_crypto.py
python ecosystem/bmb-text/benchmarks/bench_text.py
python ecosystem/bmb-json/benchmarks/bench_json.py
python ecosystem/bmb-compute/benchmarks/bench_compute.py
```

## Architecture

```
ecosystem/
├── bmb-algo/
│   ├── src/lib.bmb           # BMB source (1272 LOC)
│   ├── bindings/python/
│   │   ├── bmb_algo.py       # Python ctypes wrapper
│   │   ├── bmb_algo.pyi      # Type stubs
│   │   ├── bmb_algo.dll      # Compiled library
│   │   └── __init__.py
│   ├── tests/
│   │   └── test_bmb_algo.py  # pytest suite (189 tests)
│   ├── benchmarks/
│   │   └── bench_algo.py     # Performance comparison
│   ├── examples/
│   │   └── knapsack_solver.py
│   ├── pyproject.toml
│   ├── setup.py
│   ├── CHANGELOG.md
│   ├── LICENSE
│   └── README.md
├── bmb-compute/  (same structure)
├── bmb-crypto/   (same structure)
├── bmb-text/     (same structure)
├── bmb-json/     (same structure)
├── build_all.py              # Unified build script
├── test_all_bindings.py      # Monolithic test suite
└── test_edge_cases.py        # Edge case tests
```

## Performance

BMB libraries are compiled to native code via LLVM. Performance vs pure Python implementations:

| Function | Speedup | Library |
|----------|---------|---------|
| prime_count(10k) | 32x | bmb-algo |
| knapsack(100 items) | 6.3x | bmb-algo |
| edit_distance | 6.4x | bmb-algo |
| nqueens(10) | 4.1x | bmb-algo |
| merge_sort(15) | 3.3x | bmb-algo |

*Note: Comparisons vs Python C-backed stdlib (hashlib, json, str methods) show slower results due to ctypes FFI overhead (~1us/call). BMB's native performance matches or exceeds C — the overhead is in the Python-to-native bridge, not the algorithm.*

## Contributing

To add a new BMB binding library:

1. Create `ecosystem/bmb-<name>/src/lib.bmb` with `@export` functions
2. Build: `bmb build src/lib.bmb --shared --release -o bmb_<name>.dll`
3. Create Python wrapper: `bindings/python/bmb_<name>.py`
4. Add type stubs: `bindings/python/bmb_<name>.pyi`
5. Add tests: `tests/test_bmb_<name>.py`
6. Add to `ecosystem/build_all.py`

## Known Limitations

| Limitation | Details |
|-----------|---------|
| **FFI overhead** | Each ctypes call has ~1us overhead. For micro-operations (abs, min), Python builtins are faster. BMB shines on compute-heavy functions (sort, knapsack, SHA-256). |
| **Array size** | Sorting algorithms may hit memory limits at ~800+ elements for heap_sort. quicksort and merge_sort handle 1000+ elements fine. |
| **Control characters** | hex_decode and rot13 only handle printable ASCII (32-126). Bytes 0-31 are replaced with '?'. |
| **Integer-only** | All BMB functions operate on i64. Floating-point values must be scaled (e.g., mean_scaled returns value x 1000). |
| **Windows primary** | Tested primarily on Windows with MSYS2/UCRT64. Linux/macOS builds are supported but not CI-tested. |
| **String memory** | Output strings from BMB functions are managed by the BMB runtime. Do not call bmb_ffi_free_string on output strings. |

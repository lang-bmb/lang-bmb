# Cycles 2125-2144 Roadmap: Library Expansion + Quality
Date: 2026-03-23

## Goal
Expand library function counts, add batch APIs to amortize FFI overhead, improve wrapper quality.

## Phase 1: New BMB Functions (Cycles 2125-2132)
- **2125-2126**: bmb-algo: +8 algorithms (radix_sort, shell_sort, bellman_ford, kruskal, scc, convex_hull, longest_path, subset_sum)
- **2127-2128**: bmb-compute: +8 functions (median, mode, percentile, normalize, cumsum, moving_avg, cross_product, magnitude)
- **2129-2130**: bmb-text: +5 functions (levenshtein, str_split_count, str_pad_left, str_pad_right, str_center)
- **2131-2132**: bmb-json: +4 functions (object_keys, object_len, has_key, get_bool)

## Phase 2: Python Wrapper Quality (Cycles 2133-2136)
- **2133**: Add __all__ to all 5 libraries
- **2134**: Add batch helper functions (Python-side) for crypto/text
- **2135**: Improve error messages in bmb-text and bmb-json (read bmb_ffi_error_message)
- **2136**: Update .pyi stubs, tests, benchmarks for new functions

## Phase 3: Cross-platform + Integration (Cycles 2137-2142)
- **2137-2138**: Cross-platform build documentation + Makefile/shell script
- **2139-2140**: sdist build verification + wheel build
- **2141-2142**: Full regression test + benchmark comparison update

## Phase 4: Final Pass (Cycles 2143-2144)
- **2143**: ROADMAP + BINDING_ROADMAP update
- **2144**: Summary + commit

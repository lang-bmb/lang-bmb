# Cycles 2145-2164 Roadmap: C Headers + More Functions + WASM Exploration
Date: 2026-03-23

## Goal
Generate C headers for FFI consumers, expand libraries further, add docstrings, explore WASM.

## Phase 1: C Header Generation (Cycles 2145-2148)
- **2145**: Create header generation script that parses @export from lib.bmb
- **2146**: Generate .h files for all 5 libraries
- **2147**: Test C headers with a simple C consumer program
- **2148**: Add header generation to build_all.py

## Phase 2: More Library Functions (Cycles 2149-2154)
- **2149-2150**: bmb-algo: +6 (sieve_primes, is_palindrome_num, digit_sum, rotate_matrix, spiral_order, kth_smallest)
- **2151-2152**: bmb-crypto: +3 (rot13, hex_encode, hex_decode)
- **2153-2154**: bmb-compute: docstrings for all 32 functions + 2 new (gcd_array, lcm_array)

## Phase 3: WASM Exploration (Cycles 2155-2158)
- **2155**: Test WASM output with bmb-json (simplest library)
- **2156**: Create minimal Node.js wrapper for WASM module
- **2157**: Create browser-compatible JS wrapper
- **2158**: Document WASM build process

## Phase 4: Final Polish (Cycles 2159-2164)
- **2159-2160**: Updated edge case tests for new functions
- **2161-2162**: Updated CHANGELOGs, version bumps
- **2163-2164**: Full regression + summary + commit

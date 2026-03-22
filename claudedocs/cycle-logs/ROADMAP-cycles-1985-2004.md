# Roadmap: Cycles 1985-2004 — Library expansion + JSON + quality
Date: 2026-03-22

## Phase 1: bmb-json library (Cycles 1985-1988)
- Port stdlib/json/mod.bmb to @export library
- JSON parse/stringify with string I/O
- Python bindings + validation vs json.loads/dumps

## Phase 2: bmb-algo expansion (Cycles 1989-1992)
- More algorithms: radix sort, Huffman, matrix operations, string hashing
- Integration from gotgan-packages: bmb-rand, bmb-bitset
- Python bindings for all new functions

## Phase 3: bmb-crypto expansion (Cycles 1993-1996)
- Adler-32, Fletcher-16 from bmb-checksum
- XorShift64* PRNG from bmb-rand
- More checksums

## Phase 4: bmb-text expansion (Cycles 1997-2000)
- Trie-based multi-pattern matching
- String replace/split/join operations
- Regex-like pattern matching

## Phase 5: Build system + benchmarks (Cycles 2001-2004)
- Build script for all 4 libraries
- Benchmark automation (bmb vs Python stdlib)
- Cross-platform validation
- Documentation

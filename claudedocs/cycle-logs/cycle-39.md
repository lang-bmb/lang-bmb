# Cycle 39: XorShift64* PRNG for bmb-rand

## Date
2026-02-07

## Scope
Replace bmb-rand's broken LCG/pseudo-xorshift with proper XorShift64* PRNG (Marsaglia/Vigna). Add 12 comprehensive tests.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

Proper PRNG proves BMB handles 64-bit bitwise computation effectively. Zero dependencies, pure BMB.

## Research Summary
- XorShift64* (Vigna 2016): Period 2^64-1, passes BigCrush
- Algorithm: `x ^= x>>12; x ^= x<<25; x ^= x>>27; return x * 2685821657736338717`
- BMB bitwise ops: `bxor`, `band`, `bor` (infix), `>>`, `<<` (shift operators)
- Original bmb-rand used multiplication instead of XOR (broken "xorshift")

## Implementation

### Key Changes
- **Core PRNG**: Replaced broken xorshift with proper `xorshift64star()` using real `bxor` operations
- **Seed init**: `init_seed()` ensures nonzero state (XorShift requirement)
- **Output**: `rand_pos()` masks to 62 bits for positive values; `rand_range()` uses modulo
- **Backward compatibility**: Kept `lcg_next()` (renamed from `next_state`), LCG constants
- **API preserved**: `next_state`, `rand_range`, `rand_between`, `rand_bool`, `rand_n`, `hash_seed`, `should_swap`

### Files Modified
- `ecosystem/gotgan-packages/packages/bmb-rand/src/lib.bmb` (rewritten: 63 LOC → 243 LOC)

## Test Results
| Package | Tests | Result |
|---------|-------|--------|
| bmb-rand | 12/12 | PASS |
| Rust tests | 694/694 | PASS (no regressions) |

### Test Coverage
1. `different_seeds` — Different seeds produce different output
2. `determinism` — Same seed reproduces same sequence
3. `zero_seed` — Zero seed initializes to nonzero
4. `state_advances` — Each call produces new state
5. `rand_range_bounds` — Output in [0, max) bounds
6. `rand_between` — Output in [min, max] inclusive range
7. `rand_bool` — Returns valid boolean
8. `rand_n` — N-step advance equals N manual calls
9. `hash_seed` — Hash produces diverse nonzero values
10. `distribution` — Roughly uniform distribution (χ² style)
11. `no_short_cycle` — No cycles under 100 states
12. `lcg_legacy` — Legacy LCG API still works

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | All tests pass, proper algorithm |
| Architecture | 9/10 | Clean separation: core, output, utility, legacy |
| Philosophy Alignment | 9/10 | Pure BMB, zero dependencies, bitwise ops |
| Test Quality | 9/10 | 12 tests including statistical + cycle tests |
| Code Quality | 9/10 | Well-documented sections, backward compatible |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | `rand_range` uses modulo bias (non-uniform for non-power-of-2) | Accept for now; fix in Phase 2 |
| I-02 | L | No CSPRNG (not suitable for cryptographic use) | Expected; document in API |

## Next Cycle Recommendation
1. Add bmb-sort package (quicksort, mergesort using heap allocation)
2. Or improve bmb-collections with ordered map/set
3. Continue Phase 1 pre-RC dogfooding plan

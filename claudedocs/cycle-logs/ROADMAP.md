# 20-Cycle Roadmap: Performance Focus (Cycles 1865-1887)
Date: 2026-03-12

## Phase A: Inline Main Wrapper (Cycles 1865-1869) ✅ DONE
- Cycle 1865-1867: Early termination (discovered 13-17% call overhead)
- Cycle 1868: LTO analysis (abandoned — re-enables unrolling)
- Cycle 1869: alwaysinline + inline main wrapper → tower_of_hanoi 1.16x FAIL → 1.03x PASS

## Phase B: Full Benchmark Validation (Cycle 1869) ✅ DONE
- 10 key benchmarks validated, all stable or improved
- Bootstrap Stage 1 verified (emit-ir-only wrapper avoids duplicate main)

## Phase C: max_consecutive_ones Analysis (Cycle 1870) ✅ DONE (ACCEPTED)
- Root cause: LLVM SLP vectorizer batches icmp eq into AVX2 vpcmpeqq — non-profitable
- Accepted as LLVM limitation (cannot disable SLP selectively per function)

## Phase D: Bootstrap Performance (Cycles 1870-1871) ✅ DONE
- Cycle 1870: max_consecutive_ones deep analysis
- Cycle 1871: Runtime init fix — bmb_init_runtime() + bmb_arena_destroy() in inline wrapper
- alwaysinline + nosync + no-trapping-math on bmb_user_main in bootstrap
- 3-Stage Fixed Point verified (108,519 lines IR)

## Phase D2: Bootstrap Codegen Parity (Cycles 1872-1873) ✅ DONE
- Cycle 1872: GEP `inbounds` on all 19 sites → 2,109/2,109 GEPs in IR
- Cycle 1873: GEP `nuw` + `nonnull` on 25+ allocating function returns → 40 annotations

## EARLY TERMINATION (Cycle 1874)
- Zero actionable defects, codegen comprehensive, benchmark suite unavailable
- Remaining phases deferred:

## Phase E: Runtime Splitting (DEFERRED)
- Split bmb_runtime.c into core/string/vec/concurrent modules
- Only link needed modules → reduce binary .text size

## Phase F: Contract Category Expansion (DEFERRED, PARTIALLY DONE)
- SAE (saturating arithmetic elimination) in bootstrap
- range() postcondition in bootstrap
- nonnull/noundef bootstrap completion — DONE (Cycle 1873)

## Phase G: Final Optimization + Validation (DEFERRED)
- Full 310+ benchmark validation (requires benchmark submodule)
- 3-stage bootstrap fixed point verification — DONE (all cycles verified)
- Performance regression check

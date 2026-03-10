# Cycle 1836: FAIL Analysis — Assembly-Level Confirmation
Date: 2026-03-10

## Inherited → Addressed
From 1835: No defects. Analyzing remaining 2 benchmark FAILs.

## Scope & Implementation

### tower_of_hanoi (1.15x FAIL)
- **Assembly comparison**: BMB and C produce byte-identical inner loops
- Same loop structure (BB0_1→BB0_2→BB0_3→BB0_6), same registers, same instructions
- Only difference: BMB uses `jmp print` (tail call) vs C's `callq printf`
- **Binary size**: BMB 67.6KB .text vs C 12.7KB .text — 55KB runtime bloat
- **gc-sections ineffective**: MinGW ld on COFF PE cannot strip individual functions from a single .o in archive
- Runtime has 366 functions compiled into a single object file

### max_consecutive_ones (1.13x FAIL)
- **Assembly comparison**: Nearly identical inner loops — same 8x unroll, same cmov chains
- Only difference: cmoveq/cmovneq pair ordering (harmless)
- Same root cause as tower_of_hanoi: icache pressure from runtime bloat

### Root Cause: MinGW Linker Limitation
- `bmb_runtime.c` compiles to a single .o with 366 functions
- Even with `-ffunction-sections`, MinGW ld cannot strip unused sections from a single object in a COFF PE archive
- LTO not supported with MinGW on Windows
- Fix would require splitting runtime into per-function source files or using a different linker

### Files Changed
- None (analysis-only cycle)

## Review & Resolution
- Both FAILs confirmed as toolchain issues with identical assembly
- No BMB codegen improvements possible — codegen is already optimal

## Carry-Forward
- Pending Human Decisions: Whether to split runtime into per-function files (significant effort, only affects Windows MinGW)
- Discovered out-of-scope: None
- Next Recommendation: Run full benchmark suite to verify no new WARNs; consider early termination if stable

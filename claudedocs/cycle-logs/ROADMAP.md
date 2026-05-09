# BMB Directional Roadmap (cycle-level)

> This is the **directional** roadmap used by `/iyu:run-cycle` for trigger
> detection. The authoritative, detailed project roadmap lives in
> [`docs/ROADMAP.md`](../../docs/ROADMAP.md); this file is intentionally
> thin and revisable per cycle.
>
> Updated: 2026-05-09 (Cycle 2587 — Track R ~95% (run+analyze pipeline), Track Q ~92% (10 checks).
> M~100%, N~99%, O~95%, Q~92%, R~95%, T~95%.
> Next: Cargo test verification, M3 planning, Track T npm publish coordination.)

---

## Current phase — Performance-First gap closure ★ COMPLETED Cycle 2535

### M1 자율 부분 결과 (post-Cycle 2535)

- **Tier 1+3 sweep (16 historic benches)**: 16/16 within ≤1.05 strict gate.
  - **5 FAST** (BMB < 0.95×C): n_body, fannkuch, http_parse, json_serialize, sorting
  - **11 OK** (≤1.05): mandelbrot, fibonacci, hash_table, binary_trees,
    fasta, spectral_norm, string_hash, brainfuck, csv_parse, json_parse, lexer
  - **0 FAIL**
- **json_parse 1.12 → 1.04** (text), 1.16 → 1.04 (inkwell). M1 핵심 격차 해소.
- **Bootstrap 3-Stage Fixed Point** ✅ (Cycle 2535, 71s).
- **http_parse 빌드 결함** (cycle 2531 보고) close.

### Current phase status — what's next

Performance-First phase가 자율 부분 종료. 후속 작업은 **별도 run-cycle session 권장**:

1. ✅ **P-A.3''**: inkwell `readnone`+`speculatable` combo landed (Cycle 2539).
2. ✅ **P-A.5**: inkwell runtime decls memory enum (Cycle 2536).
3. **M2 도구층 — Track O**: context_pack pipeline.
   - ✅ Phase 2a (read_dir runtime): already done (Cycle 2500 era — `bmb_readdir` in C runtime + interpreter + codegen).
   - ✅ Phase 2b (stdlib/fs): already done (`stdlib/fs/mod.bmb` has all dir functions).
   - ✅ **Phase 2c (walker.bmb)**: DONE Cycle 2541 (`bootstrap/context_pack/walker.bmb`, 7/7 tests pass).
   - ✅ **Phase 3 (extractor.bmb)**: DONE Cycle 2542 (`bootstrap/context_pack/extractor.bmb`).
   - ✅ **Phase 4+5 (context_pack.bmb)**: DONE Cycle 2543 — full pipeline: walker+extractor+JSON+CLI. UTF-8 em-dash bug fixed.
   - ✅ **Phase 6 (token budget)**: DONE Cycle 2544 — `--max-tokens N` flag + strip_contracts filter. `budget_mode`/`budget_tokens` stats fields.
   - Phase 7 (validation, optional): 1 cycle.
4. **Track N M2 도구층** (bmb-mcp): ✅ **COMPLETE** (Cycles 2524-2557)
   - ✅ Phase 2a-2c (Cycles 2545-2548): 6 tools + 3 resources + 3 prompts. 35/35 pytest.
   - ✅ **Phase 2d (Cycles 2550-2556)**: bmb_compile, bmb_test, bmb_from_rust, bmb_context_pack, bmb_run, bmb_ir + bmb://context/stdlib resource. 74/74 pytest.
   - **Track N ~99% complete**: 12 tools, 4 resources, 3 prompts.
5. **Track Q M2 도구층**: ✅ lint --ai-friendly done (Cycle 2548, MCP-layer, 35/35 pytest).
   - `bmb_lint_explain`: 12 warning kinds with explanation + fix_suggestion.
   - Remaining Track Q (BMB-native lint module): deferred — lower priority than Phase 2d.
5. **Track T**: Node bindings PoC (M3 진입).

---

## (Historic — pre-Cycle 2535)

### High-level goals (directional, not binding)

### High-level goals (directional, not binding)

1. **ROADMAP/Spec alignment with v0.98 reality** — ✅ ACHIEVED Cycle
   2526. Strict ≤1.05x gate, 1/16 FAIL (json_parse 1.12x), v0.51.22
   stale references obsoleted, ai-proof removed (Cycle 2523 promise
   kept).

2. **Benchmark noise auto-detection** — ✅ ACHIEVED Cycle 2527.
   `scripts/benchmark.sh` + `measure-v098.sh` 양쪽에 warmup-driven
   adaptive runs gate (default ≥10 runs when warmup < 100ms).

3. **~~`loop` / `while` expression (HANDOFF P-1)~~** — ❌ **INVALIDATED
   Cycle 2528**. 진단 결과 `loop`/`while`/`break`/`continue` 모두 이미
   BMB grammar에 존재 (line 39-41, 1359-1390). HANDOFF의 사실 오류.

4. **~~String byte-access intrinsic (HANDOFF P-2)~~** — ❌ **INVALIDATED
   Cycle 2528**. BMB IR이 이미 `byte_at`을 GEP+load로 직접 인라인.
   runtime function call 없음. HANDOFF의 사실 오류.

5. **P-A: opt SimplifyCFG byte-switch aggregation 회피** ★ — Cycle
   2528 발견된 진짜 root cause. BMB IR의 `if c == 91 or c == 123` +
   `if c == 93 or c == 125` chain이 opt -O2에 의해 4-way
   `switch i8 [123, 91, 125, 93]`로 합쳐지고, llc가 이를 jump table
   + indirect jump로 lowering. C는 별개 2-way switch 두 개로 유지되어
   LLVM이 OR-32 트릭으로 컴팩트화. 격차: ~2 CPU cycles/iter.
   - **P-A.1**: opt 옵션 조사 — `-simplifycfg-merge-cond-stores`,
     `-switch-range-to-icmp` 등 sub-flag (1 cycle PoC)
   - **P-A.2**: 빌드 파이프라인 통합 (1 cycle)
   - **P-A.3**: 5 byte-stream 벤치 (brainfuck/csv/http/json/lexer)
     실측 ROI (1 cycle)
   - **대안 P-C**: llc-only 경로로 opt 우회 (1-2 cycles)

6. **Verifier-driven bounds check elimination — L3 (deferred P-3)**
   Contract `pre pos < s.len()` → 제거. json_parse 격차의 주 원인
   아니므로 Cycle 2528 이후 차순위.

### Known unknowns (not presumed answers, post-Cycle 2528 reframe)

- opt -O2 SimplifyCFG의 byte switch aggregation을 막을 stable LLVM
  flag가 있는가? `-switch-range-to-icmp`, `-simplifycfg-merge-cond-stores=false`
  등 후보. 없으면 codegen-level workaround 또는 llc-only 경로.
- llc-only 경로의 다른 벤치 회귀 가능성 — opt -O2의 다른 최적화 손실 영향 평가.
- 5 byte-stream 벤치 모두 같은 root cause인가? json_parse 외 다른 것은
  switch가 없을 수도 (csv_parse/lexer는 더 단순 패턴).
- 차후: BMB MIR-level "preserve compare-chain shape" hint 도입 가치.

### Non-goals for this phase

- Rust compiler new features (Rule 6 — BMB-first)
- Major M3+ work (Node bindings PoC, etc.) — gated on M1 perf closure
- v1.0 declaration — external signals dependent (see docs/ROADMAP.md
  § Vision Framework)

---

## Dependency chain (for trigger detection)

```
P-6 ROADMAP align ────✅ done Cycle 2526
    │
    ├─▶ P-1 loop/while (L1) ───────▶ json_parse ≤1.05x (target)
    │       │                            │
    │       ├─▶ Bootstrap 3-Stage S2==S3
    │       │       │
    │       │       └─▶ blocks: any further L1 syntax change
    │       │
    │       ├─▶ json_parse rewritten with loop (P-1 ROI verification)
    │       │
    │       └─▶ unblocks: deeper compiler iteration
    │
    ├─▶ P-2 byte intrinsic (L4) ─────▶ stacks with P-1 effect
    │
    ├─▶ P-3 bounds elim (L3) ────────▶ stacks with P-1 + P-2
    │       │
    │       └─▶ requires: contract→IR pass infrastructure
    │
    ├─▶ P-5 bench noise gate ────────▶ infra; runs anytime
    │
    └─▶ P-4 NUL-terminated view (L1) ── HUMAN decision pending
```

---

## Revision history

- 2026-05-01 post-2535: **M1 ≤1.05 strict gate 16/16 PASS** 첫 달성.
  Cycles 2532 (text noinline 자동 부여 — json_parse 1.12 → 1.04),
  2533 (inkwell parity: noinline + readonly enum — json_parse 1.16 → 1.04),
  2534 (text switch narrow-aware operand type — http_parse build close),
  2535 (Bootstrap 3-Stage Fixed Point S2 == S3 stable). 0 FAIL.
  P-A track close. M2 도구층/M3 Node bindings 별도 session.

- 2026-05-01 post-2528: **HANDOFF P-1/P-2 INVALIDATED**. IR-level
  diagnosis identified true root cause: opt -O2 SimplifyCFG가 4-way
  byte switch로 aggregate → llc jump table → indirect jump (~2 cycles
  penalty/iter). HANDOFF의 "loop/while 도입" 및 "byte intrinsic"는
  이미 완료된 작업이었음. 새 P-A track 도입.

- 2026-05-01 post-2527: **Benchmark noise gate** — `benchmark.sh` +
  `measure-v098.sh` 양쪽에 warmup-driven adaptive runs (default
  ≥10 when warmup<100ms). stderr-only verbose. 5 시나리오 검증 통과.

- 2026-05-01 post-2526: **Performance-First alignment** — ROADMAP and
  Cycle 2526 in sync. M1 strict ≤1.05x gate replaces 1.10x workaround.
  v0.51.22 stale data references obsoleted. ai-proof removed (Cycle
  2523 promise).

- 2026-04-23 post-2472: **Session end — CI green baseline achieved
  empirically**. (Pre-Performance-First era; preserved for history.)
  BMB CI 9/9 + Bootstrap + Benchmark 3-Stage on `8db5ac9e`. 8
  autonomous cycles used of 20-cycle budget.

- 2026-04-22 post-2469 ← post-2418: distribution + LLVM 22 work,
  CI clippy/submodule/checkout/static-libgcc cascade. See
  cycle-2418..2472 logs for detail. Pre-superseded by 2026-05-01
  Performance-First era.

**For per-cycle detail see `claudedocs/cycle-logs/cycle-NNNN.md`.**

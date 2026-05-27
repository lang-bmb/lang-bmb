# Cycle 3126: M8-B 배치 3 — String trivial 31개 교체 (IR pass bounds)
Date: 2026-05-25

## Re-plan
Plan valid. 9160-10900 범위 IR 변환 함수들 교체. 예상 25개 → 실제 31개 (추가 발견).

## Scope & Implementation
31개 교체, 4개 패턴:

**Elimination (output ≤ ir) — 8개**: eliminate_zext_trunc_ir, eliminate_dead_zexts_ir, eliminate_inttoptr_roundtrips_ir, eliminate_dead_ptrtoints_ir, eliminate_store_load_ir, eliminate_dead_stores_ir, eliminate_dead_functions_ir, prune_unused_decls

**Annotation (output ≥ ir) — 7개**: annotate_memory_none_ir, annotate_memory_read_ir, annotate_memory_read_interprocedural, annotate_speculatable, annotate_inlining, annotate_nofree, annotate_tailcalls

**Accumulator (output ≥ acc) — 6개**: try_add_zext, try_add_trunc_alias, try_mark_dead_zext, try_add_ptrtoint, try_add_inttoptr_alias, try_mark_dead_ptrtoint

**Lookup/Extraction (output ≤ input) — 10개**: lookup_zext_src, lookup_ptr_src, ipr_extract_at_name, cont_exit_lookup, slf_store_ptr, slf_store_val, slf_load_ptr, slf_load_dest, slf_table_get, extract_ir_fn_name

## Verification & Defect Resolution
- cargo test --release ✅ (6255 tests)
- bmb check ✅ warnings: 3087 → 3074 (−13)
- bmb verify ✅ 954/954

## Reflection
- String trivials: 237 → 206 (−31)
- IR pass bounds 패턴 확립: eliminate_*/prune → ≤, annotate_* → ≥, try_add_/try_mark_ → ≥ accumulator
- 이 패턴들은 compiler correctness를 강화하는 중요한 length contracts

## Carry-Forward
- Actionable: 남은 206개 분석 계속 (11176-22684 범위)
- Next Recommendation: Cycle 3127: 11176+ 범위 분석

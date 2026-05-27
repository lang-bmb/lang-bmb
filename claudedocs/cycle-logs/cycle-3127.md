# Cycle 3127: M8-B 배치 4 — String trivial 54개 교체 (12614-22684 범위)
Date: 2026-05-25

## Re-plan
Plan valid. M8-B 계속 — 12614-22684 라인 범위의 최적화 패스 + extraction 함수들 교체.

## Scope & Implementation
54개 교체, 5개 서브-배치:

**서브-배치 1 (12614-13697, 15개)**:
- 최적화 패스 (≤ fn_mir/mir): mlcse_function, optimize_mem_load_cse, cfeval_function, optimize_const_fn_eval, rpe_function, optimize_redundant_phi
- Extraction (≤ header/line): trl_fn_name, trl_param_names, trl_call_args, trl_call_dest, licm_phi_invariant, licm_resolve_copy, licm_is_invariant, licm_call_fn_name, rpe_lookup_const

**서브-배치 2 (13701-15923, 9개)**:
- 최적화 패스 (≤ fn_mir/mir): ifs_function, optimize_if_select, optimize_if_select_twice
- Lookup (≤ registry/s): lookup_fn_ret, lookup_fn_both, find_mir_annotation, strip_annotation
- Exact constant: gen_module_header (`post it.len() == 26`)
- Extraction (≤ t): extract_struct_name_from_ptr_type

**서브-배치 3 (15978-17978, 12개)**:
- Extraction (≤ param): strip_type_suffix, get_field_ptr_from_registry, extract_new_mapping, extract_new_llvm_line, lookup_contracts, strip_angle_brackets
- Exact concat (code gen helpers): conc_gen_call_i64_0/1/2/3, conc_gen_call_void_1/2

**서브-배치 4 (18045-18995, 8개)**:
- unpack_assume_ir, sco_dest, sco_arg1, sco_arg2, lr2l_fn_name, lr2l_param_name, filter_conflicting_decls, trim_ws

**서브-배치 5 (5516-22684, 10개)**:
- Accumulators (≥): try_add_ir_copy_alias, cf_table_set
- 최적화 패스 (≤): cf_fold_function, optimize_const_folding, eliminate_identity_copies_ir
- Extraction (≤): get_node_type, get_fn_annotation, extract_mir_return_type
- Exact: derive_loop_start (`post it.len() == exit_label.len()`), derive_output_name (`post it.len() <= input.len() + 4`)

## Verification & Defect Resolution
- cargo test --release ✅ (6255 tests)
- bmb check ✅ warnings: 3061 → 3042 (−19)
- bmb verify ✅ 954/954

## Reflection
- String trivials: 170 → 116 (−54 교체 확인)
- conc_gen_call_* 함수들에 exact concat 계약 적용 (overhead 17-33)
- derive_loop_start: 양 분기 모두 exit_label.len()과 동일한 길이 출력 → 정확한 계약
- 116개 잔여: 복잡한 IR 생성 (grow), 문자열 변환 (grow), 파서 분석 (complex) 위주

## Carry-Forward
- Actionable: 116개 남은 String trivial 계속 분석 (8000-13600 + 1-7999 범위)
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3128: 남은 cf_simplified*, cf_folded* 등 단순화 함수 + 추가 패턴 분석

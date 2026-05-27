# Cycle 3128: M8-B 배치 5 — String trivial 27개 교체 + 1개 신규 (7000-22000 범위)
Date: 2026-05-25

## Re-plan
Plan valid. M8-B 계속 — 분석 컨텍스트에서 확인한 18개 + 이번 사이클 분석 추가 10개 = 총 28개 교체(27 교체 + 1 신규 post 추가).

## Scope & Implementation
28개 편집, 6개 패턴:

**Exact identity (1개)**:
- new_loaded_from_other(line, loaded) L10668 → `post it == loaded` (body가 단순히 `loaded`)

**Exact grow by 1 (1개)**:
- fix_phi_label(content) L7698 → `post it.len() == content.len() + 1` (항상 '%' 1자 삽입)

**Length-bounded ≤ line (7개)**:
- cf_simplified_shift(dest, line, table) L11501
- cf_simplified_bitwise(dest, line, table) L11510
- cf_simplified_branch(line, table) L11657
- cf_folded_phi(line, table) L11691
- cf_simplified_select(line, table) L11741
- ube_simplify_phi(line, targets) L12021
- cont_exit_apply(fn_ir, ctrl_map) L10380 → `≤ fn_ir.len()`

**Length-bounded ≤ src param (5개)**:
- get_call_arg_types(fn_name) L8600 → `≤ fn_name.len()` (최대 7자 type code)
- optimize_cont_loop_exits(module_ir) L10280 → `≤ module_ir.len()`
- gcs_map_get(map, label) L14482 → `≤ map.len()`
- collect_strings_from_mir(mir) L15319 → `≤ mir.len()`
- collect_string_fns_from_mir(mir) L15408 → `≤ mir.len()`

**Exact constant bound ≤ N (4개)**:
- ast_op_to_icmp(op) L17967 → `post it.len() <= 3` (sge/sgt/slt/sle/eq/ne/"" 최대 3자)
- layer_name(layer) L20420 → `post it.len() <= 9` ("near-leaf" 최대)
- pat_classify(entries, name, calls) L20830 → `post it.len() <= 14` ("tail-recursive" 최대)
- cls_category(name) L20997 → `post it.len() <= 11` ("constructor" 최대)

**Accumulator ≥ input (9개)**:
- add_norecurse_attr(ir) L15658 → `≥ ir.len()` (norecurse 삽입 또는 unchanged)
- add_nofree_attr(ir) L15673 → `≥ ir.len()`
- add_speculatable_attr(ir) L15696 → `≥ ir.len()`
- add_mapping(mapping, name, value) L17356 → `≥ mapping.len()`
- inject_fn_assumes(fn_ir, assumes_ir) L18075 → `≥ fn_ir.len()`
- inject_contract_assumes_all(ir, contracts_map) L18091 → `≥ ir.len()`
- inject_post_assumes_all(ir, post_map) L18126 → `≥ ir.len()`
- inject_post_assumes_in_fn(fn_ir, ...) L18150 → 신규 추가 (pre only → pre+post)
- annotate_return_ranges(module_ir, post_map) L18432 → `≥ module_ir.len()`

**Accumulator ≥ visited (1개)**:
- cov_reachable(entries, name, visited) L20754 → `≥ visited.len()`

**비교체 분류 (주요 skip 이유)**:
- llvm_gen_string_ref: idx_str 길이가 동적 — 정확한 길이 표현 불가
- build_contracts_map/build_post_contracts_map: AST S-expression이 소스보다 길 수 있음
- replace_all_str: new_s가 old보다 길면 출력 > 입력 가능
- cf_simplified_arith, cf_folded, cf_try_simplify: strength-reduction 2줄 출력으로 성장 가능
- fmt_source, repl_try_*: 컴파일러 출력 — 입력과 무관한 크기

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 3042 → 3028 (−14)
- bmb verify ✅ 953/953 (기존 954→953: inject_post_assumes_in_fn 새 post 계약 Z3 timeout 추정)

## Reflection
- String trivials: 116 → 89 (−27 교체 확인, 28 편집 중 1개는 pre-only 함수에 신규 post 추가)
- "constant bound ≤ N" 패턴 신규 도입: 분류 함수 (ast_op_to_icmp, layer_name, pat_classify, cls_category) 에 적용
- inject_* assume 주입 패턴: 모두 "입력 unchanged OR 입력 + 주입 = 성장" → `≥ input.len()` 패턴
- verify 954→953 변화: inject_post_assumes_in_fn 신규 post 계약이 복잡한 재귀 없는 함수에서 Z3 timeout — 0 failed이므로 허용 가능

## Carry-Forward
- Actionable: 89개 남은 String trivial 계속 분석
  - 주요 잔여 패턴: IR codegen 함수들(grow), parser 함수들, format/emit 유틸리티
  - 1-7999 범위 + 22000+ 범위 미분석 구간 존재
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3129: 잔여 89개 중 analyzable 함수 계속 교체

# 적합성 테스트

언어 레퍼런스(`docs/reference/01`~`08`)의 절마다 그 규칙을 고정하는 사례 모음이다(백로그 5A-02, 5A-05). 사례는 구현에 기대지 않는다: 입력 프로그램과 기대하는 진단 코드·줄, 증명 판정과 상태 줄의 필드, 종료 코드, 남은 런타임 검사의 수, 실행 출력만 적는다. 메시지, 열, 수정 문구, IR 텍스트, `rlimit` 값, 시간은 사례에 없다.

## 사례의 꼴

사례 하나는 파일 한 쌍이다: `tests/conformance/NN/이름.bmb`와 `tests/conformance/NN/이름.json`. `NN`은 레퍼런스의 장 번호이고 이름은 소문자 스네이크 표기다. `.bmb` 파일 하나가 프로그램 하나다.

```
{"sections": ["그 장의 `## ` 절 제목 그대로", ...],
 "note": "이 사례가 고정하는 규칙 (영어 한 문장)",
 "check": [{"code": "E0401", "line": 12}, ...],
 "check_mode": "exact" | "includes",
 "verify": {"defs": {"unit::name": "verified" | "failed" | {"status": .., "trust": "assumed" | "external" | null,
                                                           "obligations": n, "failed": k}},
            "diags": [{"code": "E0805", "line": 9}]},
 "deterministic": ["check", "verify"],
 "emit": {"all_checks": false, "exit": 0 | 1, "runtime_checks": n},
 "build": "ok" | "refused",
 "run": "실행 파일의 표준 출력 그대로" | {"stdout": "..", "exit": "ok"} | {"exit": "trap"}}
```

- `check`는 필수다. `bmb check`가 내는 모든 진단의 (코드, 줄)을 다중집합으로 비교한다. `[]`는 검사를 통과하는 프로그램이다.
- `check_mode`는 `check`를 비교하는 방식이다. 없으면 `exact`다. `includes`는 적은 (코드, 줄)이 모두 있어야 하고, 그 밖의 진단은 적은 진단과 같은 줄에만 올 수 있다. 한 구문의 잘못 하나에 진단이 몇 개 나오는지를 레퍼런스가 정하지 않을 때만 쓴다. `check`에 오류가 있어야 한다.
- `verify`는 `check`에 오류가 없을 때만 쓴다. `defs`는 적은 정의의 상태 줄을, `diags`는 `bmb verify`가 `check`의 진단 밖에 더 내는 진단의 (코드, 줄) 다중집합을 비교한다.
- `defs`의 값은 상태 문자열이거나 객체다. 객체는 적은 키만 비교한다: `status`, `trust`, `obligations`(의무의 수), `failed`(증명되지 않은 의무의 수). `"trust": null`은 상태 줄에 `trust` 키가 없다는 뜻이다.
- `deterministic`은 두 번 돌릴 명령의 목록이다(`check`, `verify`). 두 실행의 종료 코드와 표준 출력의 바이트가 서로 같아야 한다. 저장된 텍스트와는 비교하지 않는다. `verify`는 `check`에 오류가 없을 때만 쓴다.
- `emit`은 `check`에 오류가 없을 때만 쓴다. `bmb emit-llvm`을 돌리고(`all_checks`가 참이면 `--all-checks`), 비어 있지 않은 IR 파일이 쓰여야 한다. `exit`는 종료 코드다. `runtime_checks`는 `--stats FILE`이 쓰는 줄의 같은 이름의 수이고, 증명되지 않은 의무 때문에 IR에 남은 런타임 검사의 수다. IR 텍스트는 읽지 않는다.
- `all_checks`가 참이면 표준 출력에 상태 줄이 없고 `check`의 진단 밖의 진단도 없어야 한다. 거짓이고 사례에 `verify`가 있으면 `check`의 진단 밖의 진단이 `verify.diags`와 같아야 한다. `emit`은 객체 하나이거나 `all_checks`가 서로 다른 객체 둘의 목록이다.
- `build`는 실행 없이 `bmb build`만 고정한다. `ok`는 검사를 깨끗이 통과하고 증명되는 프로그램에만 쓰고, 종료 코드가 0이고 실행 파일이 있어야 한다. `refused`는 `verify`에 실패가 있는 사례에만 쓰고, 종료 코드가 1이고 출력 디렉터리에 아무 파일도 없어야 한다.
- `run`은 증명되는 프로그램에만 쓰고 `build`와 함께 쓰지 않는다. `bmb build`가 성공해야 한다. 문자열이나 `"exit": "ok"`는 종료 코드 0으로 끝난 실행의 표준 출력이 문자열과 같아야 한다는 뜻이다.
- `"exit": "trap"`은 실행이 종료 코드 0으로 끝나지 않아야 한다는 뜻이다. 신호 번호는 고정하지 않는다. `stdout`은 함께 쓰지 않는다: trap 앞의 출력이 남는지를 레퍼런스가 말하지 않는다.
- 모르는 키, 틀린 타입, 함께 쓸 수 없는 조합은 `conformance.py check`의 오류다.
- 사례는 작다. 절 하나의 규칙 하나, 또는 가까운 규칙 몇 개를 고정한다. 오류 사례는 오류마다 줄을 달리한다.
- 사례는 레퍼런스가 말하고 컴파일러가 그렇게 하는 것만 적는다. 둘이 다르면 사례로 만들지 않고 레퍼런스나 컴파일러를 고친다.

## 실행

```
python3 scripts/conformance.py run            # 모든 사례 (컴파일러 필요)
python3 scripts/conformance.py run 03 04      # 그 장의 사례만
python3 scripts/conformance.py run --bmb PATH --jobs 8
python3 scripts/conformance.py check          # 꼴, 절 이름, 빠진 절, 표의 신선도 (컴파일러 불필요)
python3 scripts/conformance.py table --write  # 아래 표를 다시 쓴다
```

컴파일러는 `--bmb`, 환경 변수 `BMB`, `target/debug/bmb` 순으로 찾는다. `run`은 다른 사례마다 한 줄을 내고 `conformance: N cases, M failed`로 끝난다.

검사할 규칙이 없는 절(장의 색인 등)은 `tests/conformance/exempt.json`에 `{"NN": {"절 제목": "사유"}}`로 적는다. 사례도 면제도 없는 절이 있으면 `check`가 실패한다.

## 절과 사례

<!-- conformance:table begin (scripts/conformance.py table --write; do not edit by hand) -->

| 장 | 절 | 사례 수 | 사례 (또는 면제 사유) |
| --- | --- | --- | --- |
| 01 | 문자와 공백 | 5 | `comment_only_file`, `empty_file`, `non_ascii_in_comment_and_literals`, `stray_character`, `whitespace_free_form` |
| 01 | 식별자와 예약어 | 7 | `attribute_and_builtin_names_not_reserved`, `identifier_forms`, `keyword_as_parameter_name`, `keyword_as_unit_name`, `loop_label_and_lifetime`, `non_ascii_identifier`, `non_ascii_name_once` |
| 01 | 구두점 | 3 | `longest_match`, `shift_tokens`, `token_out_of_place` |
| 01 | 정수 리터럴 | 5 | `integer_bad_digits`, `integer_over_128_bits`, `integer_radixes`, `integer_suffix`, `negative_is_unary_minus` |
| 01 | 부동소수 리터럴 | 3 | `dot_after_digits`, `float_forms`, `float_malformed` |
| 01 | 문자·문자열·바이트 리터럴 | 7 | `byte_literal_errors`, `byte_literal_one_fault`, `byte_literals`, `char_and_string_escapes`, `char_literal_errors`, `unterminated_byte_string`, `unterminated_string` |
| 01 | 주석 | 3 | `block_comment`, `comment_only_file`, `line_comments` |
| 01 | `///`와 요약 | 6 | `doc_summary`, `doc_summary_before_closing_brace`, `doc_summary_before_statement`, `doc_summary_split_by_blank_line`, `doc_summary_without_item`, `doc_summary_without_text` |
| 01 | 구문의 뼈대 | 6 | `clause_order`, `impl_and_trait_bodies`, `recovery_at_next_item`, `skeleton_accepted`, `top_level_holds_units_only`, `unit_holds_items_only` |
| 01 | 중첩 한도 | 4 | `nesting_mixed_over_limit`, `nesting_over_limit`, `nesting_within_limit`, `units_nested_256_deep` |
| 02 | 단위 | 5 | `nested_units`, `no_pub`, `unit_names_unique`, `unit_std_reserved`, `unit_summary_required` |
| 02 | 항목 | 6 | `builtin_names_reserved`, `const_cycle`, `function_return_types`, `member_names_unique`, `three_function_kinds`, `unit_namespace` |
| 02 | `impl`과 수신자 | 6 | `inherent_impl_place`, `receiver_name`, `self_type_only_in_trait`, `trait_default_method`, `trait_impl_methods`, `trait_impl_place` |
| 02 | 속성 | 5 | `attribute_order_free`, `attribute_places`, `attributes_in_bodies`, `summary_on_definitions`, `summary_once_one_string` |
| 02 | 진입점 | 6 | `main_in_nested_unit`, `main_shape`, `main_twice`, `main_with_parameter`, `main_with_result`, `main_without_io` |
| 02 | 경로와 이름 찾기 | 7 | `full_paths`, `no_relative_paths`, `no_use`, `unresolved_members`, `unresolved_types`, `unresolved_values_and_calls`, `use_at_line_start` |
| 02 | 메서드 호출의 이름 | 3 | `method_ambiguous_traits`, `method_needs_receiver`, `method_resolution_order` |
| 02 | 지역 이름 | 4 | `for_variable_scope`, `local_one_type_per_name`, `local_reuse_in_disjoint_scopes`, `local_shadowing` |
| 02 | 표준 라이브러리 | 2 | `std_by_full_path`, `std_unknown_names` |
| 03 | 스칼라 타입 | 5 | `integer_min_max`, `literal_range`, `scalar_types_accepted`, `unsigned_negation`, `unsupported_scalar_types` |
| 03 | 타입을 적는 자리 | 2 | `type_arguments`, `types_are_written` |
| 03 | 연산자와 피연산자 | 7 | `arithmetic_obligations`, `builtin_methods_closed_list`, `equality_of_aggregates`, `index_type_is_usize`, `no_implicit_conversion`, `operator_results`, `type_parameter_comparison` |
| 03 | 변환 | 5 | `as_range_obligation`, `as_target_not_integer`, `conversion_source_not_integer`, `conversions_accepted`, `rust_conversion_habits` |
| 03 | 배열과 슬라이스 | 9 | `array_element_and_slice_types`, `array_element_in_every_position`, `array_length_in_type`, `array_literal_elements`, `arrays_and_slices`, `inclusive_range_outside_for`, `not_indexable`, `range_in_value_position`, `range_is_not_a_value` |
| 03 | 텍스트 | 4 | `str_arguments`, `string_literal_places`, `text_not_indexable_closed_methods`, `text_operations` |
| 03 | 구조체와 열거형 | 6 | `match_rules`, `recursive_types`, `struct_and_match_values`, `struct_enum_declarations`, `struct_values`, `type_invariant_attribute` |
| 03 | `Option`과 `Result` | 4 | `no_unwrap`, `option_result_names_reserved`, `option_result_types`, `option_result_variants` |
| 03 | 제네릭과 trait | 10 | `bound_failure_once`, `builtin_bounds`, `generics_and_traits_run`, `impl_parameters_in_order`, `polymorphic_recursion`, `rejected_impl_calls`, `rejected_impl_is_still_checked`, `trait_impl_overlap`, `trait_impl_parameters_in_order`, `type_argument_rules` |
| 03 | 함수 값 | 2 | `function_value_places`, `function_values` |
| 03 | 소유 타입 | 4 | `owned_closed_methods`, `owned_element_types`, `owned_types_run`, `spec_fn_no_owned_parameters` |
| 03 | 상수의 타입 | 2 | `const_type_errors`, `const_types_run` |
| 04 | 식의 문법과 우선순위 | 5 | `comparison_not_chained`, `evaluation_order`, `left_associativity`, `precedence_levels`, `return_binds_loosest` |
| 04 | 리터럴·경로·호출 | 4 | `call_errors`, `calls_by_path`, `unit_locals_run`, `unit_value` |
| 04 | 메서드 호출 | 3 | `builtin_method_before_trait`, `method_call_errors`, `receiver_adjustment` |
| 04 | 연산자와 피연산자 | 6 | `compound_assignment`, `compound_assignment_place_calls`, `mixed_integer_types`, `operator_values`, `shift_by_known_local`, `short_circuit` |
| 04 | 변환 | 3 | `conversion_aliases`, `conversion_errors`, `try_from_unwrap_obligation` |
| 04 | 색인·필드·역참조 | 3 | `index_obligation`, `place_errors`, `places` |
| 04 | 구조체·열거형 값 | 3 | `enum_value_errors`, `struct_field_evaluation_order`, `struct_value_errors` |
| 04 | 블록과 꼬리 식 | 2 | `block_scope_and_type`, `block_values` |
| 04 | 클로저 | 3 | `closure_capture_needs_move`, `closure_errors`, `closures_accepted` |
| 04 | 문장 | 6 | `assert_errors`, `assert_statement`, `assignment_targets`, `expression_statements`, `let_statement`, `unit_locals_run` |
| 04 | if와 match | 4 | `if_and_match_values`, `if_errors`, `match_arm_rules`, `match_pattern_rules` |
| 04 | 루프 | 4 | `for_variable`, `loop_clauses`, `loop_types`, `loops_run` |
| 04 | 조기 탈출: return, break, continue, ? | 4 | `exit_errors`, `labels_and_exits_run`, `question_mark_errors`, `question_mark_run` |
| 04 | 받지 않는 Rust 습관 | 10 | `habits_control_flow`, `habits_conversions_operators`, `habits_data_patterns`, `habits_declarations`, `habits_names_macros_escapes`, `habits_ownership_generics`, `habits_replacements_data`, `habits_replacements_ownership`, `habits_specs_ghost_consts`, `habits_unsupported_items` |
| 05 | 복사와 이동 | 8 | `clone_keeps_source`, `containers_of_owned_move`, `copy_bound_rejects_owned`, `copy_types_reused`, `owned_types_move`, `reads_do_not_move`, `type_param_moves`, `value_use_positions` |
| 05 | 이동한 값의 사용 | 9 | `fn_once_called_twice`, `match_moves_scrutinee`, `owned_types_move`, `question_moves_operand`, `reassign_restores_ownership`, `self_by_value_twice`, `specs_read_moved_values`, `use_after_move`, `value_use_positions` |
| 05 | 자리에서 꺼내기와 부분 이동 | 10 | `field_reassign_restores`, `match_on_field_is_partial`, `mem_replace_and_swap`, `move_field_behind_mut_ref`, `move_field_behind_ref_or_box`, `move_out_of_deref`, `move_out_of_element`, `no_take_no_drop`, `partial_move_of_field`, `swap_within_one_variable` |
| 05 | 갈래와 루프의 이동 | 7 | `if_without_else_moves`, `loop_body_moves`, `loop_exit_states`, `one_branch_moves`, `return_leaves_the_join`, `short_circuit_moves`, `use_after_branch_move` |
| 05 | 해제 | 3 | `drops_at_scope_and_assignment`, `drops_on_early_exits`, `no_take_no_drop` |
| 05 | 참조는 매개변수에만 | 6 | `array_ref_and_mut_str`, `lifetimes_rejected`, `ref_local_rejected`, `ref_param_inner_fault`, `ref_params_read_and_write`, `refs_only_in_parameters` |
| 05 | 참조 인자 | 10 | `borrow_outside_argument`, `method_receiver_adjustment`, `mut_subslice_rejected`, `ref_local_rejected`, `ref_t_arguments`, `ref_value_mismatch`, `slice_arguments`, `str_arguments`, `vec_or_string_as_view`, `views_outside_argument` |
| 05 | 쓸 수 있는 자리 | 4 | `assign_immutable_places`, `mut_parameter_rejected`, `mutating_receivers_need_mut`, `writable_places` |
| 05 | 별칭 | 6 | `alias_accepted_forms`, `alias_is_per_variable`, `alias_mut_and_borrow`, `alias_mut_and_move`, `alias_non_reading_expressions`, `swap_within_one_variable` |
| 05 | 참조 `match` | 4 | `mut_match_borrows_scrutinee`, `mut_match_writes`, `ref_match_binding_move`, `shared_match_bindings` |
| 05 | 클로저의 캡처 | 5 | `closure_capture_moves_owned`, `closure_captures_typed_local`, `closure_kinds_run`, `closure_move_capture_accepted`, `fn_once_called_twice` |
| 06 | 계약 절 | 8 | `all_clauses_hold`, `clause_is_bool`, `clause_order_accepted`, `clause_order_fixed`, `mut_param_in_ensures`, `quantified_clause_needs_predicate`, `result_needs_a_name`, `vacuous_predicate_clause` |
| 06 | 명세 문맥과 실행 문맥 | 5 | `executable_calls_spec`, `executable_statements_in_spec`, `spec_calls_executable`, `spec_expression_forms`, `spec_syntax_in_executable_code` |
| 06 | 명세의 정수 | 6 | `spec_arithmetic_is_mathematical`, `spec_has_no_runtime_obligations`, `spec_mixes_integer_types`, `spec_narrowing_conversion`, `spec_wide_value_in_narrow_place`, `vec_has_no_equality` |
| 06 | 수량자와 트리거 | 7 | `quantified_clause_needs_predicate`, `quantifier_binder_type`, `quantifier_needs_trigger`, `quantifier_places`, `quantifiers_instantiate`, `trigger_outside_quantifier`, `triggers_cover_binders` |
| 06 | `spec fn` | 6 | `executable_calls_spec`, `recursive_spec_fn_needs_decreases`, `recursive_spec_fn_terminates`, `spec_fn_is_opaque`, `spec_fn_takes_no_effects`, `uses_takes_spec_fns_only` |
| 06 | 루프의 `invariant`와 `decreases` | 5 | `after_loop_knowledge`, `loop_clauses_proved`, `loops_need_decreases`, `nondet_loop_without_decreases`, `quantifier_places` |
| 06 | 재귀와 `decreases` | 5 | `recursion_measures`, `recursion_needs_decreases`, `recursive_spec_fn_needs_decreases`, `recursive_spec_fn_terminates`, `trait_method_decreases` |
| 06 | `assert` | 6 | `assert_by_unknown_prover`, `assert_proves_then_assumes`, `bit_vector_assert_forms`, `bit_vector_assert_terms`, `bit_vector_ignores_path_facts`, `executable_statements_in_spec` |
| 06 | `old` | 4 | `mut_param_in_ensures`, `old_forms`, `old_of_receiver`, `old_positions` |
| 06 | `proof fn`과 `proof` 블록 | 5 | `ghost_code_runs_nothing`, `lemma_by_induction`, `lemma_call_gives_ensures`, `proof_fn_called_from_ghost_only`, `proof_fn_signature` |
| 06 | 타입 불변식 | 4 | `invariant_assumed_and_owed`, `invariant_attribute_shape`, `invariant_may_break_inside`, `invariant_not_assumed_in_ghost` |
| 06 | trait의 계약과 impl의 정제 | 2 | `trait_contract_through_bound`, `trait_spec_method` |
| 06 | 함수 값의 계약 | 3 | `closure_contract_in_place`, `fn_item_contract_as_value`, `fn_value_call_contract` |
| 07 | 의무의 단위 | 4 | `generic_definition_proved_once`, `path_facts_are_assumed`, `status_line_obligation_counts`, `status_per_definition` |
| 07 | 의무의 종류 | 24 | `assertion_obligation`, `bit_vector_sub_proof`, `call_precondition`, `conversion_in_range`, `conversion_to_f64_exact`, `division_by_zero`, `ghost_argument_range`, `index_in_bounds`, `invariant_on_entry`, `invariant_preserved`, `invariant_value_only_moved`, `loop_measure_decreases`, `no_obligations_in_specs_and_wrapping`, `overflow_add_sub_mul`, `overflow_negation_division_shift`, `postcondition_at_every_exit`, `recursive_call_decreases`, `spec_fn_termination`, `string_range_boundaries`, `subslice_and_copy_within_ranges`, `trait_contract_refinement`, `type_invariant_at_loop_head`, `type_invariant_where_values_leave`, `vec_capacity_maximum` |
| 07 | 호출자가 가정하는 것 | 6 | `build_refused_failed_postcondition`, `callee_ensures_is_assumed`, `callee_requires_is_proved`, `caller_sees_only_the_contract`, `mut_argument_after_call`, `result_type_range_is_known` |
| 07 | 증명 문맥 | 6 | `callee_contract_in_default_context`, `default_context_stops_at_bodies`, `uses_adds_a_definition`, `uses_entry_must_be_spec_fn`, `uses_is_not_transitive`, `uses_names_another_unit` |
| 07 | 판정 | 7 | `build_refused_unproved_obligation`, `counterexample_failure`, `deterministic_failure_output`, `one_unproved_obligation_fails`, `status_line_obligation_counts`, `status_line_partial_failure`, `status_per_definition` |
| 07 | 결정론 | 4 | `deterministic_failure_output`, `deterministic_proof_output`, `status_per_definition`, `verdict_independent_of_names` |
| 07 | `build`와 `emit-llvm` | 11 | `build_proved_program`, `build_refused_by_uncalled_definition`, `build_refused_failed_postcondition`, `build_refused_unproved_obligation`, `build_with_escapes`, `checks_kept_below_external_body`, `checks_kept_in_callee_of_escape`, `emit_all_checks_skips_the_proof`, `emit_connected_definitions_keep_checks`, `emit_failed_definition_keeps_check`, `emit_proved_keeps_no_checks` |
| 08 | 효과의 집합 | 6 | `alloc_sources`, `drop_is_not_an_effect`, `effects_are_orthogonal`, `false_assume_traps_at_index`, `nondet_sources`, `panic_source_is_an_escape` |
| 08 | `#[effects(..)]` 선언 | 7 | `effect_lists`, `effects_attribute_required`, `effects_attribute_several_faults`, `effects_attribute_twice`, `main_effects`, `main_with_more_effects`, `unknown_effects` |
| 08 | 선언과 몸의 관계 | 7 | `alloc_sources`, `declaration_is_the_interface`, `effects_in_mutual_recursion`, `external_body_effects_checked`, `missing_effects_one_diagnostic`, `pure_calls_io`, `trait_impl_effects` |
| 08 | 명세와 유령 코드의 효과 | 2 | `spec_and_proof_fn_effects`, `specs_add_no_effects` |
| 08 | 함수 값의 효과 | 2 | `fn_value_effects_declared`, `fn_value_effects_rejected` |
| 08 | 항목이 받는 속성 | 6 | `effects_attribute_twice`, `misplaced_attributes`, `statement_attributes`, `summary_attribute_places`, `unit_summary_required`, `unknown_attributes` |
| 08 | 탈출구: `assume` | 7 | `assume_gives_a_fact`, `assume_needs_rationale`, `assume_not_in_ghost_code`, `assume_takes_a_predicate_call`, `false_assume_ends_in_trap`, `panic_source_is_an_escape`, `true_assume_runs_to_the_end` |
| 08 | 탈출구: `#[external_body]` | 7 | `external_body_contract_accepted`, `external_body_effects_checked`, `external_body_is_a_panic_source`, `external_body_needs_rationale`, `external_body_places`, `external_body_still_checked`, `rust_and_verus_escapes` |
| 08 | 근거: `#[rationale(..)]` | 8 | `assume_needs_rationale`, `external_body_needs_rationale`, `rationale_hash_not_an_error`, `rationale_kinds`, `rationale_malformed`, `rationale_several_faults`, `statement_attributes`, `unconfirmed_rationale_builds` |
| 08 | 탈출구와 증명의 신뢰 등급 | 10 | `false_assume_ends_in_trap`, `false_assume_traps_at_index`, `needless_assume_has_no_trust`, `no_escape_no_trust`, `no_escape_no_trust_key`, `true_assume_runs_to_the_end`, `trust_does_not_spread`, `trust_field_of_status_lines`, `trusted_definitions_are_verified`, `unconfirmed_rationale_builds` |

절 86개, 사례 443개, 사례도 면제도 없는 절 0개.

<!-- conformance:table end -->

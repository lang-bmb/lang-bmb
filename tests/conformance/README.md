# Conformance tests

A collection of cases that pins the rules of each section of the language reference (`docs/reference/01`~`08`) (backlog 5A-02, 5A-05). A case does not lean on the implementation: it records only the input program and the expected diagnostic codes and lines, the proof verdict and the fields of the status line, the exit code, the number of remaining runtime checks, and the output of the run. Messages, columns, fix texts, IR text, `rlimit` values and times are not in a case.

## Shape of a case

One case is a pair of files: `tests/conformance/NN/name.bmb` and `tests/conformance/NN/name.json`. `NN` is the chapter number of the reference and the name is in lowercase snake case. One `.bmb` file is one program.

```
{"sections": ["the `## ` section title of that chapter, verbatim", ...],
 "note": "the rule this case pins (one sentence in English)",
 "check": [{"code": "E0401", "line": 12}, ...],
 "check_mode": "exact" | "includes",
 "verify": {"defs": {"unit::name": "verified" | "failed" | {"status": .., "trust": "assumed" | "external" | null,
                                                           "obligations": n, "failed": k}},
            "diags": [{"code": "E0805", "line": 9}]},
 "deterministic": ["check", "verify"],
 "emit": {"all_checks": false, "exit": 0 | 1, "runtime_checks": n},
 "build": "ok" | "refused",
 "run": "the standard output of the executable, verbatim" | {"stdout": "..", "exit": "ok"} | {"exit": "trap"}}
```

- `check` is required. It compares the (code, line) pairs of all diagnostics that `bmb check` emits, as a multiset. `[]` is a program that passes the check.
- `check_mode` is the way `check` is compared. If absent, it is `exact`. With `includes`, every listed (code, line) must be present, and any other diagnostic may appear only on the same line as a listed diagnostic. Use it only when the reference does not fix how many diagnostics one mistake in one construct produces. `check` must contain an error.
- `verify` is used only when `check` has no errors. `defs` compares the status lines of the listed definitions, and `diags` compares the multiset of (code, line) pairs of the diagnostics that `bmb verify` emits beyond the diagnostics of `check`.
- A value of `defs` is a status string or an object. For an object, only the listed keys are compared: `status`, `trust`, `obligations` (the number of obligations), `failed` (the number of obligations not proved). `"trust": null` means that the status line has no `trust` key.
- `deterministic` is the list of commands to run twice (`check`, `verify`). The exit codes and the standard output bytes of the two runs must be equal to each other. They are not compared with stored text. `verify` is used only when `check` has no errors.
- `emit` is used only when `check` has no errors. It runs `bmb emit-llvm` (with `--all-checks` if `all_checks` is true), and a non-empty IR file must be written. `exit` is the exit code. `runtime_checks` is the number of the same name in the line that `--stats FILE` writes, and it is the number of runtime checks left in the IR because of obligations not proved. The IR text is not read.
- If `all_checks` is true, the standard output must have no status lines and no diagnostics beyond the diagnostics of `check`. If it is false and the case has `verify`, the diagnostics beyond the diagnostics of `check` must equal `verify.diags`. `emit` is one object or a list of two objects whose `all_checks` differ.
- `build` pins only `bmb build`, without a run. `ok` is used only for a program that passes the check cleanly and is proved; the exit code must be 0 and the executable must exist. `refused` is used only for a case whose `verify` has a failure; the exit code must be 1 and the output directory must hold no file.
- `run` is used only for a program that is proved, and is not used together with `build`. `bmb build` must succeed. A string or `"exit": "ok"` means that the standard output of a run that ended with exit code 0 must equal the string.
- `"exit": "trap"` means that the run must not end with exit code 0. The signal number is not pinned. `stdout` is not used with it: the reference does not say whether the output before the trap remains.
- An unknown key, a wrong type and a combination that cannot be used together are errors of `conformance.py check`.
- A case is small. It pins one rule of one section, or a few closely related rules. An error case puts each error on a different line.
- A case records only what the reference says and the compiler does. If the two differ, no case is made; the reference or the compiler is fixed.

## Running

```
python3 scripts/conformance.py run            # all cases (needs the compiler)
python3 scripts/conformance.py run 03 04      # only the cases of those chapters
python3 scripts/conformance.py run --bmb PATH --jobs 8
python3 scripts/conformance.py check          # shape, section names, missing sections, freshness of the table (no compiler needed)
python3 scripts/conformance.py table --write  # rewrites the table below
```

The compiler is looked up in this order: `--bmb`, the environment variable `BMB`, `target/debug/bmb`. `run` prints one line for each case that differs and ends with `conformance: N cases, M failed`.

A section that has no rule to check (the index of a chapter, and the like) is recorded in `tests/conformance/exempt.json` as `{"NN": {"section title": "reason"}}`. If a section has neither a case nor an exemption, `check` fails.

## Sections and cases

<!-- conformance:table begin (scripts/conformance.py table --write; do not edit by hand) -->

| Chapter | Section | Cases | Cases (or the reason for the exemption) |
| --- | --- | --- | --- |
| 01 | Characters and whitespace | 5 | `comment_only_file`, `empty_file`, `non_ascii_in_comment_and_literals`, `stray_character`, `whitespace_free_form` |
| 01 | Identifiers and reserved words | 7 | `attribute_and_builtin_names_not_reserved`, `identifier_forms`, `keyword_as_parameter_name`, `keyword_as_unit_name`, `loop_label_and_lifetime`, `non_ascii_identifier`, `non_ascii_name_once` |
| 01 | Punctuation | 3 | `longest_match`, `shift_tokens`, `token_out_of_place` |
| 01 | Integer literals | 5 | `integer_bad_digits`, `integer_over_128_bits`, `integer_radixes`, `integer_suffix`, `negative_is_unary_minus` |
| 01 | Floating-point literals | 3 | `dot_after_digits`, `float_forms`, `float_malformed` |
| 01 | Character, string and byte literals | 7 | `byte_literal_errors`, `byte_literal_one_fault`, `byte_literals`, `char_and_string_escapes`, `char_literal_errors`, `unterminated_byte_string`, `unterminated_string` |
| 01 | Comments | 3 | `block_comment`, `comment_only_file`, `line_comments` |
| 01 | `///` and summaries | 6 | `doc_summary`, `doc_summary_before_closing_brace`, `doc_summary_before_statement`, `doc_summary_split_by_blank_line`, `doc_summary_without_item`, `doc_summary_without_text` |
| 01 | Syntax skeleton | 6 | `clause_order`, `impl_and_trait_bodies`, `recovery_at_next_item`, `skeleton_accepted`, `top_level_holds_units_only`, `unit_holds_items_only` |
| 01 | Nesting limit | 4 | `nesting_mixed_over_limit`, `nesting_over_limit`, `nesting_within_limit`, `units_nested_256_deep` |
| 02 | Units | 6 | `named_length_before_nested_unit`, `nested_units`, `no_pub`, `unit_names_unique`, `unit_std_reserved`, `unit_summary_required` |
| 02 | Items | 6 | `builtin_names_reserved`, `const_cycle`, `function_return_types`, `member_names_unique`, `three_function_kinds`, `unit_namespace` |
| 02 | `impl` and receivers | 6 | `inherent_impl_place`, `receiver_name`, `self_type_only_in_trait`, `trait_default_method`, `trait_impl_methods`, `trait_impl_place` |
| 02 | Attributes | 5 | `attribute_order_free`, `attribute_places`, `attributes_in_bodies`, `summary_on_definitions`, `summary_once_one_string` |
| 02 | Entry point | 6 | `main_in_nested_unit`, `main_shape`, `main_twice`, `main_with_parameter`, `main_with_result`, `main_without_io` |
| 02 | Paths and name lookup | 7 | `full_paths`, `no_relative_paths`, `no_use`, `unresolved_members`, `unresolved_types`, `unresolved_values_and_calls`, `use_at_line_start` |
| 02 | Method call names | 3 | `method_ambiguous_traits`, `method_needs_receiver`, `method_resolution_order` |
| 02 | Local names | 4 | `for_variable_scope`, `local_one_type_per_name`, `local_reuse_in_disjoint_scopes`, `local_shadowing` |
| 02 | Standard library | 2 | `std_by_full_path`, `std_unknown_names` |
| 03 | Scalar types | 5 | `integer_min_max`, `literal_range`, `scalar_types_accepted`, `unsigned_negation`, `unsupported_scalar_types` |
| 03 | Where types are written | 2 | `type_arguments`, `types_are_written` |
| 03 | Operators and operands | 7 | `arithmetic_obligations`, `builtin_methods_closed_list`, `equality_of_aggregates`, `index_type_is_usize`, `no_implicit_conversion`, `operator_results`, `type_parameter_comparison` |
| 03 | Conversions | 5 | `as_range_obligation`, `as_target_not_integer`, `conversion_source_not_integer`, `conversions_accepted`, `rust_conversion_habits` |
| 03 | Arrays and slices | 9 | `array_element_and_slice_types`, `array_element_in_every_position`, `array_length_in_type`, `array_literal_elements`, `arrays_and_slices`, `inclusive_range_outside_for`, `not_indexable`, `range_in_value_position`, `range_is_not_a_value` |
| 03 | Text | 4 | `str_arguments`, `string_literal_places`, `text_not_indexable_closed_methods`, `text_operations` |
| 03 | Structs and enums | 6 | `match_rules`, `recursive_types`, `struct_and_match_values`, `struct_enum_declarations`, `struct_values`, `type_invariant_attribute` |
| 03 | `Option` and `Result` | 4 | `no_unwrap`, `option_result_names_reserved`, `option_result_types`, `option_result_variants` |
| 03 | Generics and traits | 10 | `bound_failure_once`, `builtin_bounds`, `generics_and_traits_run`, `impl_parameters_in_order`, `polymorphic_recursion`, `rejected_impl_calls`, `rejected_impl_is_still_checked`, `trait_impl_overlap`, `trait_impl_parameters_in_order`, `type_argument_rules` |
| 03 | Function values | 2 | `function_value_places`, `function_values` |
| 03 | Owning types | 4 | `owned_closed_methods`, `owned_element_types`, `owned_types_run`, `spec_fn_no_owned_parameters` |
| 03 | Types of constants | 2 | `const_type_errors`, `const_types_run` |
| 04 | Expression grammar and precedence | 5 | `comparison_not_chained`, `evaluation_order`, `left_associativity`, `precedence_levels`, `return_binds_loosest` |
| 04 | Literals, paths and calls | 4 | `call_errors`, `calls_by_path`, `unit_locals_run`, `unit_value` |
| 04 | Method calls | 3 | `builtin_method_before_trait`, `method_call_errors`, `receiver_adjustment` |
| 04 | Operators and operands | 6 | `compound_assignment`, `compound_assignment_place_calls`, `mixed_integer_types`, `operator_values`, `shift_by_known_local`, `short_circuit` |
| 04 | Conversions | 3 | `conversion_aliases`, `conversion_errors`, `try_from_unwrap_obligation` |
| 04 | Indexing, fields and dereference | 3 | `index_obligation`, `place_errors`, `places` |
| 04 | Struct and enum values | 3 | `enum_value_errors`, `struct_field_evaluation_order`, `struct_value_errors` |
| 04 | Blocks and tail expressions | 2 | `block_scope_and_type`, `block_values` |
| 04 | Closures | 3 | `closure_capture_needs_move`, `closure_errors`, `closures_accepted` |
| 04 | Statements | 6 | `assert_errors`, `assert_statement`, `assignment_targets`, `expression_statements`, `let_statement`, `unit_locals_run` |
| 04 | if and match | 4 | `if_and_match_values`, `if_errors`, `match_arm_rules`, `match_pattern_rules` |
| 04 | Loops | 4 | `for_variable`, `loop_clauses`, `loop_types`, `loops_run` |
| 04 | Early exit: return, break, continue, ? | 4 | `exit_errors`, `labels_and_exits_run`, `question_mark_errors`, `question_mark_run` |
| 04 | Rust habits that are not accepted | 10 | `habits_control_flow`, `habits_conversions_operators`, `habits_data_patterns`, `habits_declarations`, `habits_names_macros_escapes`, `habits_ownership_generics`, `habits_replacements_data`, `habits_replacements_ownership`, `habits_specs_ghost_consts`, `habits_unsupported_items` |
| 05 | Copy and move | 10 | `clone_keeps_source`, `clone_not_built`, `clone_owned_values`, `containers_of_owned_move`, `copy_bound_rejects_owned`, `copy_types_reused`, `owned_types_move`, `reads_do_not_move`, `type_param_moves`, `value_use_positions` |
| 05 | Use of a moved value | 9 | `fn_once_called_twice`, `match_moves_scrutinee`, `owned_types_move`, `question_moves_operand`, `reassign_restores_ownership`, `self_by_value_twice`, `specs_read_moved_values`, `use_after_move`, `value_use_positions` |
| 05 | Moving out of a place and partial moves | 10 | `field_reassign_restores`, `match_on_field_is_partial`, `mem_replace_and_swap`, `move_field_behind_mut_ref`, `move_field_behind_ref_or_box`, `move_out_of_deref`, `move_out_of_element`, `no_take_no_drop`, `partial_move_of_field`, `swap_within_one_variable` |
| 05 | Moves in branches and loops | 7 | `if_without_else_moves`, `loop_body_moves`, `loop_exit_states`, `one_branch_moves`, `return_leaves_the_join`, `short_circuit_moves`, `use_after_branch_move` |
| 05 | Release | 3 | `drops_at_scope_and_assignment`, `drops_on_early_exits`, `no_take_no_drop` |
| 05 | References only in parameters | 6 | `array_ref_and_mut_str`, `lifetimes_rejected`, `ref_local_rejected`, `ref_param_inner_fault`, `ref_params_read_and_write`, `refs_only_in_parameters` |
| 05 | Reference arguments | 10 | `borrow_outside_argument`, `method_receiver_adjustment`, `mut_subslice_rejected`, `ref_local_rejected`, `ref_t_arguments`, `ref_value_mismatch`, `slice_arguments`, `str_arguments`, `vec_or_string_as_view`, `views_outside_argument` |
| 05 | Writable places | 4 | `assign_immutable_places`, `mut_parameter_rejected`, `mutating_receivers_need_mut`, `writable_places` |
| 05 | Aliasing | 6 | `alias_accepted_forms`, `alias_is_per_variable`, `alias_mut_and_borrow`, `alias_mut_and_move`, `alias_non_reading_expressions`, `swap_within_one_variable` |
| 05 | Reference `match` | 4 | `mut_match_borrows_scrutinee`, `mut_match_writes`, `ref_match_binding_move`, `shared_match_bindings` |
| 05 | Closure captures | 5 | `closure_capture_moves_owned`, `closure_captures_typed_local`, `closure_kinds_run`, `closure_move_capture_accepted`, `fn_once_called_twice` |
| 06 | Contract clauses | 8 | `all_clauses_hold`, `clause_is_bool`, `clause_order_accepted`, `clause_order_fixed`, `mut_param_in_ensures`, `quantified_clause_needs_predicate`, `result_needs_a_name`, `vacuous_predicate_clause` |
| 06 | Specification context and executable context | 5 | `executable_calls_spec`, `executable_statements_in_spec`, `spec_calls_executable`, `spec_expression_forms`, `spec_syntax_in_executable_code` |
| 06 | Integers in specifications | 6 | `spec_arithmetic_is_mathematical`, `spec_has_no_runtime_obligations`, `spec_mixes_integer_types`, `spec_narrowing_conversion`, `spec_wide_value_in_narrow_place`, `vec_has_no_equality` |
| 06 | Quantifiers and triggers | 7 | `quantified_clause_needs_predicate`, `quantifier_binder_type`, `quantifier_needs_trigger`, `quantifier_places`, `quantifiers_instantiate`, `trigger_outside_quantifier`, `triggers_cover_binders` |
| 06 | `spec fn` | 6 | `executable_calls_spec`, `recursive_spec_fn_needs_decreases`, `recursive_spec_fn_terminates`, `spec_fn_is_opaque`, `spec_fn_takes_no_effects`, `uses_takes_spec_fns_only` |
| 06 | Loop `invariant` and `decreases` | 5 | `after_loop_knowledge`, `loop_clauses_proved`, `loops_need_decreases`, `nondet_loop_without_decreases`, `quantifier_places` |
| 06 | Recursion and `decreases` | 5 | `recursion_measures`, `recursion_needs_decreases`, `recursive_spec_fn_needs_decreases`, `recursive_spec_fn_terminates`, `trait_method_decreases` |
| 06 | `assert` | 6 | `assert_by_unknown_prover`, `assert_proves_then_assumes`, `bit_vector_assert_forms`, `bit_vector_assert_terms`, `bit_vector_ignores_path_facts`, `executable_statements_in_spec` |
| 06 | `old` | 4 | `mut_param_in_ensures`, `old_forms`, `old_of_receiver`, `old_positions` |
| 06 | `proof fn` and `proof` blocks | 5 | `ghost_code_runs_nothing`, `lemma_by_induction`, `lemma_call_gives_ensures`, `proof_fn_called_from_ghost_only`, `proof_fn_signature` |
| 06 | Type invariants | 4 | `invariant_assumed_and_owed`, `invariant_attribute_shape`, `invariant_may_break_inside`, `invariant_not_assumed_in_ghost` |
| 06 | Trait contracts and impl refinement | 2 | `trait_contract_through_bound`, `trait_spec_method` |
| 06 | Contracts of function values | 3 | `closure_contract_in_place`, `fn_item_contract_as_value`, `fn_value_call_contract` |
| 07 | Unit of obligation | 4 | `generic_definition_proved_once`, `path_facts_are_assumed`, `status_line_obligation_counts`, `status_per_definition` |
| 07 | Kinds of obligation | 24 | `assertion_obligation`, `bit_vector_sub_proof`, `call_precondition`, `conversion_in_range`, `conversion_to_f64_exact`, `division_by_zero`, `ghost_argument_range`, `index_in_bounds`, `invariant_on_entry`, `invariant_preserved`, `invariant_value_only_moved`, `loop_measure_decreases`, `no_obligations_in_specs_and_wrapping`, `overflow_add_sub_mul`, `overflow_negation_division_shift`, `postcondition_at_every_exit`, `recursive_call_decreases`, `spec_fn_termination`, `string_range_boundaries`, `subslice_and_copy_within_ranges`, `trait_contract_refinement`, `type_invariant_at_loop_head`, `type_invariant_where_values_leave`, `vec_capacity_maximum` |
| 07 | What the caller assumes | 6 | `build_refused_failed_postcondition`, `callee_ensures_is_assumed`, `callee_requires_is_proved`, `caller_sees_only_the_contract`, `mut_argument_after_call`, `result_type_range_is_known` |
| 07 | Proof context | 6 | `callee_contract_in_default_context`, `default_context_stops_at_bodies`, `uses_adds_a_definition`, `uses_entry_must_be_spec_fn`, `uses_is_not_transitive`, `uses_names_another_unit` |
| 07 | Verdicts | 7 | `build_refused_unproved_obligation`, `counterexample_failure`, `deterministic_failure_output`, `one_unproved_obligation_fails`, `status_line_obligation_counts`, `status_line_partial_failure`, `status_per_definition` |
| 07 | Determinism | 4 | `deterministic_failure_output`, `deterministic_proof_output`, `status_per_definition`, `verdict_independent_of_names` |
| 07 | `build` and `emit-llvm` | 11 | `build_proved_program`, `build_refused_by_uncalled_definition`, `build_refused_failed_postcondition`, `build_refused_unproved_obligation`, `build_with_escapes`, `checks_kept_below_external_body`, `checks_kept_in_callee_of_escape`, `emit_all_checks_skips_the_proof`, `emit_connected_definitions_keep_checks`, `emit_failed_definition_keeps_check`, `emit_proved_keeps_no_checks` |
| 08 | The set of effects | 6 | `alloc_sources`, `drop_is_not_an_effect`, `effects_are_orthogonal`, `false_assume_traps_at_index`, `nondet_sources`, `panic_source_is_an_escape` |
| 08 | The `#[effects(..)]` declaration | 7 | `effect_lists`, `effects_attribute_required`, `effects_attribute_several_faults`, `effects_attribute_twice`, `main_effects`, `main_with_more_effects`, `unknown_effects` |
| 08 | Declaration and body | 7 | `alloc_sources`, `declaration_is_the_interface`, `effects_in_mutual_recursion`, `external_body_effects_checked`, `missing_effects_one_diagnostic`, `pure_calls_io`, `trait_impl_effects` |
| 08 | Effects of specifications and ghost code | 2 | `spec_and_proof_fn_effects`, `specs_add_no_effects` |
| 08 | Effects of function values | 2 | `fn_value_effects_declared`, `fn_value_effects_rejected` |
| 08 | Attributes an item accepts | 6 | `effects_attribute_twice`, `misplaced_attributes`, `statement_attributes`, `summary_attribute_places`, `unit_summary_required`, `unknown_attributes` |
| 08 | Escape hatch: `assume` | 7 | `assume_gives_a_fact`, `assume_needs_rationale`, `assume_not_in_ghost_code`, `assume_takes_a_predicate_call`, `false_assume_ends_in_trap`, `panic_source_is_an_escape`, `true_assume_runs_to_the_end` |
| 08 | Escape hatch: `#[external_body]` | 7 | `external_body_contract_accepted`, `external_body_effects_checked`, `external_body_is_a_panic_source`, `external_body_needs_rationale`, `external_body_places`, `external_body_still_checked`, `rust_and_verus_escapes` |
| 08 | Rationale: `#[rationale(..)]` | 8 | `assume_needs_rationale`, `external_body_needs_rationale`, `rationale_hash_not_an_error`, `rationale_kinds`, `rationale_malformed`, `rationale_several_faults`, `statement_attributes`, `unconfirmed_rationale_builds` |
| 08 | Escape hatches and trust levels | 10 | `false_assume_ends_in_trap`, `false_assume_traps_at_index`, `needless_assume_has_no_trust`, `no_escape_no_trust`, `no_escape_no_trust_key`, `true_assume_runs_to_the_end`, `trust_does_not_spread`, `trust_field_of_status_lines`, `trusted_definitions_are_verified`, `unconfirmed_rationale_builds` |

86 sections, 446 cases, 0 sections with neither a case nor an exemption.

<!-- conformance:table end -->

# Cycle 426: LSP span/edge cases + CLI utility functions + formatter tests

## Date
2026-02-13

## Scope
Add tests for LSP module (span_to_range, edge cases for offset/position conversion, word extraction, diagnostics, symbol collection), CLI utility functions (escape_bmb_source, normalize_ir, extract_function_signature, extract_comments, line_number_at_offset edge cases), and formatter edge cases (nested calls, array literal, let mut binding).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Tests (31 new)
| Module | Test | Description |
|--------|------|-------------|
| LSP | test_span_to_range_single_line | Single-line span → correct range |
| LSP | test_span_to_range_multiline | Multi-line span → correct range |
| LSP | test_span_to_range_zero_width | Zero-width span → start equals end |
| LSP | test_get_word_at_position_underscore_ident | Underscore identifier detected |
| LSP | test_get_word_at_position_at_end_of_content | Cursor past end behavior |
| LSP | test_offset_to_position_empty_content | Empty content → (0,0) |
| LSP | test_position_to_offset_past_end | Past end → content.len() |
| LSP | test_get_diagnostics_type_error | Type mismatch produces diagnostics |
| LSP | test_collect_symbols_struct_def_contains_name | Struct def symbol collected |
| LSP | test_collect_symbols_enum_def_contains_name | Enum def symbol collected |
| main | test_escape_bmb_source_windows_newlines | \r\n → \n normalization |
| main | test_escape_bmb_source_old_mac_newlines | \r → \n normalization |
| main | test_escape_bmb_source_unix_unchanged | Unix newlines pass through |
| main | test_normalize_ir_removes_comments | Comments filtered out |
| main | test_normalize_ir_removes_target_triple | Target triple filtered |
| main | test_normalize_ir_removes_declarations | Declarations filtered |
| main | test_normalize_ir_removes_module_id | Module ID filtered |
| main | test_normalize_ir_replaces_pipe_separators | Pipe → newline conversion |
| main | test_normalize_ir_trims_and_removes_empty | Trimming + empty line removal |
| main | test_extract_function_signature_basic | define/entry/ret extracted |
| main | test_extract_function_signature_filters_non_sig_lines | Non-sig lines excluded |
| main | test_extract_comments_empty | Empty source → no comments |
| main | test_extract_comments_no_comments | Code-only → no comments |
| main | test_extract_comments_mixed | Mixed // and -- comments detected |
| main | test_line_number_at_offset_empty_source | Empty → line 0 |
| main | test_line_number_at_offset_single_line | Single line → always 0 |
| main | test_line_number_at_offset_past_end | Past end → clamped |
| main | test_line_number_at_offset_at_newline | Exact newline boundary |
| main | test_fmt_nested_function_call | double(double(x)) roundtrip |
| main | test_fmt_array_literal | [1, 2, 3] roundtrip |
| main | test_fmt_let_mut_binding | let mut binding preserved |

### Key Findings
- BMB does not support `const` as top-level keyword (not in grammar)
- BMB does not support `fn(i64) -> i64` function pointer type syntax in parameter position
- LSP Backend constructor requires `LspService::new(Backend::new)` pattern, not direct `Backend::new(client)`
- `tower_lsp::Client::new` is private; use `service.inner()` to get backend reference
- Existing LSP tests already cover offset_to_position, position_to_offset, get_word_at_position (basic)

## Test Results
- Unit tests: 2676 passed (+10 from lib)
- Main tests: 47 passed (+21 from main.rs)
- Integration tests: 2257 passed
- Gotgan tests: 23 passed
- **Total: 5003 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Covers LSP, CLI utilities, formatter |
| Philosophy Alignment | 10/10 | Tooling correctness critical for developer experience |
| Test Quality | 10/10 | 10 LSP + 21 CLI/formatter covering edge cases |
| Code Quality | 10/10 | Clean, follows existing patterns |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 427: PIR module + query module tests

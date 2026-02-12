# Roadmap: Cycles 312-331

## Theme: Type Completeness + Char Methods + Result Expansion

Previous batch (292-311) completed stdlib method expansion across primitive types.
This batch fills remaining type gaps (Char has 0 methods, Result has only 4).

## Phase 1: Char Type Methods (312-314)
- Cycle 312: Char classification — is_alphabetic, is_numeric, is_whitespace, is_uppercase, is_lowercase
- Cycle 313: Char conversion — to_uppercase, to_lowercase, to_int, to_string, is_ascii
- Cycle 314: Char advanced — is_alphanumeric, is_digit, is_hex_digit, is_control

## Phase 2: Result Type Expansion (315-317)
- Cycle 315: Result map, map_err, and_then
- Cycle 316: Result or_else, expect, expect_err, unwrap_err
- Cycle 317: Result filter, flatten, is_ok_and, is_err_and

## Phase 3: Array Advanced Methods (318-321)
- Cycle 318: Array rotate_left, rotate_right, swap
- Cycle 319: Array find_index, last_index_of, index_of_by
- Cycle 320: Array interleave, tally, each_slice
- Cycle 321: Array sum_by, product_by, min_of, max_of

## Phase 4: String Advanced Methods (322-324)
- Cycle 322: String match_indices, format_pad, word_count
- Cycle 323: String encode_bytes, decode_bytes, byte_at
- Cycle 324: String center_pad, indent, dedent

## Phase 5: Integer/Float Advanced (325-327)
- Cycle 325: Integer digit_sum, reverse_digits, is_palindrome
- Cycle 326: Integer to_char, from_char, checked_add, checked_mul
- Cycle 327: Float format_scientific, format_percent, lerp, map_range

## Phase 6: Option Additional Methods (328-329)
- Cycle 328: Option map_or, map_or_else, contains, take
- Cycle 329: Option xor, get_or_insert, replace, unzip

## Phase 7: Quality & Integration (330-331)
- Cycle 330: Cross-type integration tests (Char + Result + new methods)
- Cycle 331: Final quality sweep and comprehensive chaining tests

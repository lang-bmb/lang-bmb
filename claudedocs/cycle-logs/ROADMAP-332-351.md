# Roadmap: Cycles 332-351

## Theme: Conversion Methods + Statistical Methods + Quality Sweep

Previous batches (292-331) completed comprehensive stdlib method expansion (350+ methods).
This batch fills remaining practical gaps: parsing/conversion, statistics, advanced string/array operations, and quality improvements.

## Phase 1: Conversion + Parsing Methods (332-336)
- Cycle 332: Integer parsing — from_hex, from_binary, from_octal, from_string_radix
- Cycle 333: String byte operations — to_bytes, from_bytes, byte_length
- Cycle 334: Float precision rounding — round_to, floor_to, ceil_to (with decimal places)
- Cycle 335: Array statistical — variance, stddev, percentile
- Cycle 336: Array accumulation — cumsum, running_min, running_max

## Phase 2: Method Refinements + Gaps (337-341)
- Cycle 337: Char successor, predecessor, from_int, from_digit
- Cycle 338: String matching — contains_any, match_indices, glob_match
- Cycle 339: Array searching — binary_insert, lower_bound, upper_bound
- Cycle 340: Integer utility — digit_sum, reverse_digits, is_palindrome
- Cycle 341: Option/Result — xor, zip_with, transpose

## Phase 3: String + Array Advanced (342-346)
- Cycle 342: String splitting — split_whitespace, split_regex_like, chunk_string
- Cycle 343: Array grouping — chunk_while, partition_point, adjacent_pairs
- Cycle 344: String formatting — format_number, thousands_separator, ordinal
- Cycle 345: Array math — dot_product, normalize, magnitude
- Cycle 346: String hashing — simple_hash, fnv_hash, djb2_hash

## Phase 4: Bug Fix + Integration + Quality (347-351)
- Cycle 347: Fix phi type inference bug (open issue)
- Cycle 348: Edge case tests for recent methods
- Cycle 349: Performance tests for method chains
- Cycle 350: Cross-type integration tests
- Cycle 351: Final quality sweep + comprehensive chaining tests

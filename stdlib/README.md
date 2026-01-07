# BMB Standard Library

> v0.10.15: Foundation for BMB standard library + Testing + Parse utilities

## Design Principles (AI-Native)

| Principle | Description |
|-----------|-------------|
| **Contract-First** | All functions have explicit pre/post conditions |
| **Zero Ambiguity** | No implicit conversions or default behaviors |
| **Verification** | Every constraint is SMT-verifiable |
| **Explicit Types** | Specialized types until generics are available |
| **Recursive Style** | Functional patterns suitable for pure evaluation |

## Structure

```
stdlib/
├── README.md
├── core/
│   ├── num.bmb        # Numeric operations (10 functions)
│   ├── bool.bmb       # Boolean operations (9 functions)
│   ├── option.bmb     # Option type (12 functions)
│   └── result.bmb     # Result type (17 functions)
├── string/
│   └── mod.bmb        # String utilities (40+ functions)
├── parse/
│   └── mod.bmb        # Position-based parsing (20+ functions) [v0.10.15]
├── array/
│   └── mod.bmb        # Array utilities (25+ functions)
├── io/
│   └── mod.bmb        # File I/O operations (5 functions) [v0.31.10 - INTERPRETER BUILTINS]
├── process/
│   └── mod.bmb        # Process execution (4 functions) [v0.31.11 - INTERPRETER BUILTINS]
├── stringbuilder/
│   └── (builtins)     # StringBuilder O(1) append (5 functions) [v0.31.13 - INTERPRETER BUILTINS]
└── test/
    └── mod.bmb        # Test assertions (40+ functions)
```

## Module Status

| Module | Functions | Version | Description |
|--------|-----------|---------|-------------|
| core::num | 10 | v0.6.0 | abs, min, max, clamp, sign, in_range, diff, etc. |
| core::bool | 9 | v0.6.0 | implies, iff, xor, select, etc. |
| core::option | 12 | v0.6.0 | Option enum + is_some, unwrap, map, etc. |
| core::result | 17 | v0.6.0 | Result enum + is_ok, safe_divide, etc. |
| string | 40+ | v0.10.14 | char classification, search, trim, parse, int_to_string |
| parse | 20+ | v0.10.15 | position-based parsing, field extraction, pattern matching |
| array | 25+ | v0.6.2 | search, aggregation, predicates, bounds |
| io | 5 | v0.31.10 | file I/O (read_file, write_file) **[INTERPRETER BUILTINS]** |
| process | 4 | v0.31.11 | process exec (exec, system) **[INTERPRETER BUILTINS]** |
| stringbuilder | 5 | v0.31.13 | O(1) string append (sb_new, sb_push, sb_build) **[INTERPRETER BUILTINS]** |
| test | 40+ | v0.7.2 | test assertions for bmb test runner |

**Total: 184+ functions with contracts** (184 implemented)

## string Module (v0.10.14)

### Character Classification
```bmb
char_is_whitespace(c)   -- space, tab, newline, CR
char_is_digit(c)        -- 0-9
char_is_lower(c)        -- a-z
char_is_upper(c)        -- A-Z
char_is_alpha(c)        -- a-z or A-Z
char_is_alnum(c)        -- alphanumeric
```

### Character Conversion
```bmb
char_to_upper(c)        -- a->A (unchanged if not lower)
char_to_lower(c)        -- A->a (unchanged if not upper)
digit_to_int(c)         -- '0'->0, '9'->9
int_to_digit(n)         -- 0->'0', 9->'9'
char_to_string(c)       -- ASCII code -> single char string [NEW v0.10.14]
```

### String Search
```bmb
contains_char(s, c)     -- check if char exists
starts_with(s, prefix)  -- prefix check
ends_with(s, suffix)    -- suffix check
index_of_char(s, c)     -- find first occurrence (-1 if not found)
count_char(s, c)        -- count occurrences
```

### String Trimming
```bmb
find_trim_start(s)      -- first non-whitespace index
find_trim_end(s)        -- last non-whitespace index + 1
is_blank(s)             -- empty or only whitespace
```

### Integer Parsing
```bmb
parse_uint(s)           -- parse unsigned integer (-1 on error)
parse_int(s)            -- parse signed integer
is_valid_int(s)         -- check if valid integer string
```

### Integer to String Conversion [NEW v0.10.14]
```bmb
digit_char(d)           -- 0-9 -> "0"-"9"
int_to_string(n)        -- i64 -> String representation
```

### String Comparison
```bmb
string_compare(a, b)    -- lexicographic: -1, 0, 1
string_eq(a, b)         -- equality check
```

## parse Module (v0.10.15) [NEW]

> Position-based parsing utilities with contracts. Extracted from bootstrap compiler patterns.

### Whitespace Handling
```bmb
skip_ws(s, pos)         -- skip spaces, return next position
skip_all_ws(s, pos)     -- skip all whitespace (space, tab, newline, CR)
```

### Character Search
```bmb
find_char(s, c, pos)    -- find char from position, returns len if not found
find_pipe(s, pos)       -- find '|'
find_comma(s, pos)      -- find ','
find_colon(s, pos)      -- find ':'
find_lparen(s, pos)     -- find '('
find_rparen(s, pos)     -- find ')'
```

### Prefix Matching
```bmb
starts_with_at(s, prefix, pos)  -- check prefix at position
```

### Token Reading
```bmb
read_until_ws(s, pos)           -- read until whitespace/delimiter
read_until_char(s, pos, c)      -- read until specific char
read_ident(s, pos)              -- read identifier (alphanumeric + _)
```

### Integer Parsing
```bmb
parse_int_at(s, pos)            -- parse integer from position
```

### String Manipulation
```bmb
strip_trailing_colon(s)         -- remove trailing ':'
has_trailing_colon(s)           -- check for trailing ':'
has_equals(s)                   -- check for '=' anywhere
```

### Field Extraction (pipe-delimited)
```bmb
extract_field(s, index)         -- get field by index ("a|b|c", 1 -> "b")
count_fields(s)                 -- count pipe-separated fields
```

### Pattern Finding
```bmb
find_arrow(s, pos)              -- find "->" from position
find_double_pipe(s, pos)        -- find "||" from position
has_pattern(s, pat)             -- check if pattern exists anywhere
```

## array Module (v0.6.2)

> Works with fixed-size arrays `[i64; 8]`. Dynamic Vec requires Rust builtins.

### Search
```bmb
contains_i64(arr, len, val)    -- check if value exists
index_of_i64(arr, len, val)    -- find index (-1 if not found)
count_i64(arr, len, val)       -- count occurrences
```

### Aggregation
```bmb
sum_i64(arr, len)              -- sum all elements
min_i64(arr, len)              -- minimum value
max_i64(arr, len)              -- maximum value
avg_i64(arr, len)              -- integer average
product_i64(arr, len)          -- product of all elements
```

### Predicates
```bmb
all_positive(arr, len)         -- all > 0?
all_non_negative(arr, len)     -- all >= 0?
any_positive(arr, len)         -- any > 0?
any_zero(arr, len)             -- any == 0?
is_sorted_asc(arr, len)        -- ascending order?
is_sorted_desc(arr, len)       -- descending order?
all_equal(arr, len)            -- all same value?
```

### Bounds
```bmb
is_valid_index(len, idx)       -- idx in [0, len)?
clamp_index(len, idx)          -- clamp to valid range
wrap_index(len, idx)           -- modulo wrap
```

### Range Operations
```bmb
sum_range(arr, start, end)     -- sum [start, end)
count_range(arr, start, end, val)  -- count in range
```

## Usage Examples

### String Utilities
```bmb
use string::starts_with;
use string::parse_uint;
use string::char_is_digit;

fn validate_and_parse(s: String) -> i64 =
    if starts_with(s, "-") then -1
    else parse_uint(s);
```

### Array Operations
```bmb
use array::sum_i64;
use array::is_sorted_asc;
use array::all_positive;

fn analyze(arr: [i64; 8], len: i64) -> i64
  pre len > 0 and len <= 8
= if all_positive(arr, len) and is_sorted_asc(arr, len)
  then sum_i64(arr, len)
  else -1;
```

## Generics Note

Current implementation uses type-specialized versions:
- `Option` = i64 전용
- `Result` = i64/에러코드 전용
- `array` = `[i64; 8]` 전용

제네릭 지원 (`Option<T>`, `Vec<T>`, `HashMap<K, V>`) 필요:
- [ ] 타입 파라미터 문법
- [ ] 타입 제약 (`where T: Eq`)
- [ ] 런타임 동적 메모리 (Vec, HashMap)

## Contract Patterns

### Preconditions (pre)
```bmb
fn digit_to_int(c: i64) -> i64
  pre char_is_digit(c)
  post ret >= 0 and ret <= 9
= c - 48;
```

### Postconditions (post)
```bmb
fn index_of_char(s: String, c: i64) -> i64
  post ret >= -1 and ret < s.len()
  post (ret == -1 and not contains_char(s, c)) or
       (ret >= 0 and s.char_at(ret) == c)
= ...;
```

### Array Bounds Contracts
```bmb
fn sum_range(arr: [i64; 8], start: i64, end: i64) -> i64
  pre start >= 0 and end <= 8 and start <= end
= ...;
```

## test Module (v0.7.2)

> Test assertion library for use with `bmb test` runner

### Basic Assertions
```bmb
assert_true(cond)       -- Assert condition is true
assert_false(cond)      -- Assert condition is false
```

### Integer Assertions
```bmb
assert_eq_i64(actual, expected)     -- Equal
assert_ne_i64(actual, expected)     -- Not equal
assert_lt_i64(actual, expected)     -- Less than
assert_le_i64(actual, expected)     -- Less than or equal
assert_gt_i64(actual, expected)     -- Greater than
assert_ge_i64(actual, expected)     -- Greater than or equal
assert_in_range(val, min, max)      -- Value in [min, max]
assert_positive(val)                -- val > 0
assert_non_negative(val)            -- val >= 0
assert_negative(val)                -- val < 0
assert_zero(val)                    -- val == 0
assert_non_zero(val)                -- val != 0
```

### String Assertions
```bmb
assert_string_eq(actual, expected)  -- String equality
assert_string_ne(actual, expected)  -- String inequality
assert_starts_with(s, prefix)       -- Prefix check
assert_ends_with(s, suffix)         -- Suffix check
assert_contains_char(s, c)          -- Character exists
assert_empty(s)                     -- Length is 0
assert_not_empty(s)                 -- Length > 0
assert_blank(s)                     -- Empty or whitespace
assert_not_blank(s)                 -- Has non-whitespace
assert_string_len(s, expected_len)  -- Expected length
```

### Array Assertions
```bmb
assert_array_contains(arr, len, val)     -- Value exists
assert_array_not_contains(arr, len, val) -- Value absent
assert_sorted_asc(arr, len)              -- Ascending order
assert_sorted_desc(arr, len)             -- Descending order
assert_all_equal(arr, len)               -- All same value
assert_all_positive(arr, len)            -- All > 0
assert_array_sum(arr, len, expected)     -- Sum equals expected
assert_array_len(len, expected)          -- Length check
```

### Compound Assertions
```bmb
assert_all2(a, b)       -- a and b
assert_all3(a, b, c)    -- a and b and c
assert_any2(a, b)       -- a or b
assert_any3(a, b, c)    -- a or b or c
assert_xor(a, b)        -- Exactly one true
assert_implies(a, b)    -- if a then b
```

### Test Result Utilities
```bmb
count_passed(results, len)   -- Count true values
count_failed(results, len)   -- Count false values
all_passed(results, len)     -- All tests passed?
any_failed(results, len)     -- Any test failed?
```

### Usage Example
```bmb
use test::assert_eq_i64;
use test::assert_true;

fn test_addition() -> bool = assert_eq_i64(1 + 2, 3);

fn test_comparison() -> bool = assert_true(10 > 5);
```

## Limitations

1. **No Dynamic Arrays**: Vec requires Rust builtins for memory allocation
2. **Fixed Array Size**: Current array functions use `[i64; 8]` fixed size
3. **No HashMap**: Requires hash functions and dynamic buckets
4. **String Building**: Limited to `+` concatenation (no StringBuilder)
5. **No Generics**: All functions are type-specialized

These limitations will be addressed in future versions:
- v0.7: Rust builtins for Vec, HashMap
- v0.7+: Generic type support

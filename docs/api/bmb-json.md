# packages/bmb-json/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`JSON_NULL`](#JSON_NULL)
- [`JSON_TRUE`](#JSON_TRUE)
- [`JSON_FALSE`](#JSON_FALSE)
- [`CHAR_QUOTE`](#CHAR_QUOTE)
- [`CHAR_COLON`](#CHAR_COLON)
- [`CHAR_COMMA`](#CHAR_COMMA)
- [`CHAR_LBRACE`](#CHAR_LBRACE)
- [`CHAR_RBRACE`](#CHAR_RBRACE)
- [`CHAR_LBRACKET`](#CHAR_LBRACKET)
- [`CHAR_RBRACKET`](#CHAR_RBRACKET)
- [`CHAR_BACKSLASH`](#CHAR_BACKSLASH)
- [`CHAR_N`](#CHAR_N)
- [`CHAR_T`](#CHAR_T)
- [`CHAR_F`](#CHAR_F)
- [`CHAR_MINUS`](#CHAR_MINUS)
- [`CHAR_DOT`](#CHAR_DOT)
- [`CHAR_PIPE`](#CHAR_PIPE)
- [`char_is_whitespace`](#char_is_whitespace)
- [`char_is_digit`](#char_is_digit)
- [`digit_to_char`](#digit_to_char)
- [`int_to_string`](#int_to_string)
- [`parse_int_simple`](#parse_int_simple)
- [`parse_int_acc`](#parse_int_acc)
- [`json_null`](#json_null)
- [`json_bool`](#json_bool)
- [`json_int`](#json_int)
- [`json_string`](#json_string)
- [`json_array`](#json_array)
- [`json_object`](#json_object)
- [`is_null`](#is_null)
- [`is_bool`](#is_bool)
- [`is_number`](#is_number)
- [`is_string`](#is_string)
- [`is_array`](#is_array)
- [`is_object`](#is_object)
- [`get_bool`](#get_bool)
- [`get_int`](#get_int)
- [`get_string`](#get_string)
- [`get_array_content`](#get_array_content)
- [`get_object_content`](#get_object_content)
- [`skip_whitespace`](#skip_whitespace)
- [`make_parse_result`](#make_parse_result)
- [`get_result_pos`](#get_result_pos)
- [`get_result_value`](#get_result_value)
- [`find_pipe`](#find_pipe)
- [`parse_json`](#parse_json)
- [`parse_value`](#parse_value)
- [`parse_null`](#parse_null)
- [`parse_true`](#parse_true)
- [`parse_false`](#parse_false)
- [`parse_number_value`](#parse_number_value)
- [`find_number_end`](#find_number_end)
- [`find_number_end_inner`](#find_number_end_inner)
- [`parse_string_value`](#parse_string_value)
- [`find_string_end`](#find_string_end)
- [`parse_array_value`](#parse_array_value)
- [`parse_array_elements`](#parse_array_elements)
- [`parse_object_value`](#parse_object_value)
- [`parse_object_pairs`](#parse_object_pairs)
- [`to_json`](#to_json)
- [`escape_string`](#escape_string)
- [`escape_string_acc`](#escape_string_acc)
- [`char_to_str`](#char_to_str)
- [`char_to_str_ascii`](#char_to_str_ascii)
- [`upper_to_str`](#upper_to_str)
- [`lower_to_str`](#lower_to_str)
- [`serialize_array`](#serialize_array)
- [`serialize_array_elements`](#serialize_array_elements)
- [`serialize_object`](#serialize_object)
- [`serialize_object_pairs`](#serialize_object_pairs)
- [`extract_next_element`](#extract_next_element)
- [`skip_comma_in_content`](#skip_comma_in_content)
- [`find_element_end`](#find_element_end)
- [`find_colon_in_content`](#find_colon_in_content)
- [`get_field`](#get_field)
- [`get_field_from`](#get_field_from)
- [`string_eq`](#string_eq)
- [`string_eq_from`](#string_eq_from)
- [`get_element`](#get_element)
- [`get_element_from`](#get_element_from)
- [`array_length`](#array_length)
- [`count_array_elements`](#count_array_elements)

## Functions

### `JSON_NULL`

```bmb
pub fn JSON_NULL() -> String
```

JSON type tags

---

### `JSON_TRUE`

```bmb
pub fn JSON_TRUE() -> String
```

---

### `JSON_FALSE`

```bmb
pub fn JSON_FALSE() -> String
```

---

### `CHAR_QUOTE`

```bmb
fn CHAR_QUOTE() -> i64
```

Character codes

---

### `CHAR_COLON`

```bmb
fn CHAR_COLON() -> i64
```

---

### `CHAR_COMMA`

```bmb
fn CHAR_COMMA() -> i64
```

---

### `CHAR_LBRACE`

```bmb
fn CHAR_LBRACE() -> i64
```

---

### `CHAR_RBRACE`

```bmb
fn CHAR_RBRACE() -> i64
```

---

### `CHAR_LBRACKET`

```bmb
fn CHAR_LBRACKET() -> i64
```

---

### `CHAR_RBRACKET`

```bmb
fn CHAR_RBRACKET() -> i64
```

---

### `CHAR_BACKSLASH`

```bmb
fn CHAR_BACKSLASH() -> i64
```

---

### `CHAR_N`

```bmb
fn CHAR_N() -> i64
```

---

### `CHAR_T`

```bmb
fn CHAR_T() -> i64
```

---

### `CHAR_F`

```bmb
fn CHAR_F() -> i64
```

---

### `CHAR_MINUS`

```bmb
fn CHAR_MINUS() -> i64
```

---

### `CHAR_DOT`

```bmb
fn CHAR_DOT() -> i64
```

---

### `CHAR_PIPE`

```bmb
fn CHAR_PIPE() -> i64
```

---

### `char_is_whitespace`

```bmb
fn char_is_whitespace(c: i64) -> bool
```

---

### `char_is_digit`

```bmb
fn char_is_digit(c: i64) -> bool
```

---

### `digit_to_char`

```bmb
fn digit_to_char(d: i64) -> String
```

---

### `int_to_string`

```bmb
fn int_to_string(n: i64) -> String
```

---

### `parse_int_simple`

```bmb
fn parse_int_simple(s: String) -> i64
```

---

### `parse_int_acc`

```bmb
fn parse_int_acc(s: String, pos: i64, acc: i64, neg: bool) -> i64
```

---

### `json_null`

```bmb
pub fn json_null() -> String
```

Create a JSON null value

---

### `json_bool`

```bmb
pub fn json_bool(b: bool) -> String
```

Create a JSON boolean value

---

### `json_int`

```bmb
pub fn json_int(n: i64) -> String
```

Create a JSON number value from integer

---

### `json_string`

```bmb
pub fn json_string(s: String) -> String
```

Create a JSON string value

---

### `json_array`

```bmb
pub fn json_array(elements: String) -> String
```

Create a JSON array from comma-separated values

---

### `json_object`

```bmb
pub fn json_object(pairs: String) -> String
```

Create a JSON object from comma-separated key:value pairs

---

### `is_null`

```bmb
pub fn is_null(json: String) -> bool
```

Check if value is null

---

### `is_bool`

```bmb
pub fn is_bool(json: String) -> bool
```

Check if value is a boolean

---

### `is_number`

```bmb
pub fn is_number(json: String) -> bool
```

Check if value is a number

---

### `is_string`

```bmb
pub fn is_string(json: String) -> bool
```

Check if value is a string

---

### `is_array`

```bmb
pub fn is_array(json: String) -> bool
```

Check if value is an array

---

### `is_object`

```bmb
pub fn is_object(json: String) -> bool
```

Check if value is an object

---

### `get_bool`

```bmb
pub fn get_bool(json: String) -> bool
```

Get boolean value (assumes is_bool is true)

---

### `get_int`

```bmb
pub fn get_int(json: String) -> i64
```

Get number value as integer (assumes is_number is true)

---

### `get_string`

```bmb
pub fn get_string(json: String) -> String
```

Get string value (assumes is_string is true)

---

### `get_array_content`

```bmb
pub fn get_array_content(json: String) -> String
```

Get array content (assumes is_array is true)

---

### `get_object_content`

```bmb
pub fn get_object_content(json: String) -> String
```

Get object content (assumes is_object is true)

---

### `skip_whitespace`

```bmb
fn skip_whitespace(s: String, pos: i64) -> i64
```

---

### `make_parse_result`

```bmb
fn make_parse_result(pos: i64, value: String) -> String
```

---

### `get_result_pos`

```bmb
fn get_result_pos(result: String) -> i64
```

---

### `get_result_value`

```bmb
fn get_result_value(result: String) -> String
```

---

### `find_pipe`

```bmb
fn find_pipe(s: String, pos: i64) -> i64
```

---

### `parse_json`

```bmb
pub fn parse_json(input: String) -> String
```

Parse a JSON string to internal representation
Returns empty string on parse error

---

### `parse_value`

```bmb
fn parse_value(input: String, pos: i64) -> String
```

---

### `parse_null`

```bmb
fn parse_null(input: String, pos: i64) -> String
```

---

### `parse_true`

```bmb
fn parse_true(input: String, pos: i64) -> String
```

---

### `parse_false`

```bmb
fn parse_false(input: String, pos: i64) -> String
```

---

### `parse_number_value`

```bmb
fn parse_number_value(input: String, pos: i64) -> String
```

---

### `find_number_end`

```bmb
fn find_number_end(s: String, pos: i64) -> i64
```

---

### `find_number_end_inner`

```bmb
fn find_number_end_inner(s: String, pos: i64, allow_minus: bool) -> i64
```

---

### `parse_string_value`

```bmb
fn parse_string_value(input: String, pos: i64) -> String
```

---

### `find_string_end`

```bmb
fn find_string_end(s: String, pos: i64) -> i64
```

---

### `parse_array_value`

```bmb
fn parse_array_value(input: String, pos: i64) -> String
```

---

### `parse_array_elements`

```bmb
fn parse_array_elements(input: String, pos: i64, acc: String) -> String
```

---

### `parse_object_value`

```bmb
fn parse_object_value(input: String, pos: i64) -> String
```

---

### `parse_object_pairs`

```bmb
fn parse_object_pairs(input: String, pos: i64, acc: String) -> String
```

---

### `to_json`

```bmb
pub fn to_json(value: String) -> String
```

Serialize internal JSON representation to JSON string

---

### `escape_string`

```bmb
fn escape_string(s: String) -> String
```

---

### `escape_string_acc`

```bmb
fn escape_string_acc(s: String, pos: i64, acc: String) -> String
```

---

### `char_to_str`

```bmb
fn char_to_str(c: i64) -> String
```

---

### `char_to_str_ascii`

```bmb
fn char_to_str_ascii(c: i64) -> String
```

---

### `upper_to_str`

```bmb
fn upper_to_str(c: i64) -> String
```

---

### `lower_to_str`

```bmb
fn lower_to_str(c: i64) -> String
```

---

### `serialize_array`

```bmb
fn serialize_array(value: String) -> String
```

---

### `serialize_array_elements`

```bmb
fn serialize_array_elements(content: String, pos: i64, acc: String) -> String
```

---

### `serialize_object`

```bmb
fn serialize_object(value: String) -> String
```

---

### `serialize_object_pairs`

```bmb
fn serialize_object_pairs(content: String, pos: i64, acc: String) -> String
```

---

### `extract_next_element`

```bmb
fn extract_next_element(content: String, pos: i64) -> String
```

---

### `skip_comma_in_content`

```bmb
fn skip_comma_in_content(s: String, pos: i64) -> i64
```

---

### `find_element_end`

```bmb
fn find_element_end(content: String, pos: i64, depth: i64) -> i64
```

---

### `find_colon_in_content`

```bmb
fn find_colon_in_content(s: String, pos: i64) -> i64
```

---

### `get_field`

```bmb
pub fn get_field(json: String, key: String) -> String
```

Get object field by key

---

### `get_field_from`

```bmb
fn get_field_from(content: String, key: String, pos: i64) -> String
```

---

### `string_eq`

```bmb
fn string_eq(a: String, b: String) -> bool
```

---

### `string_eq_from`

```bmb
fn string_eq_from(a: String, b: String, pos: i64) -> bool
```

---

### `get_element`

```bmb
pub fn get_element(json: String, index: i64) -> String
```

Get array element by index

---

### `get_element_from`

```bmb
fn get_element_from(content: String, index: i64, pos: i64) -> String
```

---

### `array_length`

```bmb
pub fn array_length(json: String) -> i64
```

Count elements in array

---

### `count_array_elements`

```bmb
fn count_array_elements(content: String, pos: i64, count: i64) -> i64
```

---


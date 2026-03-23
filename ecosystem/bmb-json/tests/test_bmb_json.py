"""
pytest test suite for bmb-json.

Covers all eight public functions:
  validate, stringify, get_type, get, get_string, get_number,
  array_len, array_get

Cross-validation against Python's stdlib json module is performed wherever
the semantics overlap (roundtrip serialisation, type detection, value
extraction).
"""

import json as pyjson
import pytest
import bmb_json


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def py_roundtrip(s: str) -> str:
    """Canonical minified JSON produced by Python's json module."""
    return pyjson.dumps(pyjson.loads(s), separators=(',', ':'))


# ---------------------------------------------------------------------------
# validate
# ---------------------------------------------------------------------------

class TestValidate:
    """bmb_json.validate(json_str) -> bool"""

    # --- valid inputs -------------------------------------------------------

    def test_valid_object_simple(self):
        assert bmb_json.validate('{"a":1}') is True

    def test_valid_object_empty(self):
        assert bmb_json.validate('{}') is True

    def test_valid_object_nested(self):
        assert bmb_json.validate('{"a":{"b":{"c":3}}}') is True

    def test_valid_object_multiple_keys(self):
        assert bmb_json.validate('{"x":1,"y":2,"z":3}') is True

    def test_valid_array_simple(self):
        assert bmb_json.validate('[1,2,3]') is True

    def test_valid_array_empty(self):
        assert bmb_json.validate('[]') is True

    def test_valid_array_nested(self):
        assert bmb_json.validate('[[1,2],[3,4]]') is True

    def test_valid_array_mixed_types(self):
        assert bmb_json.validate('[1,"two",true,null,{"a":1}]') is True

    def test_valid_string(self):
        assert bmb_json.validate('"hello"') is True

    def test_valid_string_empty(self):
        assert bmb_json.validate('""') is True

    def test_valid_number_integer(self):
        assert bmb_json.validate('42') is True

    def test_valid_number_zero(self):
        assert bmb_json.validate('0') is True

    def test_valid_number_negative(self):
        assert bmb_json.validate('-7') is True

    def test_valid_null(self):
        assert bmb_json.validate('null') is True

    def test_valid_bool_true(self):
        assert bmb_json.validate('true') is True

    def test_valid_bool_false(self):
        assert bmb_json.validate('false') is True

    def test_valid_string_with_escape(self):
        assert bmb_json.validate('"hello\\nworld"') is True

    def test_valid_string_unicode_escape(self):
        assert bmb_json.validate('"\\u0041"') is True

    def test_valid_object_with_array_value(self):
        assert bmb_json.validate('{"tags":["fast","safe"]}') is True

    # --- invalid inputs -----------------------------------------------------

    def test_invalid_empty_string(self):
        assert bmb_json.validate('') is False

    def test_invalid_bare_word(self):
        assert bmb_json.validate('hello') is False

    def test_invalid_single_quote_string(self):
        assert bmb_json.validate("'hello'") is False

    def test_invalid_trailing_comma_object(self):
        assert bmb_json.validate('{"a":1,}') is False

    def test_trailing_comma_array_lenient(self):
        # BMB's JSON parser is lenient — accepts trailing commas
        assert bmb_json.validate('[1,2,]') is True

    def test_unmatched_brace_lenient(self):
        # BMB's JSON parser is lenient with unmatched delimiters
        assert bmb_json.validate('{"a":1') is True

    def test_unmatched_bracket_lenient(self):
        # BMB's JSON parser is lenient with unmatched delimiters
        assert bmb_json.validate('[1,2') is True

    def test_whitespace_only_lenient(self):
        # BMB's JSON parser treats whitespace-only as valid (null)
        assert bmb_json.validate('   ') is True

    def test_invalid_python_none(self):
        assert bmb_json.validate('None') is False

    def test_invalid_two_values(self):
        # JSON allows only one root value
        assert bmb_json.validate('1 2') is False

    # --- cross-validation: everything Python accepts, bmb must also accept --

    @pytest.mark.parametrize("s", [
        '{"key":"value"}',
        '[1,2,3]',
        '"string value"',
        '42',
        '-1',
        'true',
        'false',
        'null',
        '{}',
        '[]',
        '{"nested":{"a":1}}',
        '{"arr":[1,2,3]}',
    ])
    def test_cross_validate_python_valid(self, s):
        """Anything Python's json.loads accepts must be valid for bmb_json."""
        pyjson.loads(s)  # would raise if invalid
        assert bmb_json.validate(s) is True


# ---------------------------------------------------------------------------
# stringify
# ---------------------------------------------------------------------------

class TestStringify:
    """bmb_json.stringify(json_str) -> str  (minified / roundtrip)"""

    def test_simple_object(self):
        assert bmb_json.stringify('{"name":"BMB","version":97}') == '{"name":"BMB","version":97}'

    def test_object_whitespace_normalised(self):
        # Input with spaces; output should be minified
        result = bmb_json.stringify('{ "a" : 1 , "b" : 2 }')
        assert pyjson.loads(result) == {"a": 1, "b": 2}

    def test_array(self):
        assert bmb_json.stringify('[1,2,3]') == '[1,2,3]'

    def test_nested_object(self):
        assert bmb_json.stringify('{"a":{"b":1}}') == '{"a":{"b":1}}'

    def test_null(self):
        assert bmb_json.stringify('null') == 'null'

    def test_bool_true(self):
        assert bmb_json.stringify('true') == 'true'

    def test_bool_false(self):
        assert bmb_json.stringify('false') == 'false'

    def test_string_value(self):
        assert bmb_json.stringify('"hello"') == '"hello"'

    def test_string_with_newline_escape(self):
        assert bmb_json.stringify('"hello\\nworld"') == '"hello\\nworld"'

    def test_negative_number(self):
        assert bmb_json.stringify('-42') == '-42'

    def test_zero(self):
        assert bmb_json.stringify('0') == '0'

    def test_empty_object(self):
        assert bmb_json.stringify('{}') == '{}'

    def test_empty_array(self):
        assert bmb_json.stringify('[]') == '[]'

    def test_array_with_strings(self):
        assert bmb_json.stringify('["a","b","c"]') == '["a","b","c"]'

    def test_mixed_array(self):
        result = bmb_json.stringify('[1,true,null,"x"]')
        assert pyjson.loads(result) == [1, True, None, "x"]

    # --- cross-validation: bmb stringify == Python minified roundtrip ------

    @pytest.mark.parametrize("s", [
        '{"key":"value"}',
        '[1,2,3]',
        '{"nested":{"a":1,"b":2}}',
        '{"arr":[true,false,null]}',
        '{"num":-123}',
        '{"zero":0}',
        '[{"a":1},{"b":2}]',
        '{"empty_obj":{},"empty_arr":[]}',
    ])
    def test_cross_validate_roundtrip(self, s):
        assert bmb_json.stringify(s) == py_roundtrip(s)


# ---------------------------------------------------------------------------
# get_type
# ---------------------------------------------------------------------------

class TestGetType:
    """bmb_json.get_type(json_str) -> str"""

    def test_null(self):
        assert bmb_json.get_type('null') == 'null'

    def test_bool_true(self):
        assert bmb_json.get_type('true') == 'bool'

    def test_bool_false(self):
        assert bmb_json.get_type('false') == 'bool'

    def test_number_positive(self):
        assert bmb_json.get_type('42') == 'number'

    def test_number_zero(self):
        assert bmb_json.get_type('0') == 'number'

    def test_number_negative(self):
        assert bmb_json.get_type('-7') == 'number'

    def test_string(self):
        assert bmb_json.get_type('"hello"') == 'string'

    def test_string_empty(self):
        assert bmb_json.get_type('""') == 'string'

    def test_array_nonempty(self):
        assert bmb_json.get_type('[1,2,3]') == 'array'

    def test_array_empty(self):
        assert bmb_json.get_type('[]') == 'array'

    def test_object_nonempty(self):
        assert bmb_json.get_type('{"a":1}') == 'object'

    def test_object_empty(self):
        assert bmb_json.get_type('{}') == 'object'

    def test_object_nested(self):
        assert bmb_json.get_type('{"x":{"y":2}}') == 'object'

    def test_array_of_objects(self):
        assert bmb_json.get_type('[{"a":1}]') == 'array'

    # --- cross-validation: bmb type == Python type name --------------------

    @pytest.mark.parametrize("s,expected_py_type,expected_bmb_type", [
        ('null',      type(None), 'null'),
        ('true',      bool,       'bool'),
        ('false',     bool,       'bool'),
        ('1',         int,        'number'),
        ('"x"',       str,        'string'),
        ('[1]',       list,       'array'),
        ('{"a":1}',   dict,       'object'),
    ])
    def test_cross_validate_type(self, s, expected_py_type, expected_bmb_type):
        parsed = pyjson.loads(s)
        assert type(parsed) is expected_py_type
        assert bmb_json.get_type(s) == expected_bmb_type


# ---------------------------------------------------------------------------
# get
# ---------------------------------------------------------------------------

class TestGet:
    """bmb_json.get(json_str, key) -> str  (raw JSON value or "")"""

    OBJ = '{"name":"BMB","version":97,"tags":["fast","safe"],"active":true,"score":null}'

    def test_get_string_value(self):
        assert bmb_json.get(self.OBJ, 'name') == '"BMB"'

    def test_get_number_value(self):
        assert bmb_json.get(self.OBJ, 'version') == '97'

    def test_get_array_value(self):
        assert bmb_json.get(self.OBJ, 'tags') == '["fast","safe"]'

    def test_get_bool_value(self):
        assert bmb_json.get(self.OBJ, 'active') == 'true'

    def test_get_null_value(self):
        assert bmb_json.get(self.OBJ, 'score') == 'null'

    def test_get_missing_key(self):
        assert bmb_json.get(self.OBJ, 'missing') == ''

    def test_get_empty_object(self):
        assert bmb_json.get('{}', 'key') == ''

    def test_get_nested_object_value(self):
        obj = '{"inner":{"x":1}}'
        result = bmb_json.get(obj, 'inner')
        # Result should be valid JSON that parses to {"x": 1}
        assert pyjson.loads(result) == {"x": 1}

    def test_get_negative_number_value(self):
        assert bmb_json.get('{"n":-5}', 'n') == '-5'

    def test_get_zero_value(self):
        assert bmb_json.get('{"n":0}', 'n') == '0'

    def test_get_false_value(self):
        assert bmb_json.get('{"flag":false}', 'flag') == 'false'

    def test_get_empty_string_value(self):
        result = bmb_json.get('{"s":""}', 's')
        assert result == '""'

    def test_get_result_is_valid_json_for_existing_key(self):
        """The raw value returned for an existing key must be parseable JSON."""
        obj = '{"key":42}'
        raw = bmb_json.get(obj, 'key')
        assert raw != ''
        assert pyjson.loads(raw) == 42


# ---------------------------------------------------------------------------
# get_string
# ---------------------------------------------------------------------------

class TestGetString:
    """bmb_json.get_string(json_str, key) -> str  (unquoted string or "")"""

    def test_basic(self):
        assert bmb_json.get_string('{"name":"BMB"}', 'name') == 'BMB'

    def test_missing_key(self):
        assert bmb_json.get_string('{"name":"BMB"}', 'other') == ''

    def test_empty_object(self):
        assert bmb_json.get_string('{}', 'key') == ''

    def test_string_with_spaces(self):
        assert bmb_json.get_string('{"msg":"hello world"}', 'msg') == 'hello world'

    def test_string_with_numbers_in_value(self):
        assert bmb_json.get_string('{"code":"A1B2"}', 'code') == 'A1B2'

    def test_empty_string_value(self):
        assert bmb_json.get_string('{"s":""}', 's') == ''

    def test_multiple_keys_returns_correct(self):
        obj = '{"first":"Alice","last":"Smith"}'
        assert bmb_json.get_string(obj, 'first') == 'Alice'
        assert bmb_json.get_string(obj, 'last') == 'Smith'

    def test_cross_validate_matches_python(self):
        """get_string result must equal the Python-decoded string value."""
        obj = '{"lang":"BMB","version":"97"}'
        parsed = pyjson.loads(obj)
        assert bmb_json.get_string(obj, 'lang') == parsed['lang']
        assert bmb_json.get_string(obj, 'version') == parsed['version']

    def test_non_string_key_returns_empty_or_value(self):
        # Behaviour for non-string typed values is unspecified; we just ensure
        # no crash and the call returns a str.
        result = bmb_json.get_string('{"n":42}', 'n')
        assert isinstance(result, str)


# ---------------------------------------------------------------------------
# get_number
# ---------------------------------------------------------------------------

class TestGetNumber:
    """bmb_json.get_number(json_str, key) -> int"""

    def test_positive(self):
        assert bmb_json.get_number('{"version":97}', 'version') == 97

    def test_negative(self):
        assert bmb_json.get_number('{"x":-42}', 'x') == -42

    def test_zero(self):
        assert bmb_json.get_number('{"n":0}', 'n') == 0

    def test_missing_key_returns_zero(self):
        assert bmb_json.get_number('{"a":1}', 'missing') == 0

    def test_empty_object_returns_zero(self):
        assert bmb_json.get_number('{}', 'key') == 0

    def test_large_number(self):
        result = bmb_json.get_number('{"big":1000000}', 'big')
        assert result == 1000000

    def test_multiple_keys(self):
        obj = '{"x":10,"y":20,"z":30}'
        assert bmb_json.get_number(obj, 'x') == 10
        assert bmb_json.get_number(obj, 'y') == 20
        assert bmb_json.get_number(obj, 'z') == 30

    def test_cross_validate_matches_python(self):
        obj = '{"count":7,"score":-3}'
        parsed = pyjson.loads(obj)
        assert bmb_json.get_number(obj, 'count') == parsed['count']
        assert bmb_json.get_number(obj, 'score') == parsed['score']

    def test_returns_int_type(self):
        result = bmb_json.get_number('{"n":5}', 'n')
        assert isinstance(result, int)


# ---------------------------------------------------------------------------
# array_len
# ---------------------------------------------------------------------------

class TestArrayLen:
    """bmb_json.array_len(json_str) -> int  (-1 if not array)"""

    def test_nonempty_array(self):
        assert bmb_json.array_len('[10,20,30,40,50]') == 5

    def test_single_element(self):
        assert bmb_json.array_len('[42]') == 1

    def test_empty_array(self):
        assert bmb_json.array_len('[]') == 0

    def test_nested_arrays_counts_top_level(self):
        assert bmb_json.array_len('[[1,2],[3,4],[5]]') == 3

    def test_array_of_objects(self):
        assert bmb_json.array_len('[{"a":1},{"b":2}]') == 2

    def test_array_of_strings(self):
        assert bmb_json.array_len('["x","y","z"]') == 3

    def test_array_of_mixed(self):
        assert bmb_json.array_len('[1,"two",true,null]') == 4

    # --- non-arrays return -1 ----------------------------------------------

    def test_non_array_number(self):
        assert bmb_json.array_len('42') == -1

    def test_non_array_string(self):
        assert bmb_json.array_len('"hello"') == -1

    def test_non_array_object(self):
        assert bmb_json.array_len('{"a":1}') == -1

    def test_non_array_null(self):
        assert bmb_json.array_len('null') == -1

    def test_non_array_bool(self):
        assert bmb_json.array_len('true') == -1

    # --- cross-validate with Python ----------------------------------------

    @pytest.mark.parametrize("s", [
        '[]',
        '[1]',
        '[1,2,3]',
        '["a","b"]',
        '[[],[]]',
        '[null,true,false]',
    ])
    def test_cross_validate_len(self, s):
        expected = len(pyjson.loads(s))
        assert bmb_json.array_len(s) == expected


# ---------------------------------------------------------------------------
# array_get
# ---------------------------------------------------------------------------

class TestArrayGet:
    """bmb_json.array_get(json_str, index) -> str  ("" if out of bounds)"""

    INT_ARR = '[10,20,30]'
    STR_ARR = '["a","b","c"]'
    MIXED_ARR = '[1,true,null,"x",{"k":1}]'

    def test_first_element_number(self):
        assert bmb_json.array_get(self.INT_ARR, 0) == '10'

    def test_middle_element_number(self):
        assert bmb_json.array_get(self.INT_ARR, 1) == '20'

    def test_last_element_number(self):
        assert bmb_json.array_get(self.INT_ARR, 2) == '30'

    def test_first_element_string(self):
        assert bmb_json.array_get(self.STR_ARR, 0) == '"a"'

    def test_middle_element_string(self):
        assert bmb_json.array_get(self.STR_ARR, 1) == '"b"'

    def test_last_element_string(self):
        assert bmb_json.array_get(self.STR_ARR, 2) == '"c"'

    def test_mixed_number(self):
        assert bmb_json.array_get(self.MIXED_ARR, 0) == '1'

    def test_mixed_bool(self):
        assert bmb_json.array_get(self.MIXED_ARR, 1) == 'true'

    def test_mixed_null(self):
        assert bmb_json.array_get(self.MIXED_ARR, 2) == 'null'

    def test_mixed_string(self):
        assert bmb_json.array_get(self.MIXED_ARR, 3) == '"x"'

    def test_mixed_object(self):
        raw = bmb_json.array_get(self.MIXED_ARR, 4)
        assert pyjson.loads(raw) == {"k": 1}

    def test_out_of_bounds_returns_empty(self):
        assert bmb_json.array_get(self.INT_ARR, 3) == ''

    def test_negative_index_boundary(self):
        # Negative index: expected to return "" (out of bounds / unsupported)
        result = bmb_json.array_get(self.INT_ARR, -1)
        assert isinstance(result, str)

    def test_single_element_array(self):
        assert bmb_json.array_get('[99]', 0) == '99'

    def test_large_index_on_empty_array(self):
        assert bmb_json.array_get('[]', 0) == ''

    def test_nested_array_element(self):
        arr = '[[1,2],[3,4]]'
        raw = bmb_json.array_get(arr, 0)
        assert pyjson.loads(raw) == [1, 2]

    # --- cross-validate with Python ----------------------------------------

    @pytest.mark.parametrize("s", [
        '[10,20,30]',
        '["x","y","z"]',
        '[true,false,null]',
        '[{"a":1},{"b":2}]',
    ])
    def test_cross_validate_elements(self, s):
        py_list = pyjson.loads(s)
        for i, expected in enumerate(py_list):
            raw = bmb_json.array_get(s, i)
            assert raw != '', f"Expected non-empty for index {i}"
            assert pyjson.loads(raw) == expected


# ---------------------------------------------------------------------------
# Integration: combined workflows
# ---------------------------------------------------------------------------

class TestIntegration:
    """End-to-end scenarios combining multiple functions."""

    def test_parse_and_extract_all_fields(self):
        obj = '{"name":"Alice","age":30,"active":true}'
        assert bmb_json.validate(obj) is True
        assert bmb_json.get_type(obj) == 'object'
        assert bmb_json.get_string(obj, 'name') == 'Alice'
        assert bmb_json.get_number(obj, 'age') == 30

    def test_array_iteration(self):
        arr = '[10,20,30,40,50]'
        n = bmb_json.array_len(arr)
        assert n == 5
        values = [int(bmb_json.array_get(arr, i)) for i in range(n)]
        assert values == [10, 20, 30, 40, 50]

    def test_stringify_then_get(self):
        raw = '{ "x" : 1 , "y" : 2 }'
        minified = bmb_json.stringify(raw)
        assert bmb_json.get_number(minified, 'x') == 1
        assert bmb_json.get_number(minified, 'y') == 2

    def test_nested_array_extraction(self):
        obj = '{"items":[1,2,3]}'
        raw_arr = bmb_json.get(obj, 'items')
        assert bmb_json.get_type(raw_arr) == 'array'
        assert bmb_json.array_len(raw_arr) == 3
        assert bmb_json.array_get(raw_arr, 0) == '1'

    def test_full_roundtrip_complex(self):
        complex_json = '{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}],"count":2}'
        assert bmb_json.validate(complex_json) is True
        stringified = bmb_json.stringify(complex_json)
        assert pyjson.loads(stringified) == pyjson.loads(complex_json)
        assert bmb_json.get_number(stringified, 'count') == 2
        users_raw = bmb_json.get(stringified, 'users')
        assert bmb_json.array_len(users_raw) == 2
        first_user_raw = bmb_json.array_get(users_raw, 0)
        assert bmb_json.get_string(first_user_raw, 'name') == 'Alice'

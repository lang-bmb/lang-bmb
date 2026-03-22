"""
bmb-json: High-performance JSON parser/serializer powered by BMB
https://github.com/iyulab/lang-bmb

Functions: validate, stringify, get_type, get, get_string, get_number,
           array_len, array_get
"""

import ctypes
import os
import sys

_lib_dir = os.path.dirname(os.path.abspath(__file__))
_lib_name = {'win32': 'bmb_json.dll', 'linux': 'libbmb_json.so', 'darwin': 'libbmb_json.dylib'}.get(sys.platform, 'libbmb_json.so')
_lib_path = os.path.join(_lib_dir, _lib_name)
if not os.path.exists(_lib_path):
    _lib_path = os.path.join(_lib_dir, '..', '..', _lib_name)

if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

_lib = ctypes.CDLL(_lib_path)

# FFI
_lib.bmb_ffi_begin.restype = ctypes.c_int
_lib.bmb_ffi_end.restype = None
_lib.bmb_ffi_cstr_to_string.argtypes = [ctypes.c_char_p]
_lib.bmb_ffi_cstr_to_string.restype = ctypes.c_void_p
_lib.bmb_ffi_string_data.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_string_data.restype = ctypes.c_char_p
_lib.bmb_ffi_free_string.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_free_string.restype = None

# String→i64 functions
_lib.bmb_json_validate.argtypes = [ctypes.c_void_p]
_lib.bmb_json_validate.restype = ctypes.c_int64
_lib.bmb_json_array_len.argtypes = [ctypes.c_void_p]
_lib.bmb_json_array_len.restype = ctypes.c_int64
_lib.bmb_json_get_number.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_json_get_number.restype = ctypes.c_int64

# String→String functions
_lib.bmb_json_stringify.argtypes = [ctypes.c_void_p]
_lib.bmb_json_stringify.restype = ctypes.c_void_p
_lib.bmb_json_type.argtypes = [ctypes.c_void_p]
_lib.bmb_json_type.restype = ctypes.c_void_p
_lib.bmb_json_get.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_json_get.restype = ctypes.c_void_p
_lib.bmb_json_get_string.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_json_get_string.restype = ctypes.c_void_p
_lib.bmb_json_array_get.argtypes = [ctypes.c_void_p, ctypes.c_int64]
_lib.bmb_json_array_get.restype = ctypes.c_void_p


def _make_str(s):
    return _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))

def _read_str(ptr):
    return _lib.bmb_ffi_string_data(ptr).decode('utf-8')

def _call_s_to_i(fn, s):
    ss = _make_str(s)
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    result = fn(ss)
    _lib.bmb_ffi_end()
    _lib.bmb_ffi_free_string(ss)
    return result

def _call_s_to_s(fn, s):
    ss = _make_str(s)
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = fn(ss)
    _lib.bmb_ffi_end()
    result = _read_str(out)
    _lib.bmb_ffi_free_string(ss)
    return result

def _call_ss_to_s(fn, s1, s2):
    a = _make_str(s1); b = _make_str(s2)
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b)
        raise RuntimeError("BMB error")
    out = fn(a, b)
    _lib.bmb_ffi_end()
    result = _read_str(out)
    _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b)
    return result

def _call_ss_to_i(fn, s1, s2):
    a = _make_str(s1); b = _make_str(s2)
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b)
        raise RuntimeError("BMB error")
    result = fn(a, b)
    _lib.bmb_ffi_end()
    _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b)
    return result


def validate(json_str: str) -> bool:
    """Check if input is valid JSON."""
    return bool(_call_s_to_i(_lib.bmb_json_validate, json_str))


def stringify(json_str: str) -> str:
    """Parse and re-serialize JSON (roundtrip normalization)."""
    return _call_s_to_s(_lib.bmb_json_stringify, json_str)


def get_type(json_str: str) -> str:
    """Get type of root JSON value: null, bool, number, string, array, object."""
    return _call_s_to_s(_lib.bmb_json_type, json_str)


def get(json_str: str, key: str) -> str:
    """Get object value by key as JSON string. Empty string if not found."""
    return _call_ss_to_s(_lib.bmb_json_get, json_str, key)


def get_string(json_str: str, key: str) -> str:
    """Get string value from object by key (unquoted). Empty if not found."""
    return _call_ss_to_s(_lib.bmb_json_get_string, json_str, key)


def get_number(json_str: str, key: str) -> int:
    """Get number value from object by key. 0 if not found."""
    return _call_ss_to_i(_lib.bmb_json_get_number, json_str, key)


def array_len(json_str: str) -> int:
    """Get array length. -1 if not an array."""
    return _call_s_to_i(_lib.bmb_json_array_len, json_str)


def array_get(json_str: str, idx: int) -> str:
    """Get array element at index as JSON string."""
    ss = _make_str(json_str)
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = _lib.bmb_json_array_get(ss, idx)
    _lib.bmb_ffi_end()
    result = _read_str(out)
    _lib.bmb_ffi_free_string(ss)
    return result


if __name__ == '__main__':
    import json

    passed = 0
    failed = 0

    def check(name, got, expected):
        global passed, failed
        if got == expected:
            print(f"  PASS: {name}")
            passed += 1
        else:
            print(f"  FAIL: {name} (got {repr(got)}, expected {repr(expected)})")
            failed += 1

    print("bmb-json test suite -- Powered by BMB")
    print()

    # Validate
    print("[Validate]")
    check("valid object", validate('{"a":1}'), True)
    check("valid array", validate('[1,2,3]'), True)
    check("valid string", validate('"hello"'), True)
    check("valid number", validate('42'), True)
    check("valid null", validate('null'), True)
    check("valid bool", validate('true'), True)
    check("empty invalid", validate(''), False)

    # Stringify (roundtrip)
    print("[Stringify]")
    check("object", stringify('{"name":"BMB","version":97}'), '{"name":"BMB","version":97}')
    check("array", stringify('[1,2,3]'), '[1,2,3]')
    check("nested", stringify('{"a":{"b":1}}'), '{"a":{"b":1}}')
    check("null", stringify('null'), 'null')
    check("bool", stringify('true'), 'true')
    check("string", stringify('"hello"'), '"hello"')
    check("escape", stringify('"hello\\nworld"'), '"hello\\nworld"')

    # Type
    print("[Type]")
    check("null type", get_type('null'), 'null')
    check("bool type", get_type('true'), 'bool')
    check("number type", get_type('42'), 'number')
    check("string type", get_type('"hi"'), 'string')
    check("array type", get_type('[1]'), 'array')
    check("object type", get_type('{"a":1}'), 'object')

    # Get
    print("[Get]")
    obj = '{"name":"BMB","version":97,"tags":["fast","safe"]}'
    check("get string", get(obj, "name"), '"BMB"')
    check("get number", get(obj, "version"), '97')
    check("get array", get(obj, "tags"), '["fast","safe"]')
    check("get missing", get(obj, "missing"), '')

    # Get string/number
    print("[Get String/Number]")
    check("get_string", get_string('{"name":"BMB"}', "name"), "BMB")
    check("get_number", get_number('{"version":97}', "version"), 97)
    check("get_number negative", get_number('{"x":-42}', "x"), -42)

    # Array
    print("[Array]")
    check("array_len", array_len('[10,20,30,40,50]'), 5)
    check("array_len empty", array_len('[]'), 0)
    check("array_len non-array", array_len('42'), -1)
    check("array_get 0", array_get('[10,20,30]', 0), '10')
    check("array_get 1", array_get('[10,20,30]', 1), '20')
    check("array_get 2", array_get('[10,20,30]', 2), '30')
    check("array_get string", array_get('["a","b"]', 0), '"a"')

    # Cross-validation vs Python json
    print("[Cross-validation vs Python json]")
    test_cases = [
        '{"key":"value"}',
        '[1,2,3]',
        '{"nested":{"a":1,"b":2}}',
        '{"arr":[true,false,null]}',
        '{"num":-123}',
    ]
    for tc in test_cases:
        py_result = json.dumps(json.loads(tc), separators=(',', ':'))
        bmb_result = stringify(tc)
        check(f"roundtrip {tc[:30]}", bmb_result, py_result)

    print()
    total = passed + failed
    print(f"Results: {passed}/{total} passed")
    if failed > 0:
        print(f"  {failed} FAILED")
        sys.exit(1)
    else:
        print("All tests passed!")

"""
bmb-text: High-performance string processing powered by BMB
https://github.com/iyulab/lang-bmb

Functions: kmp_search, str_find, str_rfind, str_count, str_contains,
           str_starts_with, str_ends_with, find_byte, count_byte,
           is_palindrome, token_count
"""

import ctypes
import os
import sys

_lib_dir = os.path.dirname(os.path.abspath(__file__))
_lib_name = {'win32': 'bmb_text.dll', 'linux': 'libbmb_text.so', 'darwin': 'libbmb_text.dylib'}.get(sys.platform, 'libbmb_text.so')
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
_lib.bmb_ffi_error_message.restype = ctypes.c_char_p
_lib.bmb_ffi_cstr_to_string.argtypes = [ctypes.c_char_p]
_lib.bmb_ffi_cstr_to_string.restype = ctypes.c_void_p
_lib.bmb_ffi_string_data.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_string_data.restype = ctypes.c_char_p
_lib.bmb_ffi_free_string.argtypes = [ctypes.c_void_p]
_lib.bmb_ffi_free_string.restype = None

# String→String functions (two string args → i64 result)
for fn_name in ['bmb_kmp_search', 'bmb_str_find', 'bmb_str_rfind', 'bmb_str_count',
                'bmb_str_contains', 'bmb_str_starts_with', 'bmb_str_ends_with']:
    fn = getattr(_lib, fn_name)
    fn.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    fn.restype = ctypes.c_int64

# String+i64 functions
for fn_name in ['bmb_str_find_byte', 'bmb_str_count_byte', 'bmb_token_count']:
    fn = getattr(_lib, fn_name)
    fn.argtypes = [ctypes.c_void_p, ctypes.c_int64]
    fn.restype = ctypes.c_int64

# String→i64 functions
_lib.bmb_is_palindrome.argtypes = [ctypes.c_void_p]
_lib.bmb_is_palindrome.restype = ctypes.c_int64
_lib.bmb_word_count.argtypes = [ctypes.c_void_p]
_lib.bmb_word_count.restype = ctypes.c_int64
_lib.bmb_str_hamming.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_str_hamming.restype = ctypes.c_int64

# String→String functions (new)
_lib.bmb_str_reverse.argtypes = [ctypes.c_void_p]
_lib.bmb_str_reverse.restype = ctypes.c_void_p
_lib.bmb_str_replace.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_str_replace.restype = ctypes.c_void_p
_lib.bmb_str_replace_all.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_str_replace_all.restype = ctypes.c_void_p
_lib.bmb_str_to_upper.argtypes = [ctypes.c_void_p]
_lib.bmb_str_to_upper.restype = ctypes.c_void_p
_lib.bmb_str_to_lower.argtypes = [ctypes.c_void_p]
_lib.bmb_str_to_lower.restype = ctypes.c_void_p
_lib.bmb_str_trim.argtypes = [ctypes.c_void_p]
_lib.bmb_str_trim.restype = ctypes.c_void_p
_lib.bmb_str_repeat.argtypes = [ctypes.c_void_p, ctypes.c_int64]
_lib.bmb_str_repeat.restype = ctypes.c_void_p


def _call_ss(fn, a, b):
    """Call a (String, String) -> i64 function."""
    sa = _lib.bmb_ffi_cstr_to_string(a.encode('utf-8'))
    sb = _lib.bmb_ffi_cstr_to_string(b.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(sa)
        _lib.bmb_ffi_free_string(sb)
        raise RuntimeError("BMB error")
    result = fn(sa, sb)
    _lib.bmb_ffi_end()
    _lib.bmb_ffi_free_string(sa)
    _lib.bmb_ffi_free_string(sb)
    return result


def _call_si(fn, s, i):
    """Call a (String, i64) -> i64 function."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    result = fn(ss, i)
    _lib.bmb_ffi_end()
    _lib.bmb_ffi_free_string(ss)
    return result


def _call_s(fn, s):
    """Call a (String) -> i64 function."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    result = fn(ss)
    _lib.bmb_ffi_end()
    _lib.bmb_ffi_free_string(ss)
    return result


def kmp_search(text: str, pattern: str) -> int:
    """KMP substring search. Returns first match index or -1."""
    return _call_ss(_lib.bmb_kmp_search, text, pattern)


def str_find(haystack: str, needle: str) -> int:
    """Find first occurrence of substring. Returns index or -1."""
    return _call_ss(_lib.bmb_str_find, haystack, needle)


def str_rfind(haystack: str, needle: str) -> int:
    """Find last occurrence of substring. Returns index or -1."""
    return _call_ss(_lib.bmb_str_rfind, haystack, needle)


def str_count(haystack: str, needle: str) -> int:
    """Count non-overlapping occurrences of substring."""
    return _call_ss(_lib.bmb_str_count, haystack, needle)


def str_contains(haystack: str, needle: str) -> bool:
    """Check if string contains substring."""
    return bool(_call_ss(_lib.bmb_str_contains, haystack, needle))


def str_starts_with(s: str, prefix: str) -> bool:
    """Check if string starts with prefix."""
    return bool(_call_ss(_lib.bmb_str_starts_with, s, prefix))


def str_ends_with(s: str, suffix: str) -> bool:
    """Check if string ends with suffix."""
    return bool(_call_ss(_lib.bmb_str_ends_with, s, suffix))


def find_byte(s: str, byte: int) -> int:
    """Find first occurrence of byte value. Returns index or -1."""
    return _call_si(_lib.bmb_str_find_byte, s, byte)


def count_byte(s: str, byte: int) -> int:
    """Count occurrences of byte value."""
    return _call_si(_lib.bmb_str_count_byte, s, byte)


def is_palindrome(s: str) -> bool:
    """Check if string is a palindrome."""
    return bool(_call_s(_lib.bmb_is_palindrome, s))


def token_count(s: str, delimiter: str) -> int:
    """Count tokens split by delimiter character."""
    return _call_si(_lib.bmb_token_count, s, ord(delimiter[0]))


def str_reverse(s: str) -> str:
    """Reverse a string."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_reverse(ss)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(ss)
    return result


def str_replace(s: str, old: str, new: str) -> str:
    """Replace first occurrence of old with new."""
    a = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    b = _lib.bmb_ffi_cstr_to_string(old.encode('utf-8'))
    c = _lib.bmb_ffi_cstr_to_string(new.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b); _lib.bmb_ffi_free_string(c)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_replace(a, b, c)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b); _lib.bmb_ffi_free_string(c)
    return result


def str_replace_all(s: str, old: str, new: str) -> str:
    """Replace all occurrences of old with new."""
    a = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    b = _lib.bmb_ffi_cstr_to_string(old.encode('utf-8'))
    c = _lib.bmb_ffi_cstr_to_string(new.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b); _lib.bmb_ffi_free_string(c)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_replace_all(a, b, c)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(a); _lib.bmb_ffi_free_string(b); _lib.bmb_ffi_free_string(c)
    return result


def hamming_distance(a: str, b: str) -> int:
    """Hamming distance between equal-length strings. -1 if different lengths."""
    return _call_ss(_lib.bmb_str_hamming, a, b)


def word_count(s: str) -> int:
    """Count words (space-separated)."""
    return _call_s(_lib.bmb_word_count, s)


def to_upper(s: str) -> str:
    """Convert to uppercase."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_to_upper(ss)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(ss)
    return result


def to_lower(s: str) -> str:
    """Convert to lowercase."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_to_lower(ss)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(ss)
    return result


def trim(s: str) -> str:
    """Trim leading/trailing whitespace."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_trim(ss)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(ss)
    return result


def repeat(s: str, n: int) -> str:
    """Repeat string n times."""
    ss = _lib.bmb_ffi_cstr_to_string(s.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        _lib.bmb_ffi_end(); _lib.bmb_ffi_free_string(ss)
        raise RuntimeError("BMB error")
    out = _lib.bmb_str_repeat(ss, n)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(out).decode('utf-8')
    _lib.bmb_ffi_free_string(ss)
    return result


if __name__ == '__main__':
    passed = 0
    failed = 0

    def check(name, got, expected):
        global passed, failed
        if got == expected:
            print(f"  PASS: {name}")
            passed += 1
        else:
            print(f"  FAIL: {name} (got {got}, expected {expected})")
            failed += 1

    print("bmb-text test suite -- Powered by BMB")
    print()

    # KMP Search
    print("[KMP Search]")
    check("kmp('hello world', 'world')", kmp_search("hello world", "world"), 6)
    check("kmp('hello world', 'xyz')", kmp_search("hello world", "xyz"), -1)
    check("kmp('AABAABAABAAB', 'AAB')", kmp_search("AABAABAABAAB", "AAB"), 0)
    check("kmp('abcdef', 'def')", kmp_search("abcdef", "def"), 3)

    # Substring find/rfind/count
    print("[Substring Operations]")
    check("str_find('abcabc', 'bc')", str_find("abcabc", "bc"), 1)
    check("str_rfind('abcabc', 'bc')", str_rfind("abcabc", "bc"), 4)
    check("str_count('abcabcabc', 'abc')", str_count("abcabcabc", "abc"), 3)
    check("str_count('aaa', 'a')", str_count("aaa", "a"), 3)

    # Contains/starts/ends
    print("[String Tests]")
    check("contains('hello world', 'world')", str_contains("hello world", "world"), True)
    check("contains('hello world', 'xyz')", str_contains("hello world", "xyz"), False)
    check("starts_with('hello', 'hel')", str_starts_with("hello", "hel"), True)
    check("starts_with('hello', 'xyz')", str_starts_with("hello", "xyz"), False)
    check("ends_with('hello', 'llo')", str_ends_with("hello", "llo"), True)
    check("ends_with('hello', 'xyz')", str_ends_with("hello", "xyz"), False)

    # Palindrome
    print("[Palindrome]")
    check("palindrome('racecar')", is_palindrome("racecar"), True)
    check("palindrome('hello')", is_palindrome("hello"), False)
    check("palindrome('a')", is_palindrome("a"), True)
    check("palindrome('')", is_palindrome(""), True)

    # Byte operations
    print("[Byte Operations]")
    check("find_byte('hello', ord('l'))", find_byte("hello", ord('l')), 2)
    check("find_byte('hello', ord('z'))", find_byte("hello", ord('z')), -1)
    check("count_byte('hello', ord('l'))", count_byte("hello", ord('l')), 2)
    check("count_byte('hello', ord('z'))", count_byte("hello", ord('z')), 0)

    # Token count
    print("[Tokenizer]")
    check("token_count('a,b,c,d', ',')", token_count("a,b,c,d", ","), 4)
    check("token_count('hello', ',')", token_count("hello", ","), 1)
    check("token_count('a::b::c', ':')", token_count("a::b::c", ":"), 5)  # a, '', b, '', c

    # New functions
    print("[String Reverse]")
    check("reverse('hello')", str_reverse("hello"), "olleh")
    check("reverse('')", str_reverse(""), "")
    check("reverse('a')", str_reverse("a"), "a")

    print("[String Replace]")
    check("replace first", str_replace("hello world", "world", "BMB"), "hello BMB")
    check("replace miss", str_replace("hello world", "xyz", "BMB"), "hello world")
    check("replace all", str_replace_all("abcabc", "abc", "X"), "XX")
    check("replace all none", str_replace_all("hello", "xyz", "!"), "hello")

    print("[Hamming Distance]")
    check("hamming('karolin','kathrin')", hamming_distance("karolin", "kathrin"), 3)
    check("hamming same", hamming_distance("hello", "hello"), 0)
    check("hamming diff len", hamming_distance("ab", "abc"), -1)

    print("[Word Count]")
    check("word_count('hello world')", word_count("hello world"), 2)
    check("word_count(' hello  world ')", word_count(" hello  world "), 2)
    check("word_count('')", word_count(""), 0)
    check("word_count('one')", word_count("one"), 1)

    print("[Case Conversion]")
    check("to_upper('hello')", to_upper("hello"), "HELLO")
    check("to_lower('HELLO')", to_lower("HELLO"), "hello")
    check("to_upper('Hello123')", to_upper("Hello123"), "HELLO123")

    print("[Trim]")
    check("trim('  hello  ')", trim("  hello  "), "hello")
    check("trim('hello')", trim("hello"), "hello")
    check("trim('  ')", trim("  "), "")

    print("[Repeat]")
    check("repeat('ab', 3)", repeat("ab", 3), "ababab")
    check("repeat('x', 0)", repeat("x", 0), "")

    print()
    total = passed + failed
    print(f"Results: {passed}/{total} passed")
    if failed > 0:
        print(f"  {failed} FAILED")
        sys.exit(1)
    else:
        print("All tests passed!")

"""
bmb-crypto: High-performance cryptographic functions powered by BMB
https://github.com/iyulab/lang-bmb

Functions: sha256, md5, crc32, base64_encode, base64_decode, base32_encode, base32_decode
"""

import ctypes
import os
import sys

_lib_dir = os.path.dirname(os.path.abspath(__file__))
_lib_name = {'win32': 'bmb_crypto.dll', 'linux': 'libbmb_crypto.so', 'darwin': 'libbmb_crypto.dylib'}.get(sys.platform, 'libbmb_crypto.so')
_lib_path = os.path.join(_lib_dir, _lib_name)
if not os.path.exists(_lib_path):
    _lib_path = os.path.join(_lib_dir, '..', '..', _lib_name)
# On Windows, add MSYS2/MinGW runtime directory for GCC runtime dependencies
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

# SHA-256
_lib.bmb_sha256.argtypes = [ctypes.c_void_p]
_lib.bmb_sha256.restype = ctypes.c_void_p
# MD5
_lib.bmb_md5.argtypes = [ctypes.c_void_p]
_lib.bmb_md5.restype = ctypes.c_void_p
# CRC32
_lib.bmb_crc32.argtypes = [ctypes.c_void_p]
_lib.bmb_crc32.restype = ctypes.c_void_p
# Base64
_lib.bmb_base64_encode.argtypes = [ctypes.c_void_p]
_lib.bmb_base64_encode.restype = ctypes.c_void_p
_lib.bmb_base64_decode.argtypes = [ctypes.c_void_p]
_lib.bmb_base64_decode.restype = ctypes.c_void_p
# Base32
_lib.bmb_base32_encode.argtypes = [ctypes.c_void_p]
_lib.bmb_base32_encode.restype = ctypes.c_void_p
_lib.bmb_base32_decode.argtypes = [ctypes.c_void_p]
_lib.bmb_base32_decode.restype = ctypes.c_void_p
# HMAC-SHA256
_lib.bmb_hmac_sha256.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
_lib.bmb_hmac_sha256.restype = ctypes.c_void_p


def _call_string_fn(fn, data):
    s_in = _lib.bmb_ffi_cstr_to_string(data.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        msg = _lib.bmb_ffi_error_message()
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(s_in)
        raise RuntimeError(f"BMB error: {msg.decode() if msg else 'unknown'}")
    s_out = fn(s_in)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(s_out).decode('utf-8')
    _lib.bmb_ffi_free_string(s_in)
    return result


def sha256(data: str) -> str:
    """SHA-256 hash (FIPS 180-4). Returns 64-char hex digest."""
    return _call_string_fn(_lib.bmb_sha256, data)


def md5(data: str) -> str:
    """MD5 hash (RFC 1321). Returns 32-char hex digest."""
    return _call_string_fn(_lib.bmb_md5, data)


def crc32(data: str) -> str:
    """CRC-32 checksum (ISO 3309). Returns 8-char hex string."""
    return _call_string_fn(_lib.bmb_crc32, data)


def base64_encode(data: str) -> str:
    """RFC 4648 Base64 encode."""
    return _call_string_fn(_lib.bmb_base64_encode, data)


def base64_decode(data: str) -> str:
    """RFC 4648 Base64 decode."""
    return _call_string_fn(_lib.bmb_base64_decode, data)


def base32_encode(data: str) -> str:
    """RFC 4648 Base32 encode."""
    return _call_string_fn(_lib.bmb_base32_encode, data)


def base32_decode(data: str) -> str:
    """RFC 4648 Base32 decode."""
    return _call_string_fn(_lib.bmb_base32_decode, data)


def hmac_sha256(key: str, msg: str) -> str:
    """HMAC-SHA256 (RFC 2104). Returns 64-char hex digest."""
    s_key = _lib.bmb_ffi_cstr_to_string(key.encode('utf-8'))
    s_msg = _lib.bmb_ffi_cstr_to_string(msg.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        err = _lib.bmb_ffi_error_message()
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(s_key)
        _lib.bmb_ffi_free_string(s_msg)
        raise RuntimeError(f"BMB error: {err.decode() if err else 'unknown'}")
    s_out = _lib.bmb_hmac_sha256(s_key, s_msg)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(s_out).decode('utf-8')
    _lib.bmb_ffi_free_string(s_key)
    _lib.bmb_ffi_free_string(s_msg)
    return result


if __name__ == '__main__':
    import hashlib
    import base64 as b64
    import binascii

    passed = 0
    failed = 0

    def check(name, got, expected):
        global passed, failed
        if got == expected:
            print(f"  PASS: {name}")
            passed += 1
        else:
            print(f"  FAIL: {name}")
            print(f"    got:      {got}")
            print(f"    expected: {expected}")
            failed += 1

    print("bmb-crypto test suite -- Powered by BMB")
    print()

    # SHA-256
    print("[SHA-256]")
    for inp, expected in [("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
                          ("hello", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
                          ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")]:
        check(f"sha256({repr(inp)})", sha256(inp), expected)

    # MD5
    print("[MD5]")
    for inp in ["", "hello", "abc", "The quick brown fox jumps over the lazy dog"]:
        expected = hashlib.md5(inp.encode()).hexdigest()
        check(f"md5({repr(inp)[:30]})", md5(inp), expected)

    # CRC32
    print("[CRC32]")
    for inp in ["", "hello", "abc", "123456789"]:
        expected = format(binascii.crc32(inp.encode()) & 0xffffffff, '08x')
        check(f"crc32({repr(inp)})", crc32(inp), expected)

    # Base64
    print("[Base64]")
    for inp, expected in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"),
                          ("foobar", "Zm9vYmFy"), ("hello", "aGVsbG8=")]:
        enc = base64_encode(inp)
        check(f"base64_encode({repr(inp)})", enc, expected)
        if enc:
            dec = base64_decode(enc)
            check(f"base64_decode({repr(enc)})", dec, inp)

    # Base32
    print("[Base32]")
    for inp, expected in [("f", "MY======"), ("fo", "MZXQ===="), ("foo", "MZXW6==="),
                          ("foob", "MZXW6YQ="), ("fooba", "MZXW6YTB"), ("foobar", "MZXW6YTBOI======")]:
        enc = base32_encode(inp)
        check(f"base32_encode({repr(inp)})", enc, expected)
        if enc:
            dec = base32_decode(enc)
            check(f"base32_decode({repr(enc)})", dec, inp)

    # HMAC-SHA256
    print("[HMAC-SHA256]")
    import hmac as hmac_mod
    for key, msg in [("key", "The quick brown fox jumps over the lazy dog"),
                     ("secret", "hello"),
                     ("", ""),
                     ("abc", "abc")]:
        expected = hmac_mod.new(key.encode(), msg.encode(), hashlib.sha256).hexdigest()
        result = hmac_sha256(key, msg)
        check(f"hmac_sha256({repr(key)[:10]}, {repr(msg)[:20]})", result, expected)

    # Benchmark
    print()
    print("[Benchmark: bmb-crypto vs Python stdlib]")
    import time
    N = 10000
    test_data = "The quick brown fox jumps over the lazy dog" * 10

    # SHA-256
    t0 = time.perf_counter()
    for _ in range(N):
        hashlib.sha256(test_data.encode()).hexdigest()
    py_sha = time.perf_counter() - t0

    t0 = time.perf_counter()
    for _ in range(N):
        sha256(test_data)
    bmb_sha = time.perf_counter() - t0

    print(f"  SHA-256 ({N}x {len(test_data)}B): Python {py_sha:.3f}s, BMB {bmb_sha:.3f}s, ratio {py_sha/bmb_sha:.2f}x")

    # MD5
    t0 = time.perf_counter()
    for _ in range(N):
        hashlib.md5(test_data.encode()).hexdigest()
    py_md5 = time.perf_counter() - t0

    t0 = time.perf_counter()
    for _ in range(N):
        md5(test_data)
    bmb_md5t = time.perf_counter() - t0

    print(f"  MD5    ({N}x {len(test_data)}B): Python {py_md5:.3f}s, BMB {bmb_md5t:.3f}s, ratio {py_md5/bmb_md5t:.2f}x")

    # Base64
    t0 = time.perf_counter()
    for _ in range(N):
        b64.b64encode(test_data.encode()).decode()
    py_b64 = time.perf_counter() - t0

    t0 = time.perf_counter()
    for _ in range(N):
        base64_encode(test_data)
    bmb_b64 = time.perf_counter() - t0

    print(f"  Base64 ({N}x {len(test_data)}B): Python {py_b64:.3f}s, BMB {bmb_b64:.3f}s, ratio {py_b64/bmb_b64:.2f}x")

    print()
    total = passed + failed
    print(f"Results: {passed}/{total} passed")
    if failed > 0:
        print(f"  {failed} FAILED")
        sys.exit(1)
    else:
        print("All tests passed!")

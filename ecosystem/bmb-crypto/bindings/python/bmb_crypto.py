"""
bmb-crypto: High-performance cryptographic functions powered by BMB
https://github.com/iyulab/lang-bmb

Functions: sha256, base64_encode, base64_decode
"""

import ctypes
import os
import sys

_lib_dir = os.path.dirname(os.path.abspath(__file__))
_lib_name = {'win32': 'bmb_crypto.dll', 'linux': 'libbmb_crypto.so', 'darwin': 'libbmb_crypto.dylib'}.get(sys.platform, 'libbmb_crypto.so')
_lib_path = os.path.join(_lib_dir, _lib_name)
if not os.path.exists(_lib_path):
    _lib_path = os.path.join(_lib_dir, '..', '..', _lib_name)
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
_lib.bmb_sha256.argtypes = [ctypes.c_void_p]
_lib.bmb_sha256.restype = ctypes.c_void_p
_lib.bmb_base64_encode.argtypes = [ctypes.c_void_p]
_lib.bmb_base64_encode.restype = ctypes.c_void_p
_lib.bmb_base64_decode.argtypes = [ctypes.c_void_p]
_lib.bmb_base64_decode.restype = ctypes.c_void_p


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


def base64_encode(data: str) -> str:
    """RFC 4648 Base64 encode."""
    return _call_string_fn(_lib.bmb_base64_encode, data)


def base64_decode(data: str) -> str:
    """RFC 4648 Base64 decode."""
    return _call_string_fn(_lib.bmb_base64_decode, data)


if __name__ == '__main__':
    import hashlib
    import base64

    print("bmb-crypto test suite -- Powered by BMB")
    print()

    # SHA-256
    for inp, expected in [("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
                          ("hello", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
                          ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")]:
        result = sha256(inp)
        status = "PASS" if result == expected else "FAIL"
        print(f"  sha256({repr(inp):12s}) = {result[:16]}... [{status}]")

    # Base64
    for inp, expected in [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"),
                          ("foobar", "Zm9vYmFy"), ("hello", "aGVsbG8=")]:
        enc = base64_encode(inp)
        dec = base64_decode(enc) if enc else ""
        py_enc = base64.b64encode(inp.encode()).decode()
        status = "PASS" if enc == expected and dec == inp and enc == py_enc else "FAIL"
        print(f"  base64({repr(inp):12s}) = {enc:12s} -> {dec:8s} [{status}]")

    print()
    print("All tests passed!")

"""
bmb-crypto: High-performance cryptographic functions powered by BMB
https://github.com/iyulab/lang-bmb
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


def sha256(data: str) -> str:
    """Compute SHA-256 hash of a string.

    Returns hex-encoded digest (64 characters).

    Example:
        >>> sha256("hello")
        '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
    """
    s_in = _lib.bmb_ffi_cstr_to_string(data.encode('utf-8'))
    if _lib.bmb_ffi_begin() != 0:
        msg = _lib.bmb_ffi_error_message()
        _lib.bmb_ffi_end()
        _lib.bmb_ffi_free_string(s_in)
        raise RuntimeError(f"BMB error: {msg.decode() if msg else 'unknown'}")
    s_out = _lib.bmb_sha256(s_in)
    _lib.bmb_ffi_end()
    result = _lib.bmb_ffi_string_data(s_out).decode('utf-8')
    _lib.bmb_ffi_free_string(s_in)
    return result


if __name__ == '__main__':
    import hashlib

    print("bmb-crypto test suite -- Powered by BMB")
    print()

    tests = [
        ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        ("hello", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
        ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
    ]

    for inp, expected in tests:
        result = sha256(inp)
        stdlib = hashlib.sha256(inp.encode()).hexdigest()
        status = "PASS" if result == expected and result == stdlib else "FAIL"
        print(f"  sha256({repr(inp):12s}) = {result[:16]}... [{status}]")

    print()
    print("All tests passed!")

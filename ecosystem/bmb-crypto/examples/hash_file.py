"""File-content hashing demo using bmb-crypto — SHA-256, MD5, CRC32."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_crypto

# Simulate reading a file — in real usage: content = open(path).read()
content = """\
Hello from BMB!
This file contains some sample text.
Performance > Everything.
"""

print("=== File Content ===")
print(content)

print("=== Hash Results ===")
print(f"SHA-256 : {bmb_crypto.sha256(content)}")
print(f"MD5     : {bmb_crypto.md5(content)}")
print(f"CRC32   : {bmb_crypto.crc32(content)}")
print(f"Adler32 : {bmb_crypto.adler32(content)}")

# Demonstrate that even a single-character change produces a different digest
tampered = content.replace("BMB!", "BMB?")
print("\n=== Tampered Content Hashes (one char changed) ===")
print(f"SHA-256 : {bmb_crypto.sha256(tampered)}")
print(f"MD5     : {bmb_crypto.md5(tampered)}")

# HMAC for message authentication
key = "secret-key"
print("\n=== HMAC-SHA256 ===")
print(f"HMAC    : {bmb_crypto.hmac_sha256(key, content)}")

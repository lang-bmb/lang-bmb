#!/usr/bin/env python3
"""Reference for tests/lib_run/crc32.bmb (backlog T4-04): the same inputs
checksummed by Python's `zlib.crc32`. `python3 crc32_ref.py > crc32.out`."""
import zlib

M64 = (1 << 64) - 1


def stream(n):
    x, out = 2024, bytearray()
    for _ in range(n):
        x = (x * 6364136223846793005 + 1442695040888963407) & M64
        out.append(x >> 56)
    return bytes(out)


def main():
    print("check", zlib.crc32(b"123456789"))
    print("empty", zlib.crc32(b""))
    print("fox", zlib.crc32(b"The quick brown fox jumps over the lazy dog"))
    s = stream(4096)
    for n in range(65):
        print("len", zlib.crc32(s[:n]))
    print("sum4096", sum(zlib.crc32(s[:n]) for n in range(4097)) & M64)
    print("table4096", sum(zlib.crc32(s[:n], 0) for n in range(4097)) & M64)
    short = 0
    for a in range(256):
        short += zlib.crc32(bytes([a]))
        for b in range(256):
            short += zlib.crc32(bytes([a, b]))
    print("short", short & M64)
    whole = zlib.crc32(s)
    print("whole", whole)
    print("splits", sum(zlib.crc32(s[k:], zlib.crc32(s[:k])) == whole for k in range(4097)))


main()

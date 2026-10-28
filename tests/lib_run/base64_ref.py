#!/usr/bin/env python3
"""Reference for tests/lib_run/base64.bmb (backlog T4-03): the same inputs
through Python's `base64`. A string is a canonical encoding (RFC 4648 §3.5)
when `b64decode(t, validate=True)` accepts it and `b64encode` gives `t` back.
`python3 base64_ref.py > base64.out`."""
import base64
import binascii
import zlib

M64 = (1 << 64) - 1


def decode(t):
    try:
        v = base64.b64decode(t, validate=True)
    except (binascii.Error, ValueError):
        return None
    return v if base64.b64encode(v) == t else None


def stream(n):
    x, out = 2024, bytearray()
    for _ in range(n):
        x = (x * 6364136223846793005 + 1442695040888963407) & M64
        out.append(x >> 56)
    return bytes(out)


def out(label, text):
    print(label, text.decode("latin-1"))


def exhaustive(label, pre):
    chars = b"ABQZagwz09+/=- \x80"
    count = total = 0
    for a in chars:
        for b in chars:
            for c in chars:
                for d in chars:
                    v = decode(pre + bytes([a, b, c, d]))
                    if v is not None:
                        count += 1
                        for x in v:
                            total = (total * 31 + x) & M64
    print(label, count)
    print(label, total)


def main():
    for s in [b"", b"f", b"fo", b"foo", b"foob", b"fooba", b"foobar"]:
        t = base64.b64encode(s)
        out("enc", t)
        out("dec", decode(t))
    for t in [b"Zg=", b"Zg", b"Z===", b"====", b"=Zm9", b"Zg==Zg==", b"Zm9v====", b"Zh==", b"Zm9=",
              b"Zm 9", b"Zm9v\n", b"Zm-v", b"Zm_v", b"Zm\x809", b"Zg==", b"Zm8=", b"Zm9v"]:
        v = decode(t)
        print("bad", "none" if v is None else len(v))
    s = stream(3000)
    for n in range(65):
        out("len", base64.b64encode(s[:n]))
    total = back = 0
    for n in range(3001):
        e = base64.b64encode(s[:n])
        total = (total + zlib.crc32(e)) & M64
        back += decode(e) == s[:n]
    print("crc", total)
    print("back", back)
    exhaustive("four", b"")
    exhaustive("eight", b"QUJD")


main()

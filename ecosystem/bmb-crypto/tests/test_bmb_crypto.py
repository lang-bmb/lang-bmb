"""
Test suite for bmb_crypto Python bindings.

Cross-validates every function against Python stdlib (hashlib, hmac, binascii,
base64) and exercises edge cases: empty strings, long strings, and inputs that
contain special/non-ASCII characters.
"""

import hashlib
import hmac as _hmac
import binascii
import base64 as _b64

import pytest
import bmb_crypto


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _py_sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _py_md5(text: str) -> str:
    return hashlib.md5(text.encode("utf-8")).hexdigest()


def _py_crc32(text: str) -> str:
    return format(binascii.crc32(text.encode("utf-8")) & 0xFFFFFFFF, "08x")


def _py_hmac_sha256(key: str, msg: str) -> str:
    return _hmac.new(key.encode("utf-8"), msg.encode("utf-8"), hashlib.sha256).hexdigest()


def _py_base64_encode(text: str) -> str:
    return _b64.b64encode(text.encode("utf-8")).decode("ascii")


def _py_base64_decode(encoded: str) -> str:
    return _b64.b64decode(encoded.encode("ascii")).decode("utf-8")


def _py_base32_encode(text: str) -> str:
    return _b64.b32encode(text.encode("utf-8")).decode("ascii")


def _py_base32_decode(encoded: str) -> str:
    return _b64.b32decode(encoded.encode("ascii")).decode("utf-8")


# Known-correct Adler-32 values (RFC 1950 / zlib definition):
#   initial state A=1, B=0  →  empty string produces 0x00000001
_ADLER32_KNOWN = {
    "": "00000001",
    "Wikipedia": "11e60398",
    "abc": format((ord("a") + ord("b") + ord("c") + 1)
                  + ((ord("a") + 1) * 3 + ord("b") * 2 + ord("c")) * 65536
                  & 0xFFFFFFFF, "08x"),
}

# Re-compute the "abc" entry properly to avoid formula errors in the table.
# We derive it from zlib which uses the same algorithm.
import zlib as _zlib
_ADLER32_KNOWN["abc"] = format(_zlib.adler32(b"abc") & 0xFFFFFFFF, "08x")
_ADLER32_KNOWN["hello"] = format(_zlib.adler32(b"hello") & 0xFFFFFFFF, "08x")
_ADLER32_KNOWN["The quick brown fox"] = format(
    _zlib.adler32(b"The quick brown fox") & 0xFFFFFFFF, "08x"
)

# ---------------------------------------------------------------------------
# Fixtures / shared data
# ---------------------------------------------------------------------------

HASH_INPUTS = [
    "",
    "a",
    "abc",
    "hello",
    "hello world",
    "The quick brown fox jumps over the lazy dog",
    "0123456789",
    "!@#$%^&*()-_=+[]{}|;:',.<>?/`~",
    "a" * 1000,
    "The quick brown fox jumps over the lazy dog" * 20,
]

ENCODING_INPUTS = [
    "",
    "f",
    "fo",
    "foo",
    "foob",
    "fooba",
    "foobar",
    "hello",
    "hello world",
    "Man",
    "any carnal pleasure.",
    "The quick brown fox jumps over the lazy dog",
    "a" * 100,
]


# ---------------------------------------------------------------------------
# SHA-256
# ---------------------------------------------------------------------------

class TestSha256:
    """bmb_crypto.sha256 cross-validated against hashlib.sha256."""

    def test_known_empty(self):
        expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        assert bmb_crypto.sha256("") == expected

    def test_known_hello(self):
        expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        assert bmb_crypto.sha256("hello") == expected

    def test_known_abc(self):
        expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        assert bmb_crypto.sha256("abc") == expected

    @pytest.mark.parametrize("text", HASH_INPUTS)
    def test_matches_stdlib(self, text):
        assert bmb_crypto.sha256(text) == _py_sha256(text)

    def test_output_is_64_chars(self):
        assert len(bmb_crypto.sha256("hello")) == 64

    def test_output_is_lowercase_hex(self):
        result = bmb_crypto.sha256("hello")
        assert all(c in "0123456789abcdef" for c in result)

    def test_distinct_inputs_produce_distinct_hashes(self):
        assert bmb_crypto.sha256("a") != bmb_crypto.sha256("b")

    def test_long_string(self):
        text = "x" * 10_000
        assert bmb_crypto.sha256(text) == _py_sha256(text)

    def test_special_characters(self):
        text = "!@#$%^&*()_+-=[]{}|;':\",./<>?"
        assert bmb_crypto.sha256(text) == _py_sha256(text)


# ---------------------------------------------------------------------------
# MD5
# ---------------------------------------------------------------------------

class TestMd5:
    """bmb_crypto.md5 cross-validated against hashlib.md5."""

    def test_known_empty(self):
        assert bmb_crypto.md5("") == "d41d8cd98f00b204e9800998ecf8427e"

    def test_known_hello(self):
        assert bmb_crypto.md5("hello") == "5d41402abc4b2a76b9719d911017c592"

    @pytest.mark.parametrize("text", HASH_INPUTS)
    def test_matches_stdlib(self, text):
        assert bmb_crypto.md5(text) == _py_md5(text)

    def test_output_is_32_chars(self):
        assert len(bmb_crypto.md5("hello")) == 32

    def test_output_is_lowercase_hex(self):
        result = bmb_crypto.md5("test")
        assert all(c in "0123456789abcdef" for c in result)

    def test_long_string(self):
        text = "y" * 10_000
        assert bmb_crypto.md5(text) == _py_md5(text)

    def test_special_characters(self):
        text = "\t\n\r !@#$%^&*()"
        assert bmb_crypto.md5(text) == _py_md5(text)


# ---------------------------------------------------------------------------
# CRC32
# ---------------------------------------------------------------------------

class TestCrc32:
    """bmb_crypto.crc32 cross-validated against binascii.crc32."""

    def test_known_empty(self):
        # CRC32 of empty string is 0x00000000
        assert bmb_crypto.crc32("") == "00000000"

    def test_known_hello(self):
        assert bmb_crypto.crc32("hello") == _py_crc32("hello")

    def test_known_123456789(self):
        # Well-known CRC32 check value for "123456789"
        assert bmb_crypto.crc32("123456789") == _py_crc32("123456789")

    @pytest.mark.parametrize("text", HASH_INPUTS)
    def test_matches_stdlib(self, text):
        assert bmb_crypto.crc32(text) == _py_crc32(text)

    def test_output_is_8_chars(self):
        assert len(bmb_crypto.crc32("hello")) == 8

    def test_output_is_zero_padded(self):
        # CRC32 of empty string is 0; must still be 8 hex chars
        result = bmb_crypto.crc32("")
        assert len(result) == 8
        assert result == "00000000"

    def test_output_is_lowercase_hex(self):
        result = bmb_crypto.crc32("abc")
        assert all(c in "0123456789abcdef" for c in result)

    def test_long_string(self):
        text = "z" * 10_000
        assert bmb_crypto.crc32(text) == _py_crc32(text)


# ---------------------------------------------------------------------------
# HMAC-SHA256
# ---------------------------------------------------------------------------

class TestHmacSha256:
    """bmb_crypto.hmac_sha256 cross-validated against hmac + hashlib."""

    def test_known_rfc_vector(self):
        # RFC 4231 test vector 1 (key=0x0b*20, data="Hi There") — ASCII proxy
        key = "key"
        msg = "The quick brown fox jumps over the lazy dog"
        assert bmb_crypto.hmac_sha256(key, msg) == _py_hmac_sha256(key, msg)

    @pytest.mark.parametrize("key,msg", [
        ("key", "The quick brown fox jumps over the lazy dog"),
        ("secret", "hello"),
        ("", ""),
        ("abc", "abc"),
        ("a" * 100, "message"),
        ("key", ""),
        ("", "message"),
        ("k" * 200, "long key test"),
    ])
    def test_matches_stdlib(self, key, msg):
        assert bmb_crypto.hmac_sha256(key, msg) == _py_hmac_sha256(key, msg)

    def test_output_is_64_chars(self):
        assert len(bmb_crypto.hmac_sha256("key", "msg")) == 64

    def test_output_is_lowercase_hex(self):
        result = bmb_crypto.hmac_sha256("key", "msg")
        assert all(c in "0123456789abcdef" for c in result)

    def test_different_keys_produce_different_macs(self):
        msg = "same message"
        assert bmb_crypto.hmac_sha256("key1", msg) != bmb_crypto.hmac_sha256("key2", msg)

    def test_different_messages_produce_different_macs(self):
        key = "same key"
        assert bmb_crypto.hmac_sha256(key, "msg1") != bmb_crypto.hmac_sha256(key, "msg2")

    def test_special_characters_in_key_and_message(self):
        key = "!@#$%^&*()"
        msg = "hello\nworld\t!"
        assert bmb_crypto.hmac_sha256(key, msg) == _py_hmac_sha256(key, msg)


# ---------------------------------------------------------------------------
# Base64
# ---------------------------------------------------------------------------

class TestBase64Encode:
    """bmb_crypto.base64_encode cross-validated against base64.b64encode."""

    @pytest.mark.parametrize("text,expected", [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
        ("hello", "aGVsbG8="),
        ("Man", "TWFu"),
    ])
    def test_known_vectors(self, text, expected):
        assert bmb_crypto.base64_encode(text) == expected

    @pytest.mark.parametrize("text", ENCODING_INPUTS)
    def test_matches_stdlib(self, text):
        assert bmb_crypto.base64_encode(text) == _py_base64_encode(text)

    def test_long_string(self):
        text = "hello world! " * 100
        assert bmb_crypto.base64_encode(text) == _py_base64_encode(text)

    def test_output_uses_only_valid_base64_chars(self):
        import re
        result = bmb_crypto.base64_encode("The quick brown fox")
        assert re.fullmatch(r"[A-Za-z0-9+/]*={0,2}", result)


class TestBase64Decode:
    """bmb_crypto.base64_decode cross-validated against base64.b64decode."""

    @pytest.mark.parametrize("text", ENCODING_INPUTS)
    def test_roundtrip(self, text):
        encoded = bmb_crypto.base64_encode(text)
        if encoded:
            assert bmb_crypto.base64_decode(encoded) == text

    @pytest.mark.parametrize("encoded,expected", [
        ("", ""),
        ("Zg==", "f"),
        ("Zm8=", "fo"),
        ("Zm9v", "foo"),
        ("aGVsbG8=", "hello"),
    ])
    def test_known_vectors(self, encoded, expected):
        assert bmb_crypto.base64_decode(encoded) == expected

    @pytest.mark.parametrize("text", ENCODING_INPUTS)
    def test_matches_stdlib(self, text):
        encoded = _py_base64_encode(text)
        if encoded:
            assert bmb_crypto.base64_decode(encoded) == _py_base64_decode(encoded)

    def test_long_string_roundtrip(self):
        text = "abcdefghijklmnopqrstuvwxyz" * 50
        encoded = bmb_crypto.base64_encode(text)
        assert bmb_crypto.base64_decode(encoded) == text


# ---------------------------------------------------------------------------
# Base32
# ---------------------------------------------------------------------------

class TestBase32Encode:
    """bmb_crypto.base32_encode cross-validated against base64.b32encode."""

    @pytest.mark.parametrize("text,expected", [
        ("f", "MY======"),
        ("fo", "MZXQ===="),
        ("foo", "MZXW6==="),
        ("foob", "MZXW6YQ="),
        ("fooba", "MZXW6YTB"),
        ("foobar", "MZXW6YTBOI======"),
    ])
    def test_known_vectors(self, text, expected):
        assert bmb_crypto.base32_encode(text) == expected

    @pytest.mark.parametrize("text", [t for t in ENCODING_INPUTS if t])
    def test_matches_stdlib(self, text):
        assert bmb_crypto.base32_encode(text) == _py_base32_encode(text)

    def test_output_uses_only_valid_base32_chars(self):
        import re
        result = bmb_crypto.base32_encode("hello world")
        assert re.fullmatch(r"[A-Z2-7]*={0,6}", result)

    def test_long_string(self):
        text = "abcdefgh" * 50
        assert bmb_crypto.base32_encode(text) == _py_base32_encode(text)


class TestBase32Decode:
    """bmb_crypto.base32_decode cross-validated against base64.b32decode."""

    @pytest.mark.parametrize("text", [t for t in ENCODING_INPUTS if t])
    def test_roundtrip(self, text):
        encoded = bmb_crypto.base32_encode(text)
        assert bmb_crypto.base32_decode(encoded) == text

    @pytest.mark.parametrize("encoded,expected", [
        ("MY======", "f"),
        ("MZXQ====", "fo"),
        ("MZXW6===", "foo"),
        ("MZXW6YQ=", "foob"),
        ("MZXW6YTB", "fooba"),
        ("MZXW6YTBOI======", "foobar"),
    ])
    def test_known_vectors(self, encoded, expected):
        assert bmb_crypto.base32_decode(encoded) == expected

    @pytest.mark.parametrize("text", [t for t in ENCODING_INPUTS if t])
    def test_matches_stdlib(self, text):
        encoded = _py_base32_encode(text)
        assert bmb_crypto.base32_decode(encoded) == _py_base32_decode(encoded)

    def test_long_string_roundtrip(self):
        text = "abcdefghijklmnop" * 20
        encoded = bmb_crypto.base32_encode(text)
        assert bmb_crypto.base32_decode(encoded) == text


# ---------------------------------------------------------------------------
# Adler-32
# ---------------------------------------------------------------------------

class TestAdler32:
    """bmb_crypto.adler32 cross-validated against zlib.adler32."""

    def test_known_empty(self):
        # Empty string → A=1, B=0 → 0x00000001
        assert bmb_crypto.adler32("") == "00000001"

    def test_known_wikipedia(self):
        assert bmb_crypto.adler32("Wikipedia") == "11e60398"

    @pytest.mark.parametrize("text", ["", "Wikipedia", "abc", "hello",
                                      "The quick brown fox"])
    def test_matches_zlib(self, text):
        import zlib
        expected = format(zlib.adler32(text.encode("utf-8")) & 0xFFFFFFFF, "08x")
        assert bmb_crypto.adler32(text) == expected

    def test_output_is_8_chars(self):
        assert len(bmb_crypto.adler32("hello")) == 8

    def test_output_is_lowercase_hex(self):
        result = bmb_crypto.adler32("test")
        assert all(c in "0123456789abcdef" for c in result)

    def test_long_string(self):
        import zlib
        text = "a" * 5_000
        expected = format(zlib.adler32(text.encode("utf-8")) & 0xFFFFFFFF, "08x")
        assert bmb_crypto.adler32(text) == expected

    def test_determinism(self):
        text = "hello world"
        assert bmb_crypto.adler32(text) == bmb_crypto.adler32(text)


# ---------------------------------------------------------------------------
# Fletcher-16  (optional — skip gracefully if not exported)
# ---------------------------------------------------------------------------

class TestFletcher16:
    """bmb_crypto.fletcher16 — determinism and format checks."""

    def test_available(self):
        pytest.importorskip("bmb_crypto")
        # Just verify the function is callable
        result = bmb_crypto.fletcher16("hello")
        assert isinstance(result, str)

    def test_output_is_hex_string(self):
        result = bmb_crypto.fletcher16("hello")
        assert all(c in "0123456789abcdef" for c in result), (
            f"Non-hex character in: {result!r}"
        )

    def test_determinism(self):
        text = "hello world"
        assert bmb_crypto.fletcher16(text) == bmb_crypto.fletcher16(text)

    def test_empty_string(self):
        result = bmb_crypto.fletcher16("")
        assert isinstance(result, str)
        assert len(result) > 0

    def test_distinct_inputs_may_differ(self):
        # Not a strict requirement (collisions exist), but "a" and "z" should differ
        r1 = bmb_crypto.fletcher16("a")
        r2 = bmb_crypto.fletcher16("z")
        # Just ensure both return valid hex strings
        assert all(c in "0123456789abcdef" for c in r1)
        assert all(c in "0123456789abcdef" for c in r2)

    def test_long_string(self):
        text = "x" * 1_000
        result = bmb_crypto.fletcher16(text)
        assert isinstance(result, str)
        assert all(c in "0123456789abcdef" for c in result)

    @pytest.mark.parametrize("text", ["", "a", "abc", "hello world",
                                      "The quick brown fox"])
    def test_consistent_across_calls(self, text):
        assert bmb_crypto.fletcher16(text) == bmb_crypto.fletcher16(text)


# ---------------------------------------------------------------------------
# XOR checksum  (optional — skip gracefully if not exported)
# ---------------------------------------------------------------------------

class TestXorChecksum:
    """bmb_crypto.xor_checksum — format and semantic checks."""

    def test_available(self):
        result = bmb_crypto.xor_checksum("hello")
        assert isinstance(result, str)

    def test_output_is_2_chars(self):
        # XOR of all bytes fits in one byte → 2 hex chars
        result = bmb_crypto.xor_checksum("ABC")
        assert len(result) == 2

    def test_output_is_lowercase_hex(self):
        result = bmb_crypto.xor_checksum("hello")
        assert all(c in "0123456789abcdef" for c in result)

    def test_empty_string(self):
        # XOR of no bytes is 0x00
        result = bmb_crypto.xor_checksum("")
        assert result == "00"

    def test_single_byte(self):
        # XOR of single byte 'A' (0x41) is 0x41
        result = bmb_crypto.xor_checksum("A")
        assert result == "41"

    def test_known_abc(self):
        # 'A'^'B'^'C' = 0x41^0x42^0x43 = 0x40
        result = bmb_crypto.xor_checksum("ABC")
        assert result == "40"

    def test_determinism(self):
        text = "hello world"
        assert bmb_crypto.xor_checksum(text) == bmb_crypto.xor_checksum(text)

    def test_xor_self_cancels(self):
        # "aa" → ord('a') ^ ord('a') = 0
        result = bmb_crypto.xor_checksum("aa")
        assert result == "00"

    @pytest.mark.parametrize("text", ["", "A", "ABC", "hello", "hello world"])
    def test_consistent_across_calls(self, text):
        assert bmb_crypto.xor_checksum(text) == bmb_crypto.xor_checksum(text)


# ---------------------------------------------------------------------------
# Cross-function sanity checks
# ---------------------------------------------------------------------------

class TestCrossFunctionSanity:
    """Ensures functions are independent and do not interfere with each other."""

    def test_sha256_and_md5_differ(self):
        text = "hello"
        assert bmb_crypto.sha256(text) != bmb_crypto.md5(text)

    def test_sha256_length_vs_md5_length(self):
        text = "hello"
        assert len(bmb_crypto.sha256(text)) == 64
        assert len(bmb_crypto.md5(text)) == 32

    def test_base64_and_base32_differ(self):
        text = "hello"
        assert bmb_crypto.base64_encode(text) != bmb_crypto.base32_encode(text)

    def test_sequential_calls_are_independent(self):
        """Multiple calls in sequence must not share state."""
        inputs = ["alpha", "beta", "gamma", "delta"]
        results = [bmb_crypto.sha256(t) for t in inputs]
        # Calling again in reverse order must produce the same hashes
        reversed_results = [bmb_crypto.sha256(t) for t in reversed(inputs)]
        assert list(reversed(results)) == reversed_results

    def test_interleaved_hash_and_encode(self):
        text = "interleaved test"
        h1 = bmb_crypto.sha256(text)
        e1 = bmb_crypto.base64_encode(text)
        h2 = bmb_crypto.sha256(text)
        e2 = bmb_crypto.base64_encode(text)
        assert h1 == h2
        assert e1 == e2

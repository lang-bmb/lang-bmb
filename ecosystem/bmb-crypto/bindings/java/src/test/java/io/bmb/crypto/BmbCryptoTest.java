package io.bmb.crypto;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BmbCryptoTest {

    // SHA-256 (known vectors)
    @Test void testSha256Empty() {
        assertEquals("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                     BmbCrypto.sha256(""));
    }
    @Test void testSha256Hello() {
        assertEquals("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
                     BmbCrypto.sha256("hello"));
    }

    // MD5 (known vectors)
    @Test void testMd5Empty() {
        assertEquals("d41d8cd98f00b204e9800998ecf8427e", BmbCrypto.md5(""));
    }
    @Test void testMd5Hello() {
        assertEquals("5d41402abc4b2a76b9719d911017c592", BmbCrypto.md5("hello"));
    }

    // Base64 roundtrip
    @Test void testBase64Roundtrip() {
        String encoded = BmbCrypto.base64Encode("Hello, World!");
        assertEquals("Hello, World!", BmbCrypto.base64Decode(encoded));
    }
    @Test void testBase64Empty() {
        assertEquals("", BmbCrypto.base64Encode(""));
        assertEquals("", BmbCrypto.base64Decode(""));
    }

    // Base32 roundtrip
    @Test void testBase32Roundtrip() {
        String encoded = BmbCrypto.base32Encode("hello");
        assertEquals("hello", BmbCrypto.base32Decode(encoded));
    }

    // Hex roundtrip
    @Test void testHexRoundtrip() {
        String encoded = BmbCrypto.hexEncode("hi");
        assertEquals("hi", BmbCrypto.hexDecode(encoded));
    }
    @Test void testHexEncode() {
        assertEquals("68656c6c6f", BmbCrypto.hexEncode("hello"));
    }

    // ROT-13 (symmetric)
    @Test void testRot13() {
        assertEquals("uryyb", BmbCrypto.rot13("hello"));
        assertEquals("hello", BmbCrypto.rot13(BmbCrypto.rot13("hello")));
    }

    // CRC-32 / Adler-32 / Fletcher-16 / XOR (non-empty)
    @Test void testCrc32NonEmpty()     { assertFalse(BmbCrypto.crc32("abc").isEmpty()); }
    @Test void testAdler32NonEmpty()   { assertFalse(BmbCrypto.adler32("abc").isEmpty()); }
    @Test void testFletcher16NonEmpty(){ assertFalse(BmbCrypto.fletcher16("abc").isEmpty()); }
    @Test void testXorChecksumNonEmpty(){ assertFalse(BmbCrypto.xorChecksum("abc").isEmpty()); }

    // HMAC-SHA256 (non-empty)
    @Test void testHmacSha256NonEmpty() {
        assertFalse(BmbCrypto.hmacSha256("key", "message").isEmpty());
    }
}

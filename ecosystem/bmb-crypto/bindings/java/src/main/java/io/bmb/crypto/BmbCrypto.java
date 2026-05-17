package io.bmb.crypto;

import com.sun.jna.Pointer;

import java.util.function.Supplier;

/**
 * Cryptographic utilities powered by BMB — JNA bindings.
 * All calls dispatch into the native bmb_crypto shared library.
 *
 * Thread safety: each public method wraps calls in bmb_ffi_begin/end.
 * All hash/encode functions return hex or Base64 strings (heap-allocated).
 */
public final class BmbCrypto {
    private static final BmbCryptoLib LIB = BmbCryptoLib.INSTANCE;

    private BmbCrypto() {}

    // ── Helpers ────────────────────────────────────────────────────────────────

    private static <T> T safe(Supplier<T> fn) {
        LIB.bmb_ffi_begin();
        try {
            T r = fn.get();
            if (LIB.bmb_ffi_has_error() != 0) {
                String msg = LIB.bmb_ffi_error_message().getString(0);
                throw new RuntimeException("BMB FFI error: " + msg);
            }
            return r;
        } finally {
            LIB.bmb_ffi_end();
        }
    }

    private static String stringResult(Pointer bmbStr) {
        if (bmbStr == null || bmbStr.equals(Pointer.NULL)) return "";
        String s = LIB.bmb_ffi_string_data(bmbStr).getString(0);
        LIB.bmb_ffi_free_string(bmbStr);
        return s;
    }

    private static String oneArg(String input,
            java.util.function.Function<Pointer, Pointer> fn) {
        Pointer p = LIB.bmb_ffi_cstr_to_string(input);
        try { return stringResult(safe(() -> fn.apply(p))); }
        finally { LIB.bmb_ffi_free_string(p); }
    }

    // ── Hashing ────────────────────────────────────────────────────────────────

    /** SHA-256 of {@code input}, returned as lowercase hex string. */
    public static String sha256(String input)        { return oneArg(input, LIB::bmb_sha256); }

    /** MD5 of {@code input}, returned as lowercase hex string. */
    public static String md5(String input)           { return oneArg(input, LIB::bmb_md5); }

    /** CRC-32 of {@code input}, returned as hex string. */
    public static String crc32(String input)         { return oneArg(input, LIB::bmb_crc32); }

    /** Adler-32 checksum of {@code input}, returned as hex string. */
    public static String adler32(String input)       { return oneArg(input, LIB::bmb_adler32); }

    /** Fletcher-16 checksum of {@code input}, returned as hex string. */
    public static String fletcher16(String input)    { return oneArg(input, LIB::bmb_fletcher16); }

    /** XOR checksum of {@code input}, returned as hex string. */
    public static String xorChecksum(String input)   { return oneArg(input, LIB::bmb_xor_checksum); }

    /** HMAC-SHA256 of {@code msg} with {@code key}, returned as hex string. */
    public static String hmacSha256(String key, String msg) {
        Pointer pKey = LIB.bmb_ffi_cstr_to_string(key);
        Pointer pMsg = LIB.bmb_ffi_cstr_to_string(msg);
        try { return stringResult(safe(() -> LIB.bmb_hmac_sha256(pKey, pMsg))); }
        finally { LIB.bmb_ffi_free_string(pKey); LIB.bmb_ffi_free_string(pMsg); }
    }

    // ── Encoding / Decoding ────────────────────────────────────────────────────

    /** Base64-encode {@code input}. */
    public static String base64Encode(String input)  { return oneArg(input, LIB::bmb_base64_encode); }

    /** Base64-decode {@code input}. Returns {@code ""} on invalid padding. */
    public static String base64Decode(String input)  { return oneArg(input, LIB::bmb_base64_decode); }

    /** Base32-encode {@code input}. */
    public static String base32Encode(String input)  { return oneArg(input, LIB::bmb_base32_encode); }

    /** Base32-decode {@code input}. Returns {@code ""} on invalid padding. */
    public static String base32Decode(String input)  { return oneArg(input, LIB::bmb_base32_decode); }

    /** Hex-encode {@code input} (each byte as two lowercase hex digits). */
    public static String hexEncode(String input)     { return oneArg(input, LIB::bmb_hex_encode); }

    /** Hex-decode {@code input}. Returns {@code ""} on odd length or non-hex chars. */
    public static String hexDecode(String input)     { return oneArg(input, LIB::bmb_hex_decode); }

    // ── Cipher ─────────────────────────────────────────────────────────────────

    /** ROT-13 encode/decode {@code input} (symmetric). */
    public static String rot13(String input)         { return oneArg(input, LIB::bmb_rot13); }
}

/**
 * bmb-crypto C bindings — test suite
 *
 * Tests all 14 exported functions. Exit code 0 = all pass.
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_crypto.h"

static int g_pass = 0, g_fail = 0;

#define ASSERT_STR(label, got, expected) do { \
    if (strcmp((got), (expected)) == 0) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: got '%s', expected '%s'\n", label, (got), (expected)); g_fail++; \
    } \
} while(0)

#define ASSERT_TRUE(label, cond) do { \
    if (cond) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: condition false\n", label); g_fail++; \
    } \
} while(0)

#define FFI_END_CHECK(label) do { \
    if (bmb_ffi_has_error()) { \
        printf("[FAIL] FFI error in %s: %s\n", label, bmb_ffi_error_message()); g_fail++; \
    } \
    bmb_ffi_end(); \
} while(0)

/*
 * Arena note: output void* from @export functions is arena-allocated.
 * Read the string data BEFORE bmb_ffi_end(). Do NOT call bmb_ffi_free_string
 * on outputs — only call it on inputs created by bmb_ffi_cstr_to_string.
 */
static char result_buf[4096];

static const char *call1(void *(*fn)(void *), const char *input) {
    bmb_ffi_begin();
    void *in  = bmb_ffi_cstr_to_string(input);
    void *out = fn(in);
    /* copy output before ffi_end invalidates the arena */
    const char *s = bmb_ffi_string_data(out);
    strncpy(result_buf, s, sizeof(result_buf) - 1);
    result_buf[sizeof(result_buf) - 1] = '\0';
    bmb_ffi_free_string(in);   /* input: heap-alloc, safe to free */
    /* output (out): arena-alloc, do NOT free — bmb_ffi_end() handles it */
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "FFI error: %s\n", bmb_ffi_error_message());
    }
    bmb_ffi_end();
    return result_buf;
}

static const char *call2(void *(*fn)(void *, void *), const char *a, const char *b) {
    bmb_ffi_begin();
    void *sa  = bmb_ffi_cstr_to_string(a);
    void *sb  = bmb_ffi_cstr_to_string(b);
    void *out = fn(sa, sb);
    const char *s = bmb_ffi_string_data(out);
    strncpy(result_buf, s, sizeof(result_buf) - 1);
    result_buf[sizeof(result_buf) - 1] = '\0';
    bmb_ffi_free_string(sa);   /* inputs: heap-alloc, safe to free */
    bmb_ffi_free_string(sb);
    /* output (out): arena-alloc, do NOT free */
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "FFI error: %s\n", bmb_ffi_error_message());
    }
    bmb_ffi_end();
    return result_buf;
}

/* ── SHA-256 ──────────────────────────────────────────────────────────── */
static void test_sha256(void) {
    /* Known SHA-256 of "hello" */
    ASSERT_STR("sha256(hello)",
               call1(bmb_sha256, "hello"),
               "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    ASSERT_STR("sha256(empty)",
               call1(bmb_sha256, ""),
               "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

/* ── MD5 ──────────────────────────────────────────────────────────────── */
static void test_md5(void) {
    ASSERT_STR("md5(hello)",
               call1(bmb_md5, "hello"),
               "5d41402abc4b2a76b9719d911017c592");
    ASSERT_STR("md5(empty)",
               call1(bmb_md5, ""),
               "d41d8cd98f00b204e9800998ecf8427e");
}

/* ── CRC32 / Adler32 / Fletcher16 / XOR checksum ─────────────────────── */
static void test_checksums(void) {
    /* Determinism: call twice, results must match */
    char buf1[256], buf2[256];
    strncpy(buf1, call1(bmb_crc32, "hello"), sizeof(buf1) - 1);
    strncpy(buf2, call1(bmb_crc32, "hello"), sizeof(buf2) - 1);
    ASSERT_STR("crc32 deterministic", buf1, buf2);
    ASSERT_TRUE("crc32 non-empty", strlen(buf1) > 0);

    /* Different inputs → different outputs */
    char crc_hello[256], crc_world[256];
    strncpy(crc_hello, call1(bmb_crc32, "hello"), sizeof(crc_hello) - 1);
    strncpy(crc_world, call1(bmb_crc32, "world"), sizeof(crc_world) - 1);
    ASSERT_TRUE("crc32 differs on different input", strcmp(crc_hello, crc_world) != 0);

    ASSERT_TRUE("adler32 non-empty",    strlen(call1(bmb_adler32,    "hello")) > 0);
    ASSERT_TRUE("fletcher16 non-empty", strlen(call1(bmb_fletcher16, "hello")) > 0);
    ASSERT_TRUE("xor_checksum non-empty",strlen(call1(bmb_xor_checksum,"hello")) > 0);
}

/* ── Base64 ──────────────────────────────────────────────────────────── */
static void test_base64(void) {
    ASSERT_STR("base64_encode(Hello)",
               call1(bmb_base64_encode, "Hello"),
               "SGVsbG8=");
    ASSERT_STR("base64_encode(empty)", call1(bmb_base64_encode, ""), "");

    /* round-trip: encode first, copy result, then decode */
    const char *msg = "Hello, World!";
    char enc_copy[256];
    strncpy(enc_copy, call1(bmb_base64_encode, msg), sizeof(enc_copy) - 1);
    ASSERT_STR("base64 round-trip", call1(bmb_base64_decode, enc_copy), msg);
}

/* ── Base32 ──────────────────────────────────────────────────────────── */
static void test_base32(void) {
    /* RFC 4648 base32: "hello" → "NBSWY3DPEB3W64TMMQ======" */
    char enc_copy[256];
    strncpy(enc_copy, call1(bmb_base32_encode, "hello"), sizeof(enc_copy) - 1);
    ASSERT_TRUE("base32_encode non-empty", strlen(enc_copy) > 0);
    /* round-trip */
    ASSERT_STR("base32 round-trip", call1(bmb_base32_decode, enc_copy), "hello");
}

/* ── Hex encode/decode ───────────────────────────────────────────────── */
static void test_hex(void) {
    ASSERT_STR("hex_encode(hello)",
               call1(bmb_hex_encode, "hello"),
               "68656c6c6f");
    /* round-trip */
    char hex_copy[256];
    strncpy(hex_copy, call1(bmb_hex_encode, "Hello"), sizeof(hex_copy) - 1);
    ASSERT_STR("hex round-trip", call1(bmb_hex_decode, hex_copy), "Hello");
}

/* ── ROT13 ───────────────────────────────────────────────────────────── */
static void test_rot13(void) {
    ASSERT_STR("rot13(Hello)",   call1(bmb_rot13, "Hello"),   "Uryyb");
    ASSERT_STR("rot13(rot13)",   call1(bmb_rot13, "Uryyb"),   "Hello"); /* involution */
    ASSERT_STR("rot13(digits)",  call1(bmb_rot13, "123"),     "123");   /* non-alpha unchanged */
}

/* ── HMAC-SHA256 ─────────────────────────────────────────────────────── */
static void test_hmac(void) {
    char h1[256], h2[256], h3[256];
    strncpy(h1, call2(bmb_hmac_sha256, "key", "hello"), sizeof(h1) - 1);
    strncpy(h2, call2(bmb_hmac_sha256, "key", "hello"), sizeof(h2) - 1);
    ASSERT_STR("hmac_sha256 deterministic", h1, h2);
    ASSERT_TRUE("hmac_sha256 non-empty", strlen(h1) == 64); /* 32 bytes hex = 64 chars */
    strncpy(h3, call2(bmb_hmac_sha256, "other-key", "hello"), sizeof(h3) - 1);
    ASSERT_TRUE("hmac_sha256 key-sensitive", strcmp(h1, h3) != 0);
}

int main(void) {
    printf("=== bmb-crypto C binding tests ===\n\n");

    test_sha256();
    test_md5();
    test_checksums();
    test_base64();
    test_base32();
    test_hex();
    test_rot13();
    test_hmac();

    printf("\n=== Results: %d passed, %d failed ===\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}

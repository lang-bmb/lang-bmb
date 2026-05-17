/**
 * bmb-crypto C bindings — example usage
 *
 * Build:
 *   Windows: gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_crypto.dll
 *   Linux:   gcc -O2 -I../../include -o example example.c -L../.. -lbmb_crypto -Wl,-rpath,../..
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_crypto.h"

/*
 * Arena note: output void* from @export functions is arena-allocated.
 * Read the string data BEFORE bmb_ffi_end(). Do NOT call bmb_ffi_free_string
 * on outputs — only free inputs created by bmb_ffi_cstr_to_string.
 */

static void show(const char *fn_name, void *(*fn)(void *), const char *input) {
    bmb_ffi_begin();
    void *in  = bmb_ffi_cstr_to_string(input);
    void *out = fn(in);
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "Error in %s: %s\n", fn_name, bmb_ffi_error_message());
        bmb_ffi_end();
        return;
    }
    printf("%-20s(%s) = %s\n", fn_name, input, bmb_ffi_string_data(out));
    bmb_ffi_free_string(in);   /* input: safe to free */
    /* out: arena-backed, do NOT free */
    bmb_ffi_end();
}

static void show2(const char *fn_name, void *(*fn)(void *, void *),
                  const char *a, const char *b) {
    bmb_ffi_begin();
    void *sa  = bmb_ffi_cstr_to_string(a);
    void *sb  = bmb_ffi_cstr_to_string(b);
    void *out = fn(sa, sb);
    if (bmb_ffi_has_error()) {
        fprintf(stderr, "Error in %s: %s\n", fn_name, bmb_ffi_error_message());
        bmb_ffi_end();
        return;
    }
    printf("%-20s(%s, %s) = %s\n", fn_name, a, b, bmb_ffi_string_data(out));
    bmb_ffi_free_string(sa);   /* inputs: safe to free */
    bmb_ffi_free_string(sb);
    /* out: arena-backed, do NOT free */
    bmb_ffi_end();
}

int main(void) {
    const char *hello = "hello";
    const char *world = "Hello, World!";

    show("bmb_sha256",      bmb_sha256,      hello);
    show("bmb_md5",         bmb_md5,         hello);
    show("bmb_base64_encode", bmb_base64_encode, world);
    show("bmb_hex_encode",  bmb_hex_encode,  hello);
    show("bmb_rot13",       bmb_rot13,       "Hello");
    show("bmb_crc32",       bmb_crc32,       hello);
    show("bmb_adler32",     bmb_adler32,     hello);
    show("bmb_fletcher16",  bmb_fletcher16,  hello);
    show("bmb_xor_checksum",bmb_xor_checksum,hello);
    show("bmb_base32_encode",bmb_base32_encode, hello);
    show2("bmb_hmac_sha256",bmb_hmac_sha256, "key", hello);

    /* round-trip: base64 encode then decode — two separate FFI calls */
    char enc_buf[256];
    bmb_ffi_begin();
    void *orig = bmb_ffi_cstr_to_string(world);
    void *enc  = bmb_base64_encode(orig);
    strncpy(enc_buf, bmb_ffi_string_data(enc), sizeof(enc_buf) - 1);
    bmb_ffi_free_string(orig);
    /* enc: arena-backed, do NOT free */
    bmb_ffi_end();

    bmb_ffi_begin();
    void *enc_s = bmb_ffi_cstr_to_string(enc_buf);
    void *dec   = bmb_base64_decode(enc_s);
    printf("base64 round-trip OK: %d\n", strcmp(bmb_ffi_string_data(dec), world) == 0);
    bmb_ffi_free_string(enc_s);
    /* dec: arena-backed, do NOT free */
    bmb_ffi_end();

    /* round-trip: base32 encode then decode */
    char e32_buf[256];
    bmb_ffi_begin();
    void *o32 = bmb_ffi_cstr_to_string(hello);
    void *e32 = bmb_base32_encode(o32);
    strncpy(e32_buf, bmb_ffi_string_data(e32), sizeof(e32_buf) - 1);
    bmb_ffi_free_string(o32);
    bmb_ffi_end();

    bmb_ffi_begin();
    void *e32_s = bmb_ffi_cstr_to_string(e32_buf);
    void *d32   = bmb_base32_decode(e32_s);
    printf("base32 round-trip OK: %d\n", strcmp(bmb_ffi_string_data(d32), hello) == 0);
    bmb_ffi_free_string(e32_s);
    bmb_ffi_end();

    printf("All examples passed.\n");
    return 0;
}

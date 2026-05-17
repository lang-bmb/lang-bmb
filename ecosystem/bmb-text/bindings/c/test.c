/**
 * bmb-text C bindings — test suite
 *
 * Tests all 23 exported functions. Exit code 0 = all pass.
 *
 * Arena rule: @export String return values are arena-backed.
 *   - Read data before bmb_ffi_end().
 *   - DO NOT call bmb_ffi_free_string on output strings.
 *   - Only free inputs from bmb_ffi_cstr_to_string.
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_text.h"

static int g_pass = 0, g_fail = 0;

#define ASSERT_EQ(label, got, expected) do { \
    int64_t _g = (int64_t)(got); \
    int64_t _e = (int64_t)(expected); \
    if (_g == _e) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: got %lld, expected %lld\n", label, (long long)_g, (long long)_e); g_fail++; \
    } \
} while(0)

#define ASSERT_STR(label, got, expected) do { \
    if (strcmp((got), (expected)) == 0) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: got '%s', expected '%s'\n", label, (got), (expected)); g_fail++; \
    } \
} while(0)

#define FFI_END_CHECK(label) do { \
    if (bmb_ffi_has_error()) { \
        printf("[FAIL] FFI error in %s: %s\n", label, bmb_ffi_error_message()); g_fail++; \
    } \
    bmb_ffi_end(); \
} while(0)

/* ── Scalar search functions ──────────────────────────────────────────── */
static void test_scalar_search(void) {
    bmb_ffi_begin();
    void *hay  = bmb_ffi_cstr_to_string("Hello, World!");
    void *ndl  = bmb_ffi_cstr_to_string("World");
    void *l    = bmb_ffi_cstr_to_string("l");
    void *pfx  = bmb_ffi_cstr_to_string("Hello");
    void *sfx  = bmb_ffi_cstr_to_string("!");
    void *miss = bmb_ffi_cstr_to_string("xyz");

    ASSERT_EQ("str_find(World)=7",       bmb_str_find(hay, ndl),        7);
    ASSERT_EQ("str_find(xyz)=-1",        bmb_str_find(hay, miss),       -1);
    ASSERT_EQ("str_rfind(l)=10",         bmb_str_rfind(hay, l),         10);
    ASSERT_EQ("str_count(l)=3",          bmb_str_count(hay, l),         3);
    ASSERT_EQ("str_contains(World)=1",   bmb_str_contains(hay, ndl),    1);
    ASSERT_EQ("str_contains(xyz)=0",     bmb_str_contains(hay, miss),   0);
    ASSERT_EQ("str_starts_with(Hello)=1",bmb_str_starts_with(hay, pfx), 1);
    ASSERT_EQ("str_starts_with(xyz)=0",  bmb_str_starts_with(hay, miss),0);
    ASSERT_EQ("str_ends_with(!)=1",      bmb_str_ends_with(hay, sfx),   1);
    ASSERT_EQ("str_ends_with(xyz)=0",    bmb_str_ends_with(hay, miss),  0);
    ASSERT_EQ("text_len=13",             bmb_text_len(hay),             13);
    ASSERT_EQ("word_count=2",            bmb_word_count(hay),           2);
    ASSERT_EQ("str_char_at(0)=72",       bmb_str_char_at(hay, 0),       72); /* 'H' */
    ASSERT_EQ("str_char_at(OOB)=-1",     bmb_str_char_at(hay, 100),     -1);
    ASSERT_EQ("str_compare(self)=0",     bmb_str_compare(hay, hay),     0);
    ASSERT_EQ("kmp_search(World)=7",     bmb_kmp_search(hay, ndl),      7);
    ASSERT_EQ("kmp_search(xyz)=-1",      bmb_kmp_search(hay, miss),     -1);
    ASSERT_EQ("str_find_byte(W=87)=7",   bmb_str_find_byte(hay, 87),    7);
    ASSERT_EQ("str_count_byte(l=108)=3", bmb_str_count_byte(hay, 108),  3);

    bmb_ffi_free_string(hay); bmb_ffi_free_string(ndl);
    bmb_ffi_free_string(l);   bmb_ffi_free_string(pfx);
    bmb_ffi_free_string(sfx); bmb_ffi_free_string(miss);
    FFI_END_CHECK("scalar_search");
}

/* ── Palindrome / hamming / token ─────────────────────────────────────── */
static void test_misc_scalar(void) {
    bmb_ffi_begin();
    ASSERT_EQ("is_palindrome(racecar)=1",
              bmb_is_palindrome(bmb_ffi_cstr_to_string("racecar")), 1);
    ASSERT_EQ("is_palindrome(hello)=0",
              bmb_is_palindrome(bmb_ffi_cstr_to_string("hello")),   0);
    ASSERT_EQ("is_palindrome(a)=1",
              bmb_is_palindrome(bmb_ffi_cstr_to_string("a")),       1);
    ASSERT_EQ("str_hamming(ab,ac)=1",
              bmb_str_hamming(bmb_ffi_cstr_to_string("ab"),
                              bmb_ffi_cstr_to_string("ac")),        1);
    ASSERT_EQ("str_hamming(diff_len)=-1",
              bmb_str_hamming(bmb_ffi_cstr_to_string("abc"),
                              bmb_ffi_cstr_to_string("ab")),        -1);
    ASSERT_EQ("token_count(a,b,c)=3",
              bmb_token_count(bmb_ffi_cstr_to_string("a,b,c"), (int64_t)','), 3);
    FFI_END_CHECK("misc_scalar");
}

/* ── String-returning functions ──────────────────────────────────────── */
static void test_str_transform(void) {
    char buf[256];

    /* str_reverse */
    bmb_ffi_begin();
    void *in = bmb_ffi_cstr_to_string("Hello");
    void *out = bmb_str_reverse(in);
    ASSERT_STR("str_reverse(Hello)=olleH", bmb_ffi_string_data(out), "olleH");
    bmb_ffi_free_string(in);
    FFI_END_CHECK("str_reverse");

    /* str_to_upper */
    bmb_ffi_begin();
    in  = bmb_ffi_cstr_to_string("hello world");
    out = bmb_str_to_upper(in);
    ASSERT_STR("str_to_upper", bmb_ffi_string_data(out), "HELLO WORLD");
    bmb_ffi_free_string(in);
    FFI_END_CHECK("str_to_upper");

    /* str_to_lower */
    bmb_ffi_begin();
    in  = bmb_ffi_cstr_to_string("HELLO WORLD");
    out = bmb_str_to_lower(in);
    ASSERT_STR("str_to_lower", bmb_ffi_string_data(out), "hello world");
    bmb_ffi_free_string(in);
    FFI_END_CHECK("str_to_lower");

    /* str_trim */
    bmb_ffi_begin();
    in  = bmb_ffi_cstr_to_string("  hello  ");
    out = bmb_str_trim(in);
    ASSERT_STR("str_trim", bmb_ffi_string_data(out), "hello");
    bmb_ffi_free_string(in);
    FFI_END_CHECK("str_trim");

    /* str_repeat */
    bmb_ffi_begin();
    in  = bmb_ffi_cstr_to_string("ab");
    out = bmb_str_repeat(in, 3);
    ASSERT_STR("str_repeat(ab,3)=ababab", bmb_ffi_string_data(out), "ababab");
    bmb_ffi_free_string(in);
    FFI_END_CHECK("str_repeat");

    /* str_replace (first) */
    bmb_ffi_begin();
    void *src = bmb_ffi_cstr_to_string("foo bar foo");
    void *old = bmb_ffi_cstr_to_string("foo");
    void *nw  = bmb_ffi_cstr_to_string("baz");
    out = bmb_str_replace(src, old, nw);
    ASSERT_STR("str_replace", bmb_ffi_string_data(out), "baz bar foo");
    bmb_ffi_free_string(src); bmb_ffi_free_string(old); bmb_ffi_free_string(nw);
    FFI_END_CHECK("str_replace");

    /* str_replace_all */
    bmb_ffi_begin();
    src = bmb_ffi_cstr_to_string("foo bar foo");
    old = bmb_ffi_cstr_to_string("foo");
    nw  = bmb_ffi_cstr_to_string("baz");
    out = bmb_str_replace_all(src, old, nw);
    ASSERT_STR("str_replace_all", bmb_ffi_string_data(out), "baz bar baz");
    bmb_ffi_free_string(src); bmb_ffi_free_string(old); bmb_ffi_free_string(nw);
    FFI_END_CHECK("str_replace_all");

    /* round-trip: reverse twice = identity */
    bmb_ffi_begin();
    in  = bmb_ffi_cstr_to_string("Hello");
    out = bmb_str_reverse(in);
    strncpy(buf, bmb_ffi_string_data(out), sizeof(buf) - 1);
    bmb_ffi_free_string(in);
    FFI_END_CHECK("reverse_step1");

    bmb_ffi_begin();
    in  = bmb_ffi_cstr_to_string(buf);
    out = bmb_str_reverse(in);
    ASSERT_STR("reverse round-trip", bmb_ffi_string_data(out), "Hello");
    bmb_ffi_free_string(in);
    FFI_END_CHECK("reverse_step2");
}

int main(void) {
    printf("=== bmb-text C binding tests ===\n\n");

    test_scalar_search();
    test_misc_scalar();
    test_str_transform();

    printf("\n=== Results: %d passed, %d failed ===\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}

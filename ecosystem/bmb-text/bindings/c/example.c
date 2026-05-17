/**
 * bmb-text C bindings — example usage
 *
 * Build:
 *   Windows: gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_text.dll
 *   Linux:   gcc -O2 -I../../include -o example example.c -L../.. -lbmb_text -Wl,-rpath,../..
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_text.h"

/*
 * Arena rule: return values of @export String functions are arena-backed.
 * Read data before bmb_ffi_end(). DO NOT call bmb_ffi_free_string on outputs.
 * Only free inputs created by bmb_ffi_cstr_to_string.
 */

int main(void) {
    /* ── Scalar searches ─────────────────────────────────────── */
    bmb_ffi_begin();
    void *haystack = bmb_ffi_cstr_to_string("Hello, World!");
    void *needle   = bmb_ffi_cstr_to_string("World");
    void *empty    = bmb_ffi_cstr_to_string("");

    printf("str_find(World)        = %lld\n", (long long)bmb_str_find(haystack, needle));
    printf("str_rfind(l)           = %lld\n", (long long)bmb_str_rfind(haystack, bmb_ffi_cstr_to_string("l")));
    printf("str_count(l)           = %lld\n", (long long)bmb_str_count(haystack, bmb_ffi_cstr_to_string("l")));
    printf("str_contains(World)    = %lld\n", (long long)bmb_str_contains(haystack, needle));
    printf("str_starts_with(Hello) = %lld\n", (long long)bmb_str_starts_with(haystack, bmb_ffi_cstr_to_string("Hello")));
    printf("str_ends_with(!)       = %lld\n", (long long)bmb_str_ends_with(haystack, bmb_ffi_cstr_to_string("!")));
    printf("text_len               = %lld\n", (long long)bmb_text_len(haystack));
    printf("word_count             = %lld\n", (long long)bmb_word_count(haystack));
    printf("is_palindrome(racecar) = %lld\n", (long long)bmb_is_palindrome(bmb_ffi_cstr_to_string("racecar")));
    printf("str_char_at(0)         = %c\n",   (char)bmb_str_char_at(haystack, 0));
    printf("str_compare(eq)        = %lld\n", (long long)bmb_str_compare(haystack, haystack));
    printf("kmp_search(World)      = %lld\n", (long long)bmb_kmp_search(haystack, needle));
    printf("str_hamming(ab,ac)     = %lld\n", (long long)bmb_str_hamming(
        bmb_ffi_cstr_to_string("ab"), bmb_ffi_cstr_to_string("ac")));
    printf("token_count(a,b,c)     = %lld\n", (long long)bmb_token_count(
        bmb_ffi_cstr_to_string("a,b,c"), (int64_t)','));
    printf("str_find_byte(W=87)    = %lld\n", (long long)bmb_str_find_byte(haystack, 87));
    printf("str_count_byte(l=108)  = %lld\n", (long long)bmb_str_count_byte(haystack, 108));

    bmb_ffi_free_string(haystack);
    bmb_ffi_free_string(needle);
    bmb_ffi_free_string(empty);
    bmb_ffi_end();

    /* ── String-returning functions (arena-backed output) ──── */
    bmb_ffi_begin();
    void *s = bmb_ffi_cstr_to_string("  Hello, World!  ");
    void *trimmed  = bmb_str_trim(s);
    void *upper    = bmb_str_to_upper(s);
    void *lower    = bmb_str_to_lower(s);
    printf("str_trim               = [%s]\n", bmb_ffi_string_data(trimmed));
    printf("str_to_upper           = %s\n",   bmb_ffi_string_data(upper));
    printf("str_to_lower           = %s\n",   bmb_ffi_string_data(lower));
    /* outputs (trimmed/upper/lower) are arena-backed: do NOT free */
    bmb_ffi_free_string(s);
    bmb_ffi_end();

    bmb_ffi_begin();
    void *rev_in  = bmb_ffi_cstr_to_string("Hello");
    void *rev_out = bmb_str_reverse(rev_in);
    printf("str_reverse(Hello)     = %s\n", bmb_ffi_string_data(rev_out));
    bmb_ffi_free_string(rev_in);
    bmb_ffi_end();

    bmb_ffi_begin();
    void *rep_in  = bmb_ffi_cstr_to_string("ab");
    void *rep_out = bmb_str_repeat(rep_in, 3);
    printf("str_repeat(ab,3)       = %s\n", bmb_ffi_string_data(rep_out));
    bmb_ffi_free_string(rep_in);
    bmb_ffi_end();

    bmb_ffi_begin();
    void *src     = bmb_ffi_cstr_to_string("foo bar foo");
    void *old_pat = bmb_ffi_cstr_to_string("foo");
    void *new_pat = bmb_ffi_cstr_to_string("baz");
    void *replaced     = bmb_str_replace(src, old_pat, new_pat);
    void *replaced_all = bmb_str_replace_all(src, old_pat, new_pat);
    printf("str_replace            = %s\n", bmb_ffi_string_data(replaced));
    printf("str_replace_all        = %s\n", bmb_ffi_string_data(replaced_all));
    bmb_ffi_free_string(src);
    bmb_ffi_free_string(old_pat);
    bmb_ffi_free_string(new_pat);
    bmb_ffi_end();

    printf("All examples passed.\n");
    return 0;
}

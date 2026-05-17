package io.bmb.text;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/** Raw JNA interface — direct 1:1 mapping of the bmb_text native C ABI. */
interface BmbTextLib extends Library {
    BmbTextLib INSTANCE = Native.load("bmb_text", BmbTextLib.class);

    // FFI Safety API
    int     bmb_ffi_begin();
    void    bmb_ffi_end();
    int     bmb_ffi_has_error();
    Pointer bmb_ffi_error_message();

    // String FFI API — BmbString* is opaque from Java
    Pointer bmb_ffi_cstr_to_string(String s);
    Pointer bmb_ffi_string_data(Pointer s);
    void    bmb_ffi_free_string(Pointer s);

    // Search (return position/count)
    long    bmb_kmp_search(Pointer text, Pointer pattern);
    long    bmb_str_find(Pointer haystack, Pointer needle);
    long    bmb_str_rfind(Pointer haystack, Pointer needle);
    long    bmb_str_count(Pointer haystack, Pointer needle);
    long    bmb_str_contains(Pointer haystack, Pointer needle);
    long    bmb_str_starts_with(Pointer s, Pointer prefix);
    long    bmb_str_ends_with(Pointer s, Pointer suffix);
    long    bmb_str_find_byte(Pointer s, long needle);
    long    bmb_str_count_byte(Pointer s, long needle);
    long    bmb_str_hamming(Pointer a, Pointer b);

    // Metrics
    long    bmb_is_palindrome(Pointer s);
    long    bmb_token_count(Pointer s, long delim);
    long    bmb_word_count(Pointer s);
    long    bmb_text_len(Pointer s);
    long    bmb_str_char_at(Pointer s, long idx);
    long    bmb_str_compare(Pointer a, Pointer b);

    // Transformations (return new BmbString*)
    Pointer bmb_str_reverse(Pointer s);
    Pointer bmb_str_replace(Pointer s, Pointer old_pat, Pointer new_pat);
    Pointer bmb_str_replace_all(Pointer s, Pointer old_pat, Pointer new_pat);
    Pointer bmb_str_to_upper(Pointer s);
    Pointer bmb_str_to_lower(Pointer s);
    Pointer bmb_str_trim(Pointer s);
    Pointer bmb_str_repeat(Pointer s, long n);
}

package io.bmb.text;

import com.sun.jna.Pointer;

import java.util.function.Supplier;

/**
 * Text processing powered by BMB — JNA bindings.
 * All calls dispatch into the native bmb_text shared library.
 *
 * Thread safety: each public method wraps calls in bmb_ffi_begin/end.
 */
public final class BmbText {
    private static final BmbTextLib LIB = BmbTextLib.INSTANCE;

    private BmbText() {}

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

    // BMB function results are arena-allocated (bmb_alloc), not malloc.
    // Read the data only — do NOT call bmb_ffi_free_string on the return value.
    // Inputs from bmb_ffi_cstr_to_string (malloc) are freed by the finally block.
    private static String stringResult(Pointer bmbStr) {
        if (bmbStr == null || bmbStr.equals(Pointer.NULL)) return "";
        return LIB.bmb_ffi_string_data(bmbStr).getString(0);
    }

    // ── Search ─────────────────────────────────────────────────────────────────

    /** KMP search: first occurrence of {@code pattern} in {@code text}, or -1. */
    public static long kmpSearch(String text, String pattern) {
        Pointer pt = LIB.bmb_ffi_cstr_to_string(text);
        Pointer pp = LIB.bmb_ffi_cstr_to_string(pattern);
        try { return safe(() -> LIB.bmb_kmp_search(pt, pp)); }
        finally { LIB.bmb_ffi_free_string(pt); LIB.bmb_ffi_free_string(pp); }
    }

    /** First occurrence of {@code needle} in {@code haystack}, or -1. */
    public static long find(String haystack, String needle) {
        Pointer ph = LIB.bmb_ffi_cstr_to_string(haystack);
        Pointer pn = LIB.bmb_ffi_cstr_to_string(needle);
        try { return safe(() -> LIB.bmb_str_find(ph, pn)); }
        finally { LIB.bmb_ffi_free_string(ph); LIB.bmb_ffi_free_string(pn); }
    }

    /** Last occurrence of {@code needle} in {@code haystack}, or -1. */
    public static long rfind(String haystack, String needle) {
        Pointer ph = LIB.bmb_ffi_cstr_to_string(haystack);
        Pointer pn = LIB.bmb_ffi_cstr_to_string(needle);
        try { return safe(() -> LIB.bmb_str_rfind(ph, pn)); }
        finally { LIB.bmb_ffi_free_string(ph); LIB.bmb_ffi_free_string(pn); }
    }

    /** Count non-overlapping occurrences of {@code needle} in {@code haystack}. */
    public static long count(String haystack, String needle) {
        Pointer ph = LIB.bmb_ffi_cstr_to_string(haystack);
        Pointer pn = LIB.bmb_ffi_cstr_to_string(needle);
        try { return safe(() -> LIB.bmb_str_count(ph, pn)); }
        finally { LIB.bmb_ffi_free_string(ph); LIB.bmb_ffi_free_string(pn); }
    }

    /** Returns true if {@code haystack} contains {@code needle}. */
    public static boolean contains(String haystack, String needle) {
        Pointer ph = LIB.bmb_ffi_cstr_to_string(haystack);
        Pointer pn = LIB.bmb_ffi_cstr_to_string(needle);
        try { return safe(() -> LIB.bmb_str_contains(ph, pn)) != 0; }
        finally { LIB.bmb_ffi_free_string(ph); LIB.bmb_ffi_free_string(pn); }
    }

    /** Returns true if {@code s} starts with {@code prefix}. */
    public static boolean startsWith(String s, String prefix) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        Pointer pp = LIB.bmb_ffi_cstr_to_string(prefix);
        try { return safe(() -> LIB.bmb_str_starts_with(ps, pp)) != 0; }
        finally { LIB.bmb_ffi_free_string(ps); LIB.bmb_ffi_free_string(pp); }
    }

    /** Returns true if {@code s} ends with {@code suffix}. */
    public static boolean endsWith(String s, String suffix) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        Pointer pp = LIB.bmb_ffi_cstr_to_string(suffix);
        try { return safe(() -> LIB.bmb_str_ends_with(ps, pp)) != 0; }
        finally { LIB.bmb_ffi_free_string(ps); LIB.bmb_ffi_free_string(pp); }
    }

    /** First position of byte value {@code needle} in {@code s}, or -1. */
    public static long findByte(String s, long needle) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_str_find_byte(ps, needle)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Count occurrences of byte value {@code needle} in {@code s}. */
    public static long countByte(String s, long needle) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_str_count_byte(ps, needle)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Hamming distance between {@code a} and {@code b} (must be same byte length). */
    public static long hamming(String a, String b) {
        Pointer pa = LIB.bmb_ffi_cstr_to_string(a);
        Pointer pb = LIB.bmb_ffi_cstr_to_string(b);
        try { return safe(() -> LIB.bmb_str_hamming(pa, pb)); }
        finally { LIB.bmb_ffi_free_string(pa); LIB.bmb_ffi_free_string(pb); }
    }

    // ── Metrics ────────────────────────────────────────────────────────────────

    /** Returns true if {@code s} is a byte-level palindrome. */
    public static boolean isPalindrome(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_is_palindrome(ps)) != 0; }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Count tokens in {@code s} split by byte delimiter {@code delim} (e.g. ',' = 44). */
    public static long tokenCount(String s, long delim) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_token_count(ps, delim)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Count whitespace-separated words in {@code s}. */
    public static long wordCount(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_word_count(ps)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Byte length of {@code s}. */
    public static long len(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_text_len(ps)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Byte value at {@code idx} in {@code s}, or -1 if out of bounds. */
    public static long charAt(String s, long idx) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return safe(() -> LIB.bmb_str_char_at(ps, idx)); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Lexicographic comparison: negative/0/positive (strcmp semantics). */
    public static long compare(String a, String b) {
        Pointer pa = LIB.bmb_ffi_cstr_to_string(a);
        Pointer pb = LIB.bmb_ffi_cstr_to_string(b);
        try { return safe(() -> LIB.bmb_str_compare(pa, pb)); }
        finally { LIB.bmb_ffi_free_string(pa); LIB.bmb_ffi_free_string(pb); }
    }

    // ── Transformations ────────────────────────────────────────────────────────

    /** Reverse the bytes of {@code s}. */
    public static String reverse(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return stringResult(safe(() -> LIB.bmb_str_reverse(ps))); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Replace first occurrence of {@code oldPat} with {@code newPat} in {@code s}. */
    public static String replace(String s, String oldPat, String newPat) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        Pointer po = LIB.bmb_ffi_cstr_to_string(oldPat);
        Pointer pn = LIB.bmb_ffi_cstr_to_string(newPat);
        try { return stringResult(safe(() -> LIB.bmb_str_replace(ps, po, pn))); }
        finally {
            LIB.bmb_ffi_free_string(ps);
            LIB.bmb_ffi_free_string(po);
            LIB.bmb_ffi_free_string(pn);
        }
    }

    /** Replace all occurrences of {@code oldPat} with {@code newPat} in {@code s}. */
    public static String replaceAll(String s, String oldPat, String newPat) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        Pointer po = LIB.bmb_ffi_cstr_to_string(oldPat);
        Pointer pn = LIB.bmb_ffi_cstr_to_string(newPat);
        try { return stringResult(safe(() -> LIB.bmb_str_replace_all(ps, po, pn))); }
        finally {
            LIB.bmb_ffi_free_string(ps);
            LIB.bmb_ffi_free_string(po);
            LIB.bmb_ffi_free_string(pn);
        }
    }

    /** Convert {@code s} to uppercase (ASCII only). */
    public static String toUpper(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return stringResult(safe(() -> LIB.bmb_str_to_upper(ps))); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Convert {@code s} to lowercase (ASCII only). */
    public static String toLower(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return stringResult(safe(() -> LIB.bmb_str_to_lower(ps))); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Trim leading and trailing ASCII whitespace from {@code s}. */
    public static String trim(String s) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return stringResult(safe(() -> LIB.bmb_str_trim(ps))); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }

    /** Repeat {@code s} {@code n} times. */
    public static String repeat(String s, long n) {
        Pointer ps = LIB.bmb_ffi_cstr_to_string(s);
        try { return stringResult(safe(() -> LIB.bmb_str_repeat(ps, n))); }
        finally { LIB.bmb_ffi_free_string(ps); }
    }
}

using System.Runtime.InteropServices;

namespace BmbTextLib;

/// <summary>
/// High-performance text search and transform functions powered by BMB — P/Invoke bindings.
/// All functions call into the native bmb_text shared library.
/// </summary>
public static class Text
{
    private const string LibName = "bmb_text";

    // ── FFI Safety API ───────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern int    bmb_ffi_begin();
    [DllImport(LibName)] private static extern void   bmb_ffi_end();
    [DllImport(LibName)] private static extern int    bmb_ffi_has_error();
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_error_message();

    // ── String FFI API ───────────────────────────────────────────────────────
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_cstr_to_string([MarshalAs(UnmanagedType.LPStr)] string s);
    [DllImport(LibName)] private static extern IntPtr bmb_ffi_string_data(IntPtr s);
    [DllImport(LibName)] private static extern long   bmb_ffi_string_len(IntPtr s);
    [DllImport(LibName)] private static extern void   bmb_ffi_free_string(IntPtr s);

    // ── Search functions (return int64) ──────────────────────────────────────
    [DllImport(LibName)] private static extern long bmb_kmp_search(IntPtr text, IntPtr pattern);
    [DllImport(LibName)] private static extern long bmb_str_find(IntPtr haystack, IntPtr needle);
    [DllImport(LibName)] private static extern long bmb_str_rfind(IntPtr haystack, IntPtr needle);
    [DllImport(LibName)] private static extern long bmb_str_count(IntPtr haystack, IntPtr needle);
    [DllImport(LibName)] private static extern long bmb_str_contains(IntPtr haystack, IntPtr needle);
    [DllImport(LibName)] private static extern long bmb_str_starts_with(IntPtr s, IntPtr prefix);
    [DllImport(LibName)] private static extern long bmb_str_ends_with(IntPtr s, IntPtr suffix);
    [DllImport(LibName)] private static extern long bmb_str_find_byte(IntPtr s, long needle);
    [DllImport(LibName)] private static extern long bmb_str_count_byte(IntPtr s, long needle);
    [DllImport(LibName)] private static extern long bmb_is_palindrome(IntPtr s);
    [DllImport(LibName)] private static extern long bmb_token_count(IntPtr s, long delim);
    [DllImport(LibName)] private static extern long bmb_str_hamming(IntPtr a, IntPtr b);
    [DllImport(LibName)] private static extern long bmb_word_count(IntPtr s);
    [DllImport(LibName)] private static extern long bmb_text_len(IntPtr s);

    // ── Transform functions (return BMB string) ──────────────────────────────
    [DllImport(LibName)] private static extern IntPtr bmb_str_reverse(IntPtr s);
    [DllImport(LibName)] private static extern IntPtr bmb_str_replace(IntPtr s, IntPtr old_pat, IntPtr new_pat);
    [DllImport(LibName)] private static extern IntPtr bmb_str_replace_all(IntPtr s, IntPtr old_pat, IntPtr new_pat);
    [DllImport(LibName)] private static extern IntPtr bmb_str_to_upper(IntPtr s);
    [DllImport(LibName)] private static extern IntPtr bmb_str_to_lower(IntPtr s);
    [DllImport(LibName)] private static extern IntPtr bmb_str_trim(IntPtr s);
    [DllImport(LibName)] private static extern IntPtr bmb_str_repeat(IntPtr s, long n);

    // ── Helpers ──────────────────────────────────────────────────────────────

    private static T Safe<T>(Func<T> fn)
    {
        bmb_ffi_begin();
        try
        {
            T r = fn();
            if (bmb_ffi_has_error() != 0)
                throw new InvalidOperationException(
                    Marshal.PtrToStringAnsi(bmb_ffi_error_message()) ?? "BMB FFI error");
            return r;
        }
        finally
        {
            bmb_ffi_end();
        }
    }

    private static T WithBmbString<T>(string s, Func<IntPtr, T> fn)
    {
        IntPtr p = bmb_ffi_cstr_to_string(s);
        try { return fn(p); }
        finally { bmb_ffi_free_string(p); }
    }

    private static T WithBmbStrings<T>(string a, string b, Func<IntPtr, IntPtr, T> fn)
    {
        IntPtr pa = bmb_ffi_cstr_to_string(a);
        IntPtr pb = bmb_ffi_cstr_to_string(b);
        try { return fn(pa, pb); }
        finally { bmb_ffi_free_string(pa); bmb_ffi_free_string(pb); }
    }

    private static T WithBmbStrings3<T>(string a, string b, string c, Func<IntPtr, IntPtr, IntPtr, T> fn)
    {
        IntPtr pa = bmb_ffi_cstr_to_string(a);
        IntPtr pb = bmb_ffi_cstr_to_string(b);
        IntPtr pc = bmb_ffi_cstr_to_string(c);
        try { return fn(pa, pb, pc); }
        finally { bmb_ffi_free_string(pa); bmb_ffi_free_string(pb); bmb_ffi_free_string(pc); }
    }

    // BMB function results are arena-allocated (bmb_alloc), not malloc.
    // Read the data only — do NOT call bmb_ffi_free_string on the return value.
    // Inputs from bmb_ffi_cstr_to_string (malloc) are freed by WithBmbString(s).
    private static string BmbStringToCS(IntPtr p)
    {
        if (p == IntPtr.Zero) return "";
        return Marshal.PtrToStringAnsi(bmb_ffi_string_data(p)) ?? "";
    }

    // ── Public API — Search ──────────────────────────────────────────────────

    /// <summary>KMP search: returns the first index of <paramref name="pattern"/> in <paramref name="text"/>, or -1.</summary>
    public static long KmpSearch(string text, string pattern) =>
        WithBmbStrings(text, pattern, (pt, pp) => Safe(() => bmb_kmp_search(pt, pp)));

    /// <summary>Returns the index of the first occurrence of <paramref name="needle"/> in <paramref name="haystack"/>, or -1.</summary>
    public static long StrFind(string haystack, string needle) =>
        WithBmbStrings(haystack, needle, (ph, pn) => Safe(() => bmb_str_find(ph, pn)));

    /// <summary>Returns the index of the last occurrence of <paramref name="needle"/> in <paramref name="haystack"/>, or -1.</summary>
    public static long StrRfind(string haystack, string needle) =>
        WithBmbStrings(haystack, needle, (ph, pn) => Safe(() => bmb_str_rfind(ph, pn)));

    /// <summary>Counts non-overlapping occurrences of <paramref name="needle"/> in <paramref name="haystack"/>.</summary>
    public static long StrCount(string haystack, string needle) =>
        WithBmbStrings(haystack, needle, (ph, pn) => Safe(() => bmb_str_count(ph, pn)));

    /// <summary>Returns true if <paramref name="haystack"/> contains <paramref name="needle"/>.</summary>
    public static bool StrContains(string haystack, string needle) =>
        WithBmbStrings(haystack, needle, (ph, pn) => Safe(() => bmb_str_contains(ph, pn))) != 0;

    /// <summary>Returns true if <paramref name="s"/> starts with <paramref name="prefix"/>.</summary>
    public static bool StrStartsWith(string s, string prefix) =>
        WithBmbStrings(s, prefix, (ps, pp) => Safe(() => bmb_str_starts_with(ps, pp))) != 0;

    /// <summary>Returns true if <paramref name="s"/> ends with <paramref name="suffix"/>.</summary>
    public static bool StrEndsWith(string s, string suffix) =>
        WithBmbStrings(s, suffix, (ps, ps2) => Safe(() => bmb_str_ends_with(ps, ps2))) != 0;

    /// <summary>Returns the index of the first byte equal to <paramref name="needle"/> in <paramref name="s"/>, or -1.</summary>
    public static long StrFindByte(string s, byte needle) =>
        WithBmbString(s, p => Safe(() => bmb_str_find_byte(p, needle)));

    /// <summary>Counts occurrences of byte <paramref name="needle"/> in <paramref name="s"/>.</summary>
    public static long StrCountByte(string s, byte needle) =>
        WithBmbString(s, p => Safe(() => bmb_str_count_byte(p, needle)));

    /// <summary>Returns true if <paramref name="s"/> is a palindrome.</summary>
    public static bool IsPalindrome(string s) =>
        WithBmbString(s, p => Safe(() => bmb_is_palindrome(p))) != 0;

    /// <summary>Counts tokens in <paramref name="s"/> split by delimiter byte <paramref name="delim"/>.</summary>
    public static long TokenCount(string s, byte delim) =>
        WithBmbString(s, p => Safe(() => bmb_token_count(p, delim)));

    /// <summary>Hamming distance between <paramref name="a"/> and <paramref name="b"/> (must be equal length).</summary>
    public static long StrHamming(string a, string b) =>
        WithBmbStrings(a, b, (pa, pb) => Safe(() => bmb_str_hamming(pa, pb)));

    /// <summary>Count whitespace-delimited words in <paramref name="s"/>.</summary>
    public static long WordCount(string s) =>
        WithBmbString(s, p => Safe(() => bmb_word_count(p)));

    /// <summary>Byte length of <paramref name="s"/>.</summary>
    public static long TextLen(string s) =>
        WithBmbString(s, p => Safe(() => bmb_text_len(p)));

    // ── Public API — Transform ───────────────────────────────────────────────

    /// <summary>Reverse the bytes of <paramref name="s"/>.</summary>
    public static string StrReverse(string s) =>
        WithBmbString(s, p => Safe(() => BmbStringToCS(bmb_str_reverse(p))));

    /// <summary>Replace the first occurrence of <paramref name="oldPat"/> with <paramref name="newPat"/> in <paramref name="s"/>.</summary>
    public static string StrReplace(string s, string oldPat, string newPat) =>
        WithBmbStrings3(s, oldPat, newPat, (ps, po, pn) => Safe(() => BmbStringToCS(bmb_str_replace(ps, po, pn))));

    /// <summary>Replace all occurrences of <paramref name="oldPat"/> with <paramref name="newPat"/> in <paramref name="s"/>.</summary>
    public static string StrReplaceAll(string s, string oldPat, string newPat) =>
        WithBmbStrings3(s, oldPat, newPat, (ps, po, pn) => Safe(() => BmbStringToCS(bmb_str_replace_all(ps, po, pn))));

    /// <summary>Convert <paramref name="s"/> to upper-case.</summary>
    public static string StrToUpper(string s) =>
        WithBmbString(s, p => Safe(() => BmbStringToCS(bmb_str_to_upper(p))));

    /// <summary>Convert <paramref name="s"/> to lower-case.</summary>
    public static string StrToLower(string s) =>
        WithBmbString(s, p => Safe(() => BmbStringToCS(bmb_str_to_lower(p))));

    /// <summary>Trim leading and trailing whitespace from <paramref name="s"/>.</summary>
    public static string StrTrim(string s) =>
        WithBmbString(s, p => Safe(() => BmbStringToCS(bmb_str_trim(p))));

    /// <summary>Repeat <paramref name="s"/> exactly <paramref name="n"/> times.</summary>
    public static string StrRepeat(string s, long n) =>
        WithBmbString(s, p => Safe(() => BmbStringToCS(bmb_str_repeat(p, n))));
}

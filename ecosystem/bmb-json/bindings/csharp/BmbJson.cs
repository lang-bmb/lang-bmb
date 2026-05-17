using System.Runtime.InteropServices;

namespace BmbJsonLib;

/// <summary>
/// JSON parsing powered by BMB — P/Invoke bindings.
/// All functions call into the native bmb_json shared library.
/// Functions returning BMB strings automatically convert to C# strings (arena memory — not individually freed).
/// </summary>
public static class Json
{
    private const string LibName = "bmb_json";

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

    // ── JSON API (raw P/Invoke) ───────────────────────────────────────────────
    [DllImport(LibName)] private static extern long   bmb_json_validate(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_json_stringify(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_json_type(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_json_get(IntPtr input, IntPtr key);
    [DllImport(LibName)] private static extern IntPtr bmb_json_get_string(IntPtr input, IntPtr key);
    [DllImport(LibName)] private static extern long   bmb_json_get_number(IntPtr input, IntPtr key);
    [DllImport(LibName)] private static extern long   bmb_json_array_len(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_json_array_get(IntPtr input, long idx);
    [DllImport(LibName)] private static extern long   bmb_json_has_key(IntPtr input, IntPtr key);
    [DllImport(LibName)] private static extern long   bmb_json_object_len(IntPtr input);
    [DllImport(LibName)] private static extern long   bmb_json_get_bool(IntPtr input, IntPtr key);
    [DllImport(LibName)] private static extern long   bmb_json_count(IntPtr input);

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

    // BMB function results are arena-allocated (bmb_alloc), not malloc.
    // Read the data only — do NOT call bmb_ffi_free_string on the return value.
    // Inputs from bmb_ffi_cstr_to_string (malloc) are freed by WithBmbString(s).
    private static string BmbStringToCS(IntPtr p)
    {
        if (p == IntPtr.Zero) return "";
        return Marshal.PtrToStringAnsi(bmb_ffi_string_data(p)) ?? "";
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// <summary>
    /// Validate JSON: parse + serialize roundtrip.
    /// Returns true if valid, false otherwise.
    /// </summary>
    public static bool Validate(string input) =>
        WithBmbString(input, p => Safe(() => bmb_json_validate(p))) != 0;

    /// <summary>
    /// Parse and re-serialize JSON (roundtrip normalization).
    /// Returns normalized JSON string.
    /// </summary>
    public static string Stringify(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_json_stringify(p))));

    /// <summary>
    /// Get the JSON value type: "null", "bool", "number", "string", "array", or "object".
    /// </summary>
    public static string Type(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_json_type(p))));

    /// <summary>
    /// Get an object value by key. Returns the JSON string representation of the value.
    /// </summary>
    public static string Get(string input, string key) =>
        WithBmbStrings(input, key, (pi, pk) => Safe(() => BmbStringToCS(bmb_json_get(pi, pk))));

    /// <summary>
    /// Get a string value from an object by key. Returns the raw string (no quotes).
    /// </summary>
    public static string GetString(string input, string key) =>
        WithBmbStrings(input, key, (pi, pk) => Safe(() => BmbStringToCS(bmb_json_get_string(pi, pk))));

    /// <summary>
    /// Get a number value from an object by key. Returns 0 if not found.
    /// </summary>
    public static long GetNumber(string input, string key) =>
        WithBmbStrings(input, key, (pi, pk) => Safe(() => bmb_json_get_number(pi, pk)));

    /// <summary>
    /// Get the length of a JSON array. Returns -1 if input is not an array.
    /// </summary>
    public static long ArrayLen(string input) =>
        WithBmbString(input, p => Safe(() => bmb_json_array_len(p)));

    /// <summary>
    /// Get the element at index idx from a JSON array. Returns the JSON string representation.
    /// </summary>
    public static string ArrayGet(string input, long idx) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_json_array_get(p, idx))));

    /// <summary>
    /// Check if a JSON object has a given key. Returns true if found.
    /// </summary>
    public static bool HasKey(string input, string key) =>
        WithBmbStrings(input, key, (pi, pk) => Safe(() => bmb_json_has_key(pi, pk))) != 0;

    /// <summary>
    /// Get the number of keys in a JSON object. Returns -1 for non-objects.
    /// </summary>
    public static long ObjectLen(string input) =>
        WithBmbString(input, p => Safe(() => bmb_json_object_len(p)));

    /// <summary>
    /// Get a boolean value from an object by key.
    /// Returns 1 (true), 0 (false), or -1 (key missing / not a bool).
    /// </summary>
    public static long GetBool(string input, string key) =>
        WithBmbStrings(input, key, (pi, pk) => Safe(() => bmb_json_get_bool(pi, pk)));

    /// <summary>
    /// Count total number of elements in a JSON structure (recursive).
    /// </summary>
    public static long Count(string input) =>
        WithBmbString(input, p => Safe(() => bmb_json_count(p)));
}

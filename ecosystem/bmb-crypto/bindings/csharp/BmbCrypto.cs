using System.Runtime.InteropServices;

namespace BmbCryptoLib;

/// <summary>
/// Cryptographic hashing and encoding functions powered by BMB — P/Invoke bindings.
/// All functions call into the native bmb_crypto shared library.
/// </summary>
public static class Crypto
{
    private const string LibName = "bmb_crypto";

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

    // ── Crypto / Encoding functions ──────────────────────────────────────────
    [DllImport(LibName)] private static extern IntPtr bmb_sha256(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_base64_encode(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_base64_decode(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_md5(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_crc32(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_base32_encode(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_base32_decode(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_hmac_sha256(IntPtr key, IntPtr msg);
    [DllImport(LibName)] private static extern IntPtr bmb_adler32(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_fletcher16(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_xor_checksum(IntPtr input);
    [DllImport(LibName)] private static extern IntPtr bmb_rot13(IntPtr input);

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

    private static string BmbStringToCS(IntPtr p)
    {
        if (p == IntPtr.Zero) return "";
        string r = Marshal.PtrToStringAnsi(bmb_ffi_string_data(p)) ?? "";
        bmb_ffi_free_string(p);
        return r;
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// <summary>SHA-256 hash of the input string, returned as a hex string.</summary>
    public static string Sha256(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_sha256(p))));

    /// <summary>Base64-encode the input string.</summary>
    public static string Base64Encode(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_base64_encode(p))));

    /// <summary>Base64-decode the input string.</summary>
    public static string Base64Decode(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_base64_decode(p))));

    /// <summary>MD5 hash of the input string, returned as a hex string.</summary>
    public static string Md5(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_md5(p))));

    /// <summary>CRC32 checksum of the input string, returned as a hex string.</summary>
    public static string Crc32(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_crc32(p))));

    /// <summary>Base32-encode the input string.</summary>
    public static string Base32Encode(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_base32_encode(p))));

    /// <summary>Base32-decode the input string.</summary>
    public static string Base32Decode(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_base32_decode(p))));

    /// <summary>HMAC-SHA256 of <paramref name="msg"/> using <paramref name="key"/>, returned as a hex string.</summary>
    public static string HmacSha256(string key, string msg) =>
        WithBmbStrings(key, msg, (pk, pm) => Safe(() => BmbStringToCS(bmb_hmac_sha256(pk, pm))));

    /// <summary>Adler-32 checksum of the input string, returned as a hex string.</summary>
    public static string Adler32(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_adler32(p))));

    /// <summary>Fletcher-16 checksum of the input string, returned as a hex string.</summary>
    public static string Fletcher16(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_fletcher16(p))));

    /// <summary>XOR checksum of the input string, returned as a hex string.</summary>
    public static string XorChecksum(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_xor_checksum(p))));

    /// <summary>ROT13 transform of the input string.</summary>
    public static string Rot13(string input) =>
        WithBmbString(input, p => Safe(() => BmbStringToCS(bmb_rot13(p))));
}

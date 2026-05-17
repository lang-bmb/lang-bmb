package io.bmb.json;

import com.sun.jna.Pointer;

import java.util.function.Supplier;

/**
 * JSON processing powered by BMB — JNA bindings.
 * All calls dispatch into the native bmb_json shared library.
 *
 * Thread safety: each public method wraps calls in bmb_ffi_begin/end.
 */
public final class BmbJson {
    private static final BmbJsonLib LIB = BmbJsonLib.INSTANCE;

    private BmbJson() {}

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

    // ── Validation ─────────────────────────────────────────────────────────────

    /** Returns true if {@code json} is syntactically valid JSON. */
    public static boolean validate(String json) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return safe(() -> LIB.bmb_json_validate(pJson)) != 0; }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }

    // ── Serialization ──────────────────────────────────────────────────────────

    /** Parse and re-serialize (roundtrip normalization). */
    public static String stringify(String json) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return stringResult(safe(() -> LIB.bmb_json_stringify(pJson))); }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }

    // ── Type inspection ────────────────────────────────────────────────────────

    /** Returns the JSON type: "null", "bool", "number", "string", "array", "object". */
    public static String type(String json) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return stringResult(safe(() -> LIB.bmb_json_type(pJson))); }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }

    /** Count total top-level elements (object key count or array length + 1). */
    public static long count(String json) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return safe(() -> LIB.bmb_json_count(pJson)); }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }

    // ── Object access ──────────────────────────────────────────────────────────

    /** Get the JSON value for {@code key} as a JSON string, or {@code ""} if absent. */
    public static String get(String json, String key) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        Pointer pKey  = LIB.bmb_ffi_cstr_to_string(key);
        try { return stringResult(safe(() -> LIB.bmb_json_get(pJson, pKey))); }
        finally { LIB.bmb_ffi_free_string(pJson); LIB.bmb_ffi_free_string(pKey); }
    }

    /** Get a string value for {@code key} (quotes stripped), or {@code ""} if absent/non-string. */
    public static String getString(String json, String key) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        Pointer pKey  = LIB.bmb_ffi_cstr_to_string(key);
        try { return stringResult(safe(() -> LIB.bmb_json_get_string(pJson, pKey))); }
        finally { LIB.bmb_ffi_free_string(pJson); LIB.bmb_ffi_free_string(pKey); }
    }

    /** Get a numeric value for {@code key} as i64, or 0 if absent/non-number. */
    public static long getNumber(String json, String key) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        Pointer pKey  = LIB.bmb_ffi_cstr_to_string(key);
        try { return safe(() -> LIB.bmb_json_get_number(pJson, pKey)); }
        finally { LIB.bmb_ffi_free_string(pJson); LIB.bmb_ffi_free_string(pKey); }
    }

    /** Get a bool value for {@code key}: 1=true, 0=false, -1=missing/non-bool. */
    public static long getBool(String json, String key) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        Pointer pKey  = LIB.bmb_ffi_cstr_to_string(key);
        try { return safe(() -> LIB.bmb_json_get_bool(pJson, pKey)); }
        finally { LIB.bmb_ffi_free_string(pJson); LIB.bmb_ffi_free_string(pKey); }
    }

    /** Returns true if the JSON object contains {@code key}. */
    public static boolean hasKey(String json, String key) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        Pointer pKey  = LIB.bmb_ffi_cstr_to_string(key);
        try { return safe(() -> LIB.bmb_json_has_key(pJson, pKey)) != 0; }
        finally { LIB.bmb_ffi_free_string(pJson); LIB.bmb_ffi_free_string(pKey); }
    }

    /** Returns the number of keys in a JSON object, or -1 if not an object. */
    public static long objectLen(String json) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return safe(() -> LIB.bmb_json_object_len(pJson)); }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }

    // ── Array access ───────────────────────────────────────────────────────────

    /** Returns the length of a JSON array, or -1 if not an array. */
    public static long arrayLen(String json) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return safe(() -> LIB.bmb_json_array_len(pJson)); }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }

    /** Get array element at {@code idx} as a JSON string, or {@code ""} if out of bounds. */
    public static String arrayGet(String json, long idx) {
        Pointer pJson = LIB.bmb_ffi_cstr_to_string(json);
        try { return stringResult(safe(() -> LIB.bmb_json_array_get(pJson, idx))); }
        finally { LIB.bmb_ffi_free_string(pJson); }
    }
}

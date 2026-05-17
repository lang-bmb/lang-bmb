package io.bmb.json;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/** Raw JNA interface — direct 1:1 mapping of the bmb_json native C ABI. */
interface BmbJsonLib extends Library {
    BmbJsonLib INSTANCE = Native.load("bmb_json", BmbJsonLib.class);

    // FFI Safety API
    int     bmb_ffi_begin();
    void    bmb_ffi_end();
    int     bmb_ffi_has_error();
    Pointer bmb_ffi_error_message();

    // String FFI API — BmbString* is opaque from Java
    Pointer bmb_ffi_cstr_to_string(String s);
    Pointer bmb_ffi_string_data(Pointer s);
    void    bmb_ffi_free_string(Pointer s);

    // Validation
    long    bmb_json_validate(Pointer input);

    // Serialization
    Pointer bmb_json_stringify(Pointer input);

    // Type inspection
    Pointer bmb_json_type(Pointer input);
    long    bmb_json_count(Pointer input);

    // Object access
    Pointer bmb_json_get(Pointer input, Pointer key);
    Pointer bmb_json_get_string(Pointer input, Pointer key);
    long    bmb_json_get_number(Pointer input, Pointer key);
    long    bmb_json_get_bool(Pointer input, Pointer key);
    long    bmb_json_has_key(Pointer input, Pointer key);
    long    bmb_json_object_len(Pointer input);

    // Array access
    long    bmb_json_array_len(Pointer input);
    Pointer bmb_json_array_get(Pointer input, long idx);
}

package io.bmb.crypto;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;

/** Raw JNA interface — direct 1:1 mapping of the bmb_crypto native C ABI. */
interface BmbCryptoLib extends Library {
    BmbCryptoLib INSTANCE = Native.load("bmb_crypto", BmbCryptoLib.class);

    // FFI Safety API
    int     bmb_ffi_begin();
    void    bmb_ffi_end();
    int     bmb_ffi_has_error();
    Pointer bmb_ffi_error_message();

    // String FFI API — BmbString* is opaque from Java
    Pointer bmb_ffi_cstr_to_string(String s);
    Pointer bmb_ffi_string_data(Pointer s);
    void    bmb_ffi_free_string(Pointer s);

    // Hashing
    Pointer bmb_sha256(Pointer input);
    Pointer bmb_md5(Pointer input);
    Pointer bmb_crc32(Pointer input);
    Pointer bmb_adler32(Pointer input);
    Pointer bmb_fletcher16(Pointer input);
    Pointer bmb_xor_checksum(Pointer input);
    Pointer bmb_hmac_sha256(Pointer key, Pointer msg);

    // Encoding / Decoding
    Pointer bmb_base64_encode(Pointer input);
    Pointer bmb_base64_decode(Pointer input);
    Pointer bmb_base32_encode(Pointer input);
    Pointer bmb_base32_decode(Pointer input);
    Pointer bmb_hex_encode(Pointer input);
    Pointer bmb_hex_decode(Pointer input);

    // Cipher
    Pointer bmb_rot13(Pointer input);
}

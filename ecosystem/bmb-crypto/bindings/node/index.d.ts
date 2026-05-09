/**
 * bmb-crypto: Cryptographic functions powered by BMB
 * Node.js FFI bindings via koffi.
 */

/** SHA-256 hash as lowercase hex string. */
export declare function sha256(s: string): string;

/** MD5 hash as lowercase hex string. */
export declare function md5(s: string): string;

/** CRC-32 as hex string. */
export declare function crc32(s: string): string;

/** Base64 encode. */
export declare function base64_encode(s: string): string;

/** Base64 decode. */
export declare function base64_decode(s: string): string;

/** Base32 encode. */
export declare function base32_encode(s: string): string;

/** Base32 decode. */
export declare function base32_decode(s: string): string;

/** HMAC-SHA256 as lowercase hex string. */
export declare function hmac_sha256(key: string, msg: string): string;

/** Adler-32 checksum as hex string. */
export declare function adler32(s: string): string;

/** Fletcher-16 checksum as hex string. */
export declare function fletcher16(s: string): string;

/** XOR checksum (single byte) as hex string. */
export declare function xor_checksum(s: string): string;

/** ROT-13 transform. */
export declare function rot13(s: string): string;

/** Hex encode (each byte as two hex digits). */
export declare function hex_encode(s: string): string;

/** Hex decode. */
export declare function hex_decode(s: string): string;

/**
 * bmb-json: High-performance JSON processing powered by BMB
 * Node.js FFI bindings via koffi.
 *
 * Note: All functions accept JSON as a string (not a parsed JS object).
 * Outputs that are strings return library-owned memory — do NOT free them.
 */

/** Validate JSON string. Returns true if valid. */
export declare function validate(json: string): boolean;

/** Get the type of the top-level JSON value: "object", "array", "string", "number", "bool", "null", or "unknown". */
export declare function get_type(json: string): string;

/** Compact-stringify a JSON value (removes whitespace). */
export declare function stringify(json: string): string;

/** Array length. Returns 0 for non-arrays. */
export declare function array_len(json: string): number;

/** Object key count. Returns 0 for non-objects. */
export declare function object_len(json: string): number;

/** Count of all values in the JSON tree. */
export declare function count(json: string): number;

/** Get number value at key in JSON object. */
export declare function get_number(json: string, key: string): number;

/** True if JSON object has key. */
export declare function has_key(json: string, key: string): boolean;

/** Get boolean value at key (returns 0 or 1 as number). */
export declare function get_bool(json: string, key: string): number;

/** Get the raw JSON string at key in a JSON object. */
export declare function get(json: string, key: string): string;

/** Get string value at key in JSON object. */
export declare function get_string(json: string, key: string): string;

/** Get element at index from JSON array as raw JSON string. */
export declare function array_get(json: string, idx: number): string;

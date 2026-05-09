/**
 * bmb-text: High-performance string processing powered by BMB
 * Node.js FFI bindings via koffi.
 */

/** KMP pattern search. Returns first match index or -1. */
export declare function kmp_search(text: string, pattern: string): number;

/** Find first occurrence of pattern in s. Returns index or -1. */
export declare function str_find(s: string, pattern: string): number;

/** Find last occurrence of pattern in s. Returns index or -1. */
export declare function str_rfind(s: string, pattern: string): number;

/** Count occurrences of pattern in s. */
export declare function str_count(s: string, pattern: string): number;

/** True if s contains pattern. */
export declare function str_contains(s: string, pattern: string): boolean;

/** True if s starts with prefix. */
export declare function str_starts_with(s: string, prefix: string): boolean;

/** True if s ends with suffix. */
export declare function str_ends_with(s: string, suffix: string): boolean;

/** Hamming distance between two strings of equal length. */
export declare function hamming_distance(a: string, b: string): number;

/** Lexicographic compare: -1, 0, or 1. */
export declare function str_compare(a: string, b: string): number;

/** Find first occurrence of byte value b (0-255) in s. Returns index or -1. */
export declare function find_byte(s: string, b: number): number;

/** Count occurrences of byte value b (0-255) in s. */
export declare function count_byte(s: string, b: number): number;

/** Count tokens in s separated by sep. */
export declare function token_count(s: string, sep: string): number;

/** Character (byte) at index i. Returns -1 if out of range. */
export declare function str_char_at(s: string, i: number): number;

/** True if s is a palindrome. */
export declare function is_palindrome(s: string): boolean;

/** Word count (whitespace-separated). */
export declare function word_count(s: string): number;

/** Byte length of s. */
export declare function str_len(s: string): number;

/** Reverse s. */
export declare function str_reverse(s: string): string;

/** Convert s to uppercase. */
export declare function to_upper(s: string): string;

/** Convert s to lowercase. */
export declare function to_lower(s: string): string;

/** Trim leading and trailing whitespace. */
export declare function trim(s: string): string;

/** Replace first occurrence of `from` with `to` in s. */
export declare function str_replace(s: string, from: string, to: string): string;

/** Replace all occurrences of `from` with `to` in s. */
export declare function str_replace_all(s: string, from: string, to: string): string;

/** Repeat s n times. */
export declare function repeat(s: string, n: number): string;

// v0.51.46: Windows compatibility fixes
#ifdef _WIN32
#define NOMINMAX  // Prevent min/max macros
#define _CRT_SECURE_NO_WARNINGS  // Suppress fopen/scanf warnings
#endif

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>  // v0.90.90: memcpy/memset/strlen for optimized string operations
#include <math.h>    // v0.90.95: Float math functions (floor, ceil, round, sqrt, fabs, isnan)

// v0.70: Threading support
#ifdef _WIN32
#include <winsock2.h>  // v0.88: Must be before windows.h
#include <ws2tcpip.h>
#include <windows.h>
#else
#include <pthread.h>
#include <errno.h>   // v0.77: For ETIMEDOUT
#include <time.h>    // v0.77: For clock_gettime
#endif

// v0.98: Event loop for async I/O
#include "bmb_event_loop.h"

// Forward declaration for event loop singleton (defined in v0.99 section)
BmbEventLoop* bmb_get_event_loop(void);

// BMB Runtime Library

// v0.51.51: BmbString struct for type-safe string handling
// Matches LLVM IR: %BmbString = type { ptr, i64, i64 }
typedef struct {
    char* data;      // pointer to null-terminated string data
    int64_t len;     // string length (excluding null terminator)
    int64_t cap;     // capacity (excluding null terminator)
} BmbString;

// v0.88.2: Forward declarations for arena allocator
static void* bmb_alloc(size_t size);
static int g_arena_enabled;

// Helper to create a new BmbString from raw char*
static BmbString* bmb_string_wrap(char* data) {
    if (!data) {
        data = (char*)bmb_alloc(1);
        data[0] = '\0';
    }
    BmbString* s = (BmbString*)bmb_alloc(sizeof(BmbString));
    int64_t len = (int64_t)strlen(data);
    s->data = data;
    s->len = len;
    s->cap = len;
    return s;
}

void bmb_println_i64(int64_t n) { printf("%" PRId64 "\n", n); }
void bmb_print_i64(int64_t n) { printf("%" PRId64, n); }
// v0.60.43: Float output for spectral_norm, n_body benchmarks
void bmb_println_f64(double f) { printf("%.9f\n", f); }
void bmb_print_f64(double f) { printf("%.9f", f); }
int64_t bmb_read_int() { int64_t n; scanf("%" SCNd64, &n); return n; }
void bmb_assert(int cond) { if (!cond) { fprintf(stderr, "Assertion failed!\n"); exit(1); } }
int64_t bmb_abs(int64_t n) { return n < 0 ? -n : n; }
int64_t bmb_min(int64_t a, int64_t b) { return a < b ? a : b; }
int64_t bmb_max(int64_t a, int64_t b) { return a > b ? a : b; }
double bmb_i64_to_f64(int64_t n) { return (double)n; }
int64_t bmb_f64_to_i64(double f) { return (int64_t)f; }

// v0.90.95: Float method support — floor, ceil, round, sqrt, abs, is_nan, min, max, to_int
double bmb_f64_floor(double f) { return floor(f); }
double bmb_f64_ceil(double f) { return ceil(f); }
double bmb_f64_round(double f) { return round(f); }
double bmb_f64_sqrt(double f) { return sqrt(f); }
double bmb_f64_abs(double f) { return fabs(f); }
int64_t bmb_f64_is_nan(double f) { return isnan(f) ? 1 : 0; }
double bmb_f64_min(double a, double b) { return a < b ? a : b; }
double bmb_f64_max(double a, double b) { return a > b ? a : b; }
int64_t bmb_f64_to_int(double f) { return (int64_t)f; }

// Forward declaration for bmb_string_slice (defined later, used by trim)
BmbString* bmb_string_slice(const BmbString* s, int64_t start, int64_t end);

// v0.90.96: String methods — starts_with, ends_with, contains, index_of, trim, replace
int64_t bmb_string_starts_with(BmbString* s, BmbString* prefix) {
    if (!s || !prefix) return 0;
    if (prefix->len > s->len) return 0;
    return memcmp(s->data, prefix->data, (size_t)prefix->len) == 0 ? 1 : 0;
}

int64_t bmb_string_ends_with(BmbString* s, BmbString* suffix) {
    if (!s || !suffix) return 0;
    if (suffix->len > s->len) return 0;
    return memcmp(s->data + s->len - suffix->len, suffix->data, (size_t)suffix->len) == 0 ? 1 : 0;
}

int64_t bmb_string_contains(BmbString* s, BmbString* needle) {
    if (!s || !needle) return 0;
    if (needle->len == 0) return 1;
    if (needle->len > s->len) return 0;
    int64_t limit = s->len - needle->len;
    for (int64_t i = 0; i <= limit; i++) {
        if (memcmp(s->data + i, needle->data, (size_t)needle->len) == 0) return 1;
    }
    return 0;
}

int64_t bmb_string_index_of(BmbString* s, BmbString* needle) {
    if (!s || !needle) return -1;
    if (needle->len == 0) return 0;
    if (needle->len > s->len) return -1;
    int64_t limit = s->len - needle->len;
    for (int64_t i = 0; i <= limit; i++) {
        if (memcmp(s->data + i, needle->data, (size_t)needle->len) == 0) return i;
    }
    return -1;
}

BmbString* bmb_string_trim(BmbString* s) {
    if (!s || s->len == 0) return s;
    int64_t start = 0;
    int64_t end = s->len;
    while (start < end && (s->data[start] == ' ' || s->data[start] == '\t' ||
           s->data[start] == '\n' || s->data[start] == '\r')) start++;
    while (end > start && (s->data[end-1] == ' ' || s->data[end-1] == '\t' ||
           s->data[end-1] == '\n' || s->data[end-1] == '\r')) end--;
    return bmb_string_slice(s, start, end);
}

BmbString* bmb_string_replace(BmbString* s, BmbString* old_str, BmbString* new_str) {
    if (!s || !old_str || !new_str || old_str->len == 0) return s;
    // Count occurrences first
    int64_t count = 0;
    int64_t limit = s->len - old_str->len;
    for (int64_t i = 0; i <= limit; i++) {
        if (memcmp(s->data + i, old_str->data, (size_t)old_str->len) == 0) {
            count++;
            i += old_str->len - 1;
        }
    }
    if (count == 0) return s;
    // Allocate result
    int64_t new_len = s->len + count * (new_str->len - old_str->len);
    char* data = (char*)bmb_alloc(new_len + 1);
    int64_t j = 0;
    for (int64_t i = 0; i < s->len; ) {
        if (i <= limit && memcmp(s->data + i, old_str->data, (size_t)old_str->len) == 0) {
            memcpy(data + j, new_str->data, (size_t)new_str->len);
            j += new_str->len;
            i += old_str->len;
        } else {
            data[j++] = s->data[i++];
        }
    }
    data[j] = '\0';
    return bmb_string_wrap(data);
}

// v0.90.98: String methods — to_upper, to_lower, repeat, is_empty
BmbString* bmb_string_to_upper(BmbString* s) {
    if (!s || s->len == 0) return s;
    char* data = (char*)bmb_alloc(s->len + 1);
    for (int64_t i = 0; i < s->len; i++) {
        char c = s->data[i];
        data[i] = (c >= 'a' && c <= 'z') ? c - 32 : c;
    }
    data[s->len] = '\0';
    return bmb_string_wrap(data);
}

BmbString* bmb_string_to_lower(BmbString* s) {
    if (!s || s->len == 0) return s;
    char* data = (char*)bmb_alloc(s->len + 1);
    for (int64_t i = 0; i < s->len; i++) {
        char c = s->data[i];
        data[i] = (c >= 'A' && c <= 'Z') ? c + 32 : c;
    }
    data[s->len] = '\0';
    return bmb_string_wrap(data);
}

BmbString* bmb_string_repeat(BmbString* s, int64_t n) {
    if (!s || n <= 0) {
        char* data = (char*)bmb_alloc(1);
        data[0] = '\0';
        return bmb_string_wrap(data);
    }
    int64_t total = s->len * n;
    char* data = (char*)bmb_alloc(total + 1);
    for (int64_t i = 0; i < n; i++) {
        memcpy(data + i * s->len, s->data, (size_t)s->len);
    }
    data[total] = '\0';
    return bmb_string_wrap(data);
}

int64_t bmb_string_is_empty(BmbString* s) {
    return (!s || s->len == 0) ? 1 : 0;
}

// v0.97: Character functions
// v0.60.107: bmb_chr returns BmbString* to match string type system
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_chr(int64_t n) {
    char* data = (char*)bmb_alloc(2);
    data[0] = (char)n;
    data[1] = '\0';
    return bmb_string_wrap(data);
}
// v0.46: bmb_ord takes ptr (char*) to match LLVM codegen expectations
int64_t bmb_ord(const char* s) {
    if (!s || s[0] == '\0') return 0;
    return (int64_t)(unsigned char)s[0];
}

// v0.97: String functions - updated to use BmbString* (v0.60.47)
void bmb_print_str(const BmbString* s) {
    if (s && s->data) {
        printf("%.*s", (int)s->len, s->data);
    }
}
void bmb_println_str(const BmbString* s) {
    if (s && s->data) {
        printf("%.*s\n", (int)s->len, s->data);
    } else {
        printf("\n");
    }
}
int64_t bmb_str_len(const char* s) { int64_t len = 0; while (s[len]) len++; return len; }

// v0.98: Vector functions
// Layout: ptr[0] = capacity, ptr[1] = length, ptr[2...] = data
int64_t bmb_vec_new() {
    int64_t* vec = (int64_t*)malloc(10 * sizeof(int64_t));
    vec[0] = 8;  // capacity
    vec[1] = 0;  // length
    return (int64_t)vec;
}

int64_t bmb_vec_with_capacity(int64_t cap) {
    int64_t* vec = (int64_t*)malloc((cap + 2) * sizeof(int64_t));
    vec[0] = cap;  // capacity
    vec[1] = 0;    // length
    return (int64_t)vec;
}

void bmb_vec_push(int64_t vec_ptr, int64_t value) {
    int64_t* vec = (int64_t*)vec_ptr;
    int64_t cap = vec[0];
    int64_t len = vec[1];
    if (len >= cap) {
        // Grow: double capacity
        int64_t new_cap = cap * 2;
        int64_t* new_vec = (int64_t*)realloc(vec, (new_cap + 2) * sizeof(int64_t));
        new_vec[0] = new_cap;
        vec = new_vec;
    }
    vec[2 + len] = value;
    vec[1] = len + 1;
}

int64_t bmb_vec_pop(int64_t vec_ptr) {
    int64_t* vec = (int64_t*)vec_ptr;
    int64_t len = vec[1];
    if (len == 0) return 0;  // Empty vector
    vec[1] = len - 1;
    return vec[2 + len - 1];
}

int64_t bmb_vec_get(int64_t vec_ptr, int64_t index) {
    int64_t* vec = (int64_t*)vec_ptr;
    return vec[2 + index];
}

void bmb_vec_set(int64_t vec_ptr, int64_t index, int64_t value) {
    int64_t* vec = (int64_t*)vec_ptr;
    vec[2 + index] = value;
}

int64_t bmb_vec_len(int64_t vec_ptr) {
    int64_t* vec = (int64_t*)vec_ptr;
    return vec[1];
}

int64_t bmb_vec_cap(int64_t vec_ptr) {
    int64_t* vec = (int64_t*)vec_ptr;
    return vec[0];
}

void bmb_vec_free(int64_t vec_ptr) {
    free((void*)vec_ptr);
}

void bmb_vec_clear(int64_t vec_ptr) {
    int64_t* vec = (int64_t*)vec_ptr;
    vec[1] = 0;  // Reset length
}

// v0.90.97: Functional array methods — push, pop, concat, slice, join return NEW arrays
// Array representation: [capacity, length, data[0], data[1], ...]
int64_t bmb_array_push(int64_t arr_ptr, int64_t value) {
    int64_t* arr = (int64_t*)arr_ptr;
    int64_t len = arr[1];
    int64_t new_cap = len + 1;
    int64_t* result = (int64_t*)bmb_alloc((new_cap + 2) * sizeof(int64_t));
    result[0] = new_cap;
    result[1] = len + 1;
    for (int64_t i = 0; i < len; i++) result[2 + i] = arr[2 + i];
    result[2 + len] = value;
    return (int64_t)result;
}

int64_t bmb_array_pop(int64_t arr_ptr) {
    int64_t* arr = (int64_t*)arr_ptr;
    int64_t len = arr[1];
    if (len == 0) return arr_ptr;
    int64_t new_len = len - 1;
    int64_t* result = (int64_t*)bmb_alloc((new_len + 2) * sizeof(int64_t));
    result[0] = new_len;
    result[1] = new_len;
    for (int64_t i = 0; i < new_len; i++) result[2 + i] = arr[2 + i];
    return (int64_t)result;
}

int64_t bmb_array_concat(int64_t arr1_ptr, int64_t arr2_ptr) {
    int64_t* a1 = (int64_t*)arr1_ptr;
    int64_t* a2 = (int64_t*)arr2_ptr;
    int64_t len1 = a1[1], len2 = a2[1];
    int64_t total = len1 + len2;
    int64_t* result = (int64_t*)bmb_alloc((total + 2) * sizeof(int64_t));
    result[0] = total;
    result[1] = total;
    for (int64_t i = 0; i < len1; i++) result[2 + i] = a1[2 + i];
    for (int64_t i = 0; i < len2; i++) result[2 + len1 + i] = a2[2 + i];
    return (int64_t)result;
}

int64_t bmb_array_slice(int64_t arr_ptr, int64_t start, int64_t end) {
    int64_t* arr = (int64_t*)arr_ptr;
    int64_t len = arr[1];
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (start >= end) {
        int64_t* result = (int64_t*)bmb_alloc(2 * sizeof(int64_t));
        result[0] = 0; result[1] = 0;
        return (int64_t)result;
    }
    int64_t new_len = end - start;
    int64_t* result = (int64_t*)bmb_alloc((new_len + 2) * sizeof(int64_t));
    result[0] = new_len;
    result[1] = new_len;
    for (int64_t i = 0; i < new_len; i++) result[2 + i] = arr[2 + start + i];
    return (int64_t)result;
}

int64_t bmb_array_len(int64_t arr_ptr) {
    int64_t* arr = (int64_t*)arr_ptr;
    return arr[1];
}

// v0.90.97: Integer methods — clamp, pow
int64_t bmb_clamp(int64_t n, int64_t lo, int64_t hi) {
    if (n < lo) return lo;
    if (n > hi) return hi;
    return n;
}

int64_t bmb_pow(int64_t base, int64_t exp) {
    if (exp < 0) return 0;
    int64_t result = 1;
    while (exp > 0) {
        if (exp & 1) result *= base;
        base *= base;
        exp >>= 1;
    }
    return result;
}

// v0.99: String conversion functions
// v0.88.2: Uses arena-aware allocation
char* bmb_char_to_string(int32_t c) {
    char* s = (char*)bmb_alloc(5);  // UTF-8 max 4 bytes + null
    if (c < 0x80) {
        s[0] = (char)c;
        s[1] = '\0';
    } else if (c < 0x800) {
        s[0] = (char)(0xC0 | (c >> 6));
        s[1] = (char)(0x80 | (c & 0x3F));
        s[2] = '\0';
    } else if (c < 0x10000) {
        s[0] = (char)(0xE0 | (c >> 12));
        s[1] = (char)(0x80 | ((c >> 6) & 0x3F));
        s[2] = (char)(0x80 | (c & 0x3F));
        s[3] = '\0';
    } else {
        s[0] = (char)(0xF0 | (c >> 18));
        s[1] = (char)(0x80 | ((c >> 12) & 0x3F));
        s[2] = (char)(0x80 | ((c >> 6) & 0x3F));
        s[3] = (char)(0x80 | (c & 0x3F));
        s[4] = '\0';
    }
    return s;
}

// v0.88.2: Uses arena-aware allocation
char* bmb_int_to_string(int64_t n) {
    char* s = (char*)bmb_alloc(21);  // Max i64 is 20 digits + sign
    snprintf(s, 21, "%ld", (long)n);
    return s;
}

// v0.60.244: Fast integer-to-BmbString conversion for bootstrap compiler
// Returns BmbString* which matches the bootstrap's String type
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_fast_i2s(int64_t n) {
    char* s = (char*)bmb_alloc(21);
    snprintf(s, 21, "%" PRId64, n);
    return bmb_string_wrap(s);
}

// Memory access functions
void bmb_store_i64(int64_t ptr, int64_t value) {
    *((int64_t*)ptr) = value;
}

int64_t bmb_load_i64(int64_t ptr) {
    return *((int64_t*)ptr);
}

// v0.51.51: Byte-level memory access for high-performance string parsing
int64_t bmb_load_u8(int64_t ptr) {
    return (int64_t)(*((unsigned char*)ptr));
}

void bmb_store_u8(int64_t ptr, int64_t value) {
    *((unsigned char*)ptr) = (unsigned char)value;
}

// Get raw pointer to string data for direct memory access
int64_t bmb_str_data(const char* s) {
    return (int64_t)s;
}

// calloc wrapper (returns pointer as i64)
int64_t bmb_calloc(int64_t count, int64_t size) {
    return (int64_t)calloc((size_t)count, (size_t)size);
}

// Box convenience
int64_t bmb_box_new_i64(int64_t value) {
    int64_t* ptr = (int64_t*)malloc(sizeof(int64_t));
    *ptr = value;
    return (int64_t)ptr;
}

// v0.51.51: String concatenation - updated to work with BmbString structs
// Parameters are BmbString* (from LLVM IR %BmbString structs)
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_string_concat(const BmbString* a, const BmbString* b) {
    if (!a || !b || !a->data || !b->data) {
        return bmb_string_wrap(NULL);
    }
    int64_t len_a = a->len;
    int64_t len_b = b->len;
    int64_t total = len_a + len_b;
    char* result = (char*)bmb_alloc(total + 1);
    memcpy(result, a->data, (size_t)len_a);
    memcpy(result + len_a, b->data, (size_t)len_b);
    result[total] = '\0';

    BmbString* s = (BmbString*)bmb_alloc(sizeof(BmbString));
    s->data = result;
    s->len = total;
    s->cap = total;
    return s;
}

// v0.90.90: Multi-string concat — single allocation for 3/5/7 strings
BmbString* bmb_string_concat3(const BmbString* a, const BmbString* b, const BmbString* c) {
    int64_t la = (a && a->data) ? a->len : 0;
    int64_t lb = (b && b->data) ? b->len : 0;
    int64_t lc = (c && c->data) ? c->len : 0;
    int64_t total = la + lb + lc;
    char* result = (char*)bmb_alloc(total + 1);
    if (la) memcpy(result, a->data, (size_t)la);
    if (lb) memcpy(result + la, b->data, (size_t)lb);
    if (lc) memcpy(result + la + lb, c->data, (size_t)lc);
    result[total] = '\0';
    BmbString* s = (BmbString*)bmb_alloc(sizeof(BmbString));
    s->data = result; s->len = total; s->cap = total;
    return s;
}

BmbString* bmb_string_concat5(const BmbString* a, const BmbString* b, const BmbString* c,
                               const BmbString* d, const BmbString* e) {
    int64_t la = (a && a->data) ? a->len : 0;
    int64_t lb = (b && b->data) ? b->len : 0;
    int64_t lc = (c && c->data) ? c->len : 0;
    int64_t ld = (d && d->data) ? d->len : 0;
    int64_t le = (e && e->data) ? e->len : 0;
    int64_t total = la + lb + lc + ld + le;
    char* result = (char*)bmb_alloc(total + 1);
    int64_t off = 0;
    if (la) { memcpy(result + off, a->data, (size_t)la); off += la; }
    if (lb) { memcpy(result + off, b->data, (size_t)lb); off += lb; }
    if (lc) { memcpy(result + off, c->data, (size_t)lc); off += lc; }
    if (ld) { memcpy(result + off, d->data, (size_t)ld); off += ld; }
    if (le) { memcpy(result + off, e->data, (size_t)le); off += le; }
    result[total] = '\0';
    BmbString* s = (BmbString*)bmb_alloc(sizeof(BmbString));
    s->data = result; s->len = total; s->cap = total;
    return s;
}

BmbString* bmb_string_concat7(const BmbString* a, const BmbString* b, const BmbString* c,
                               const BmbString* d, const BmbString* e, const BmbString* f,
                               const BmbString* g) {
    int64_t la = (a && a->data) ? a->len : 0;
    int64_t lb = (b && b->data) ? b->len : 0;
    int64_t lc = (c && c->data) ? c->len : 0;
    int64_t ld = (d && d->data) ? d->len : 0;
    int64_t le = (e && e->data) ? e->len : 0;
    int64_t lf = (f && f->data) ? f->len : 0;
    int64_t lg = (g && g->data) ? g->len : 0;
    int64_t total = la + lb + lc + ld + le + lf + lg;
    char* result = (char*)bmb_alloc(total + 1);
    int64_t off = 0;
    if (la) { memcpy(result + off, a->data, (size_t)la); off += la; }
    if (lb) { memcpy(result + off, b->data, (size_t)lb); off += lb; }
    if (lc) { memcpy(result + off, c->data, (size_t)lc); off += lc; }
    if (ld) { memcpy(result + off, d->data, (size_t)ld); off += ld; }
    if (le) { memcpy(result + off, e->data, (size_t)le); off += le; }
    if (lf) { memcpy(result + off, f->data, (size_t)lf); off += lf; }
    if (lg) { memcpy(result + off, g->data, (size_t)lg); off += lg; }
    result[total] = '\0';
    BmbString* s = (BmbString*)bmb_alloc(sizeof(BmbString));
    s->data = result; s->len = total; s->cap = total;
    return s;
}

// v0.51.51: String functions updated for BmbString structs

// Convert C string to BmbString
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_string_from_cstr(const char* s) {
    if (!s) return bmb_string_wrap(NULL);
    int64_t len = (int64_t)strlen(s);
    char* copy = (char*)bmb_alloc(len + 1);
    memcpy(copy, s, (size_t)(len + 1));
    BmbString* str = (BmbString*)bmb_alloc(sizeof(BmbString));
    str->data = copy;
    str->len = len;
    str->cap = len;
    return str;
}

// Create new string with given length (allocates copy)
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_string_new(const char* s, int64_t len) {
    char* result = (char*)bmb_alloc(len + 1);
    memcpy(result, s, (size_t)len);
    result[len] = '\0';
    BmbString* str = (BmbString*)bmb_alloc(sizeof(BmbString));
    str->data = result;
    str->len = len;
    str->cap = len;
    return str;
}

// String length - parameter is BmbString*
int64_t bmb_string_len(const BmbString* s) {
    if (!s) return 0;
    return s->len;
}

// Get byte at index - parameter is BmbString*
int64_t bmb_string_char_at(const BmbString* s, int64_t index) {
    if (!s || !s->data || index < 0 || index >= s->len) return 0;
    return (int64_t)(unsigned char)s->data[index];
}

// v0.51.51: Alias for codegen compatibility - takes BmbString*
int64_t char_at(const BmbString* s, int64_t index) {
    return bmb_string_char_at(s, index);
}

// String equality comparison - parameters are BmbString*
int64_t bmb_string_eq(const BmbString* a, const BmbString* b) {
    if (a == b) return 1;  // Same struct pointer
    if (!a || !b) return 0;
    if (a->len != b->len) return 0;  // Different lengths
    return memcmp(a->data, b->data, (size_t)a->len) == 0 ? 1 : 0;
}

// String slice (substring from start to end, exclusive) - returns BmbString*
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_string_slice(const BmbString* s, int64_t start, int64_t end) {
    if (!s || !s->data || start < 0 || end < start || start > s->len) {
        return bmb_string_wrap(NULL);
    }
    if (end > s->len) end = s->len;
    int64_t len = end - start;
    char* result = (char*)bmb_alloc(len + 1);
    memcpy(result, s->data + start, (size_t)len);
    result[len] = '\0';
    BmbString* str = (BmbString*)bmb_alloc(sizeof(BmbString));
    str->data = result;
    str->len = len;
    str->cap = len;
    return str;
}

// v0.51.51: Wrapper functions updated for BmbString*
BmbString* slice(const BmbString* s, int64_t start, int64_t end) {
    return bmb_string_slice(s, start, end);
}

int64_t byte_at(const BmbString* s, int64_t index) {
    return bmb_string_char_at(s, index);
}

// v0.51.51: len wrapper takes BmbString*
int64_t len(const BmbString* s) {
    return bmb_string_len(s);
}

// v0.51.51: chr returns BmbString*
// v0.88.2: Uses arena-aware allocation
BmbString* chr(int64_t n) {
    char* data = (char*)bmb_alloc(2);
    data[0] = (char)n;
    data[1] = '\0';
    return bmb_string_wrap(data);
}

// v0.51.51: char_to_string returns BmbString*
// v0.88.2: Uses arena-aware allocation
BmbString* char_to_string(int32_t c) {
    char* data = (char*)bmb_alloc(2);
    data[0] = (char)c;
    data[1] = '\0';
    return bmb_string_wrap(data);
}

// v0.51.51: ord takes BmbString*
int64_t ord(const BmbString* s) {
    if (!s || !s->data || s->len == 0) return 0;
    return (int64_t)(unsigned char)s->data[0];
}

// v0.51.51: Low-level memory access wrappers for high-performance parsing
int64_t load_u8(int64_t ptr) {
    return bmb_load_u8(ptr);
}

void store_u8(int64_t ptr, int64_t value) {
    bmb_store_u8(ptr, value);
}

// v0.90.3: Unprefixed aliases for bootstrap compatibility
void store_i64(int64_t ptr, int64_t value) {
    bmb_store_i64(ptr, value);
}

int64_t load_i64(int64_t ptr) {
    return bmb_load_i64(ptr);
}

int64_t vec_new(void) {
    return bmb_vec_new();
}

void vec_push(int64_t vec_ptr, int64_t value) {
    bmb_vec_push(vec_ptr, value);
}

int64_t vec_get(int64_t vec_ptr, int64_t index) {
    return bmb_vec_get(vec_ptr, index);
}

void vec_set(int64_t vec_ptr, int64_t index, int64_t value) {
    bmb_vec_set(vec_ptr, index, value);
}

int64_t vec_len(int64_t vec_ptr) {
    return bmb_vec_len(vec_ptr);
}

void vec_free(int64_t vec_ptr) {
    bmb_vec_free(vec_ptr);
}

// v0.51.51: str_data takes BmbString*, returns pointer to data
int64_t str_data(const BmbString* s) {
    if (!s || !s->data) return 0;
    return (int64_t)s->data;
}

// v0.51.51: print_str takes BmbString*
void print_str(const BmbString* s) {
    if (s && s->data) printf("%s", s->data);
}

void println(int64_t n) {
    bmb_println_i64(n);
}

// v0.60.43: Float output wrappers
void println_f64(double f) {
    bmb_println_f64(f);
}

void print_f64(double f) {
    bmb_print_f64(f);
}

// v0.51.51: println_str takes BmbString*
void println_str(const BmbString* s) {
    if (s && s->data) printf("%s\n", s->data);
    else printf("\n");
}

// v0.46: StringBuilder functions for efficient string building
typedef struct {
    char* data;
    int64_t len;
    int64_t cap;
} StringBuilder;

int64_t bmb_sb_new(void) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    sb->cap = 1024;
    sb->len = 0;
    sb->data = (char*)malloc(sb->cap);
    sb->data[0] = '\0';
    return (int64_t)sb;
}

// v0.51.45: StringBuilder with pre-allocated capacity (P0-E optimization)
// Avoids reallocations when final size is known
int64_t bmb_sb_with_capacity(int64_t capacity) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    sb->cap = capacity > 0 ? capacity : 64;
    sb->len = 0;
    sb->data = (char*)malloc(sb->cap);
    sb->data[0] = '\0';
    return (int64_t)sb;
}

// v0.51.51: sb_push takes BmbString*
// v0.90.90: Use memcpy for bulk copy
int64_t bmb_sb_push(int64_t handle, const BmbString* s) {
    if (!s || !s->data || !handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;
    int64_t slen = s->len;
    if (slen == 0) return sb->len;

    // Grow if needed
    int64_t required = sb->len + slen + 1;
    if (required > sb->cap) {
        int64_t new_cap = sb->cap;
        while (new_cap < required) new_cap *= 2;
        sb->data = (char*)realloc(sb->data, new_cap);
        sb->cap = new_cap;
    }

    // Append with memcpy
    memcpy(sb->data + sb->len, s->data, (size_t)slen);
    sb->len += slen;
    sb->data[sb->len] = '\0';
    return sb->len;
}

// v0.51.18: Push a single character to StringBuilder
int64_t bmb_sb_push_char(int64_t handle, int64_t ch) {
    if (!handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;

    // Grow if needed
    if (sb->len + 2 > sb->cap) {
        sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }

    sb->data[sb->len++] = (char)ch;
    sb->data[sb->len] = '\0';
    return sb->len;
}

// v0.51.18: Push an integer as string to StringBuilder
int64_t bmb_sb_push_int(int64_t handle, int64_t n) {
    if (!handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;

    // Convert integer to string (max 20 digits for i64 + sign + null)
    char buf[32];
    int neg = (n < 0);
    if (neg) n = -n;

    int i = 0;
    if (n == 0) {
        buf[i++] = '0';
    } else {
        while (n > 0) {
            buf[i++] = '0' + (n % 10);
            n /= 10;
        }
    }
    if (neg) buf[i++] = '-';

    // Grow if needed
    while (sb->len + i + 1 > sb->cap) {
        sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }

    // Append in reverse order
    while (i > 0) {
        sb->data[sb->len++] = buf[--i];
    }
    sb->data[sb->len] = '\0';
    return sb->len;
}

// v0.51.51: Push escaped string (JSON-style escaping) to StringBuilder - takes BmbString*
int64_t bmb_sb_push_escaped(int64_t handle, const BmbString* s) {
    if (!handle || !s || !s->data) return 0;
    StringBuilder* sb = (StringBuilder*)handle;

    for (int64_t i = 0; i < s->len; i++) {
        char c = s->data[i];
        char esc = 0;
        switch (c) {
            case '"':  esc = '"';  break;
            case '\\': esc = '\\'; break;
            case '\n': esc = 'n';  break;
            case '\r': esc = 'r';  break;
            case '\t': esc = 't';  break;
            default: break;
        }

        // Grow if needed (max 2 chars per iteration)
        if (sb->len + 3 > sb->cap) {
            sb->cap *= 2;
            sb->data = (char*)realloc(sb->data, sb->cap);
        }

        if (esc) {
            sb->data[sb->len++] = '\\';
            sb->data[sb->len++] = esc;
        } else {
            sb->data[sb->len++] = c;
        }
    }
    sb->data[sb->len] = '\0';
    return sb->len;
}

int64_t bmb_sb_len(int64_t handle) {
    StringBuilder* sb = (StringBuilder*)handle;
    return sb->len;
}

// v0.51.51: sb_build returns BmbString*
// v0.88.2: Uses arena-aware allocation
BmbString* bmb_sb_build(int64_t handle) {
    if (!handle) {
        return bmb_string_wrap(NULL);
    }
    StringBuilder* sb = (StringBuilder*)handle;
    // Return copy of the built string (use memcpy + direct struct init)
    int64_t len = sb->len;
    char* result = (char*)bmb_alloc(len + 1);
    memcpy(result, sb->data, (size_t)(len + 1));
    BmbString* str = (BmbString*)bmb_alloc(sizeof(BmbString));
    str->data = result;
    str->len = len;
    str->cap = len;
    return str;
}

int64_t bmb_sb_clear(int64_t handle) {
    StringBuilder* sb = (StringBuilder*)handle;
    sb->len = 0;
    sb->data[0] = '\0';
    return 0;
}

// v0.88.6: Search for a comma-separated entry in SB buffer (zero arena allocation)
int64_t bmb_sb_contains(int64_t handle, const BmbString* marker) {
    if (!handle || !marker || !marker->data) return 0;
    StringBuilder* sb = (StringBuilder*)handle;
    int64_t mlen = marker->len;
    if (mlen == 0 || sb->len == 0) return 0;
    int64_t pos = 0;
    while (pos <= sb->len - mlen) {
        // Check if entry at pos matches marker
        int64_t match = 1;
        for (int64_t i = 0; i < mlen; i++) {
            if (sb->data[pos + i] != marker->data[i]) { match = 0; break; }
        }
        if (match) {
            // Verify it's a complete entry (at start/after comma, at end/before comma)
            int64_t at_start = (pos == 0) || (sb->data[pos - 1] == ',');
            int64_t at_end = (pos + mlen >= sb->len) || (sb->data[pos + mlen] == ',');
            if (at_start && at_end) return 1;
        }
        // Skip to next comma
        while (pos < sb->len && sb->data[pos] != ',') pos++;
        pos++; // skip comma
    }
    return 0;
}

// v0.60.63: Print StringBuilder directly without allocation
int64_t bmb_sb_println(int64_t handle) {
    StringBuilder* sb = (StringBuilder*)handle;
    if (sb && sb->data) {
        puts(sb->data);  // puts adds newline automatically
    } else {
        puts("");
    }
    return 0;
}

// v0.60.65: Print null-terminated C string directly
int64_t bmb_puts_cstr(const char* s) {
    if (s) {
        puts(s);
    } else {
        puts("");
    }
    return 0;
}

// v0.88.2: Memory management functions

// Free a BmbString and its data
// Note: In arena mode, individual frees are no-ops (arena handles bulk deallocation)
int64_t bmb_string_free(const BmbString* s) {
    if (!s) return 0;
    if (g_arena_enabled) return 0;  // Arena handles deallocation
    if (s->data) free(s->data);
    free((void*)s);
    return 0;
}

// Wrapper for BMB code
int64_t free_string(const BmbString* s) { return bmb_string_free(s); }

// Free a StringBuilder and its buffer
int64_t bmb_sb_free(int64_t handle) {
    if (handle) {
        StringBuilder* sb = (StringBuilder*)handle;
        if (sb->data) free(sb->data);
        free(sb);
    }
    return 0;
}

int64_t sb_free(int64_t handle) { return bmb_sb_free(handle); }

// v0.88.2: Arena allocator for bulk string allocation
// Reduces malloc overhead and enables bulk deallocation
// v0.88.4: BMB has no GC/destructors - arena is the ONLY memory management.
//   Without arena, every string allocation leaks (malloc without free).
//   Arena pools all allocations and frees at process exit.
//   Limit default 4GB, configurable via BMB_ARENA_MAX_SIZE env var.
//   When limit exceeded: EXIT with error (not BSOD from OOM).
#define BMB_ARENA_BLOCK_SIZE (8 * 1024 * 1024)  // 8MB blocks
#define BMB_ARENA_DEFAULT_MAX_SIZE ((size_t)4 * 1024 * 1024 * 1024)  // 4GB default
static size_t g_arena_max_size = 0;  // 0 = not yet initialized
static int g_arena_limit_warned = 0;

typedef struct BmbArenaBlock {
    char* data;
    size_t used;
    size_t capacity;
    struct BmbArenaBlock* next;
} BmbArenaBlock;

static BmbArenaBlock* g_arena_head = NULL;
static BmbArenaBlock* g_arena_current = NULL;
// g_arena_enabled is forward-declared near top of file
static size_t g_arena_total_allocated = 0;

static BmbArenaBlock* bmb_arena_new_block(size_t min_size) {
    size_t cap = min_size > BMB_ARENA_BLOCK_SIZE ? min_size + 64 : BMB_ARENA_BLOCK_SIZE;
    BmbArenaBlock* block = (BmbArenaBlock*)malloc(sizeof(BmbArenaBlock));
    if (!block) return NULL;
    block->data = (char*)malloc(cap);
    if (!block->data) { free(block); return NULL; }
    block->used = 0;
    block->capacity = cap;
    block->next = NULL;
    g_arena_total_allocated += cap;
    return block;
}

// v0.88.4: Initialize arena max size from environment variable
static void bmb_arena_init_limit(void) {
    if (g_arena_max_size != 0) return;  // Already initialized
    const char* env = getenv("BMB_ARENA_MAX_SIZE");
    if (env) {
        size_t val = 0;
        for (int i = 0; env[i]; i++) {
            if (env[i] >= '0' && env[i] <= '9') val = val * 10 + (env[i] - '0');
        }
        // Support M/G suffixes
        if (env[0]) {
            char last = env[0];
            for (int i = 1; env[i]; i++) last = env[i];
            if (last == 'G' || last == 'g') val *= (size_t)1024 * 1024 * 1024;
            else if (last == 'M' || last == 'm') val *= (size_t)1024 * 1024;
        }
        g_arena_max_size = val > 0 ? val : BMB_ARENA_DEFAULT_MAX_SIZE;
    } else {
        g_arena_max_size = BMB_ARENA_DEFAULT_MAX_SIZE;
    }
}

static inline void* bmb_arena_alloc(size_t size) {
    // Align to 8 bytes
    size = (size + 7) & ~((size_t)7);

    // Fast path: arena has space (most common case)
    if (g_arena_current && g_arena_current->used + size <= g_arena_current->capacity) {
        void* ptr = g_arena_current->data + g_arena_current->used;
        g_arena_current->used += size;
        return ptr;
    }

    // Slow path: need new block or limit check
    bmb_arena_init_limit();
    if (g_arena_total_allocated + size > g_arena_max_size) {
        fprintf(stderr, "[bmb] FATAL: arena memory limit exceeded (%zu MB / %zu MB max)\n",
                g_arena_total_allocated / (1024 * 1024),
                g_arena_max_size / (1024 * 1024));
        fprintf(stderr, "[bmb] Set BMB_ARENA_MAX_SIZE environment variable to increase (e.g. 8G)\n");
        exit(1);
    }

    if (!g_arena_current || g_arena_current->used + size > g_arena_current->capacity) {
        BmbArenaBlock* block = bmb_arena_new_block(size);
        if (!block) return malloc(size);  // fallback to malloc
        if (g_arena_current) {
            g_arena_current->next = block;
        } else {
            g_arena_head = block;
        }
        g_arena_current = block;
    }

    void* ptr = g_arena_current->data + g_arena_current->used;
    g_arena_current->used += size;
    return ptr;
}

// Enable/disable arena mode (1=enable, 0=disable)
int64_t bmb_arena_mode(int64_t enable) {
    g_arena_enabled = (int)enable;
    if (enable && !g_arena_head) {
        g_arena_head = bmb_arena_new_block(BMB_ARENA_BLOCK_SIZE);
        g_arena_current = g_arena_head;
    }
    return 0;
}

// Reset arena (free all blocks except the first, reset the first)
int64_t bmb_arena_reset(void) {
    if (!g_arena_head) return 0;

    // Free all blocks except the first
    BmbArenaBlock* block = g_arena_head->next;
    while (block) {
        BmbArenaBlock* next = block->next;
        g_arena_total_allocated -= block->capacity;
        free(block->data);
        free(block);
        block = next;
    }

    // Reset the first block
    g_arena_head->used = 0;
    g_arena_head->next = NULL;
    g_arena_current = g_arena_head;
    return 0;
}

// v0.88.6: Arena save/restore for per-function memory reclamation
static BmbArenaBlock* g_arena_save_block = NULL;
static size_t g_arena_save_used = 0;
static size_t g_arena_save_total = 0;

int64_t bmb_arena_save(void) {
    g_arena_save_block = g_arena_current;
    g_arena_save_used = g_arena_current ? g_arena_current->used : 0;
    g_arena_save_total = g_arena_total_allocated;
    return 0;
}

int64_t bmb_arena_restore(void) {
    if (!g_arena_save_block) return 0;
    BmbArenaBlock* block = g_arena_save_block->next;
    while (block) {
        BmbArenaBlock* next = block->next;
        free(block->data);
        free(block);
        block = next;
    }
    g_arena_save_block->next = NULL;
    g_arena_save_block->used = g_arena_save_used;
    g_arena_current = g_arena_save_block;
    g_arena_total_allocated = g_arena_save_total;
    return 0;
}

// Get current arena memory usage
int64_t bmb_arena_usage(void) {
    return (int64_t)g_arena_total_allocated;
}

// Free all arena memory
int64_t bmb_arena_destroy(void) {
    BmbArenaBlock* block = g_arena_head;
    while (block) {
        BmbArenaBlock* next = block->next;
        free(block->data);
        free(block);
        block = next;
    }
    g_arena_head = NULL;
    g_arena_current = NULL;
    g_arena_total_allocated = 0;
    g_arena_enabled = 0;
    g_arena_limit_warned = 0;
    return 0;
}

// Wrappers for BMB code
int64_t arena_mode(int64_t enable) { return bmb_arena_mode(enable); }
int64_t arena_reset(void) { return bmb_arena_reset(); }
int64_t arena_save(void) { return bmb_arena_save(); }
int64_t arena_restore(void) { return bmb_arena_restore(); }
int64_t arena_usage(void) { return bmb_arena_usage(); }
int64_t arena_destroy(void) { return bmb_arena_destroy(); }

// Arena-aware allocation helper (used internally)
static void* bmb_alloc(size_t size) {
    if (g_arena_enabled) return bmb_arena_alloc(size);
    return malloc(size);
}

// v0.51.18: HashMap implementation (open addressing with linear probing)
// Based on: https://github.com/DavidLeeds/hashmap
// See also: https://en.wikipedia.org/wiki/Linear_probing

#define HASHMAP_INITIAL_CAPACITY 131072  // Power of 2, suitable for ~100k entries

typedef struct {
    int64_t key;
    int64_t value;
    int state;  // 0=empty, 1=occupied, 2=deleted (tombstone)
} HashEntry;

typedef struct {
    HashEntry* entries;
    int64_t count;
    int64_t capacity;
} HashMap;

// Hash function for i64 keys (Fibonacci hashing variant)
static int64_t hashmap_hash_i64(int64_t key) {
    uint64_t h = (uint64_t)key * 0x517cc1b727220a95ULL;
    return (int64_t)(h ^ (h >> 32));
}

int64_t hashmap_new(void) {
    HashMap* m = (HashMap*)malloc(sizeof(HashMap));
    if (!m) return 0;
    m->entries = (HashEntry*)calloc(HASHMAP_INITIAL_CAPACITY, sizeof(HashEntry));
    if (!m->entries) {
        free(m);
        return 0;
    }
    m->count = 0;
    m->capacity = HASHMAP_INITIAL_CAPACITY;
    return (int64_t)m;
}

void hashmap_free(int64_t handle) {
    if (!handle) return;
    HashMap* m = (HashMap*)handle;
    free(m->entries);
    free(m);
}

int64_t hashmap_len(int64_t handle) {
    if (!handle) return 0;
    HashMap* m = (HashMap*)handle;
    return m->count;
}

int64_t hashmap_insert(int64_t handle, int64_t key, int64_t value) {
    if (!handle) return 0;
    HashMap* m = (HashMap*)handle;

    int64_t hash = hashmap_hash_i64(key);
    int64_t mask = m->capacity - 1;
    int64_t idx = hash & mask;

    for (int64_t i = 0; i < m->capacity; i++) {
        HashEntry* e = &m->entries[idx];
        if (e->state == 0 || e->state == 2) {
            // Empty or deleted slot - insert here
            e->key = key;
            e->value = value;
            e->state = 1;
            m->count++;
            return 0;
        } else if (e->state == 1 && e->key == key) {
            // Key exists - update value
            int64_t old = e->value;
            e->value = value;
            return old;
        }
        idx = (idx + 1) & mask;
    }
    return 0;  // Table full (shouldn't happen with proper sizing)
}

int64_t hashmap_get(int64_t handle, int64_t key) {
    if (!handle) return INT64_MIN;
    HashMap* m = (HashMap*)handle;

    int64_t hash = hashmap_hash_i64(key);
    int64_t mask = m->capacity - 1;
    int64_t idx = hash & mask;

    for (int64_t i = 0; i < m->capacity; i++) {
        HashEntry* e = &m->entries[idx];
        if (e->state == 0) {
            // Empty slot - key not found
            return INT64_MIN;
        } else if (e->state == 1 && e->key == key) {
            // Found
            return e->value;
        }
        // Continue probing (skip deleted slots)
        idx = (idx + 1) & mask;
    }
    return INT64_MIN;
}

int64_t hashmap_remove(int64_t handle, int64_t key) {
    if (!handle) return INT64_MIN;
    HashMap* m = (HashMap*)handle;

    int64_t hash = hashmap_hash_i64(key);
    int64_t mask = m->capacity - 1;
    int64_t idx = hash & mask;

    for (int64_t i = 0; i < m->capacity; i++) {
        HashEntry* e = &m->entries[idx];
        if (e->state == 0) {
            // Empty slot - key not found
            return INT64_MIN;
        } else if (e->state == 1 && e->key == key) {
            // Found - mark as deleted (tombstone)
            int64_t old = e->value;
            e->state = 2;
            m->count--;
            return old;
        }
        idx = (idx + 1) & mask;
    }
    return INT64_MIN;
}

// v0.60.262: Check if key exists in hashmap
int64_t hashmap_contains(int64_t handle, int64_t key) {
    if (!handle) return 0;
    HashMap* m = (HashMap*)handle;

    int64_t hash = hashmap_hash_i64(key);
    int64_t mask = m->capacity - 1;
    int64_t idx = hash & mask;

    for (int64_t i = 0; i < m->capacity; i++) {
        HashEntry* e = &m->entries[idx];
        if (e->state == 0) {
            // Empty slot - key not found
            return 0;
        } else if (e->state == 1 && e->key == key) {
            // Found
            return 1;
        }
        idx = (idx + 1) & mask;
    }
    return 0;
}

// v0.90.82: String-content hashmap (keys compared by string content, not pointer identity)
// Uses FNV-1a hash of string bytes for distribution, strcmp for equality
// Same open-addressing with linear probing as the i64 hashmap

typedef struct {
    int64_t key;    // BmbString* (stored as i64)
    int64_t value;  // arbitrary i64 value
    int state;      // 0=empty, 1=occupied, 2=deleted
} StrHashEntry;

typedef struct {
    StrHashEntry* entries;
    int64_t count;
    int64_t capacity;
} StrHashMap;

// FNV-1a hash of string content
static int64_t str_hash_content(int64_t key_handle) {
    if (!key_handle) return 0;
    BmbString* s = (BmbString*)key_handle;
    uint64_t h = 14695981039346656037ULL;  // FNV offset basis
    for (int64_t i = 0; i < s->len; i++) {
        h ^= (uint8_t)s->data[i];
        h *= 1099511628211ULL;  // FNV prime
    }
    return (int64_t)(h ^ (h >> 32));
}

// Compare two BmbString* by content
static int str_key_eq(int64_t a, int64_t b) {
    if (a == b) return 1;  // Same pointer
    if (!a || !b) return 0;
    BmbString* sa = (BmbString*)a;
    BmbString* sb = (BmbString*)b;
    if (sa->len != sb->len) return 0;
    return memcmp(sa->data, sb->data, (size_t)sa->len) == 0;
}

#define STR_HASHMAP_INITIAL_CAPACITY 4096

int64_t str_hashmap_new(void) {
    StrHashMap* m = (StrHashMap*)malloc(sizeof(StrHashMap));
    if (!m) return 0;
    m->entries = (StrHashEntry*)calloc(STR_HASHMAP_INITIAL_CAPACITY, sizeof(StrHashEntry));
    if (!m->entries) { free(m); return 0; }
    m->count = 0;
    m->capacity = STR_HASHMAP_INITIAL_CAPACITY;
    return (int64_t)m;
}

void str_hashmap_free(int64_t handle) {
    if (!handle) return;
    StrHashMap* m = (StrHashMap*)handle;
    free(m->entries);
    free(m);
}

// Resize when load factor > 0.7
static void str_hashmap_resize(StrHashMap* m) {
    int64_t new_cap = m->capacity * 2;
    StrHashEntry* new_entries = (StrHashEntry*)calloc(new_cap, sizeof(StrHashEntry));
    if (!new_entries) return;
    int64_t new_mask = new_cap - 1;
    for (int64_t i = 0; i < m->capacity; i++) {
        StrHashEntry* old = &m->entries[i];
        if (old->state == 1) {
            int64_t hash = str_hash_content(old->key);
            int64_t idx = hash & new_mask;
            while (new_entries[idx].state == 1) {
                idx = (idx + 1) & new_mask;
            }
            new_entries[idx] = *old;
        }
    }
    free(m->entries);
    m->entries = new_entries;
    m->capacity = new_cap;
}

int64_t str_hashmap_insert(int64_t handle, int64_t key, int64_t value) {
    if (!handle) return 0;
    StrHashMap* m = (StrHashMap*)handle;
    // Resize if load factor > 0.7
    if (m->count * 10 > m->capacity * 7) {
        str_hashmap_resize(m);
    }
    int64_t hash = str_hash_content(key);
    int64_t mask = m->capacity - 1;
    int64_t idx = hash & mask;
    for (int64_t i = 0; i < m->capacity; i++) {
        StrHashEntry* e = &m->entries[idx];
        if (e->state == 0 || e->state == 2) {
            e->key = key;
            e->value = value;
            e->state = 1;
            m->count++;
            return 0;
        } else if (e->state == 1 && str_key_eq(e->key, key)) {
            int64_t old = e->value;
            e->value = value;
            return old;
        }
        idx = (idx + 1) & mask;
    }
    return 0;
}

int64_t str_hashmap_get(int64_t handle, int64_t key) {
    if (!handle) return 0;
    StrHashMap* m = (StrHashMap*)handle;
    int64_t hash = str_hash_content(key);
    int64_t mask = m->capacity - 1;
    int64_t idx = hash & mask;
    for (int64_t i = 0; i < m->capacity; i++) {
        StrHashEntry* e = &m->entries[idx];
        if (e->state == 0) return 0;
        if (e->state == 1 && str_key_eq(e->key, key)) return e->value;
        idx = (idx + 1) & mask;
    }
    return 0;
}

// v0.90.83: Cached registry lookup for type checker performance
// Parses "name1=value1;name2=value2;..." into StrHashMap, caches by slot
// 3 cache slots: 0=fn_reg, 1=struct_reg, 2=enum_reg
#define REG_CACHE_SLOTS 3
static StrHashMap* g_reg_caches[REG_CACHE_SLOTS] = {NULL, NULL, NULL};
static int64_t g_reg_cache_lens[REG_CACHE_SLOTS] = {-1, -1, -1};

// Returns BmbString* (ptr) for the looked-up value, or empty string if not found
BmbString* reg_cached_lookup(const BmbString* reg, const BmbString* name, int64_t slot) {
    if (!reg || reg->len == 0 || slot < 0 || slot >= REG_CACHE_SLOTS) {
        return bmb_string_new("", 0);
    }

    // Cache invalidation: registry only grows, so length change = content change
    if (g_reg_caches[slot] == NULL || reg->len != g_reg_cache_lens[slot]) {
        // Rebuild cache
        if (g_reg_caches[slot]) {
            str_hashmap_free((int64_t)g_reg_caches[slot]);
        }
        int64_t map = str_hashmap_new();
        g_reg_cache_lens[slot] = reg->len;

        // Parse "name=sig;name=sig;..."
        const char* d = reg->data;
        int64_t pos = 0;
        while (pos < reg->len) {
            // Find '=' (ASCII 61)
            int64_t eq = pos;
            while (eq < reg->len && d[eq] != '=') eq++;
            if (eq >= reg->len) break;

            // Find ';' (ASCII 59) or end
            int64_t semi = eq + 1;
            while (semi < reg->len && d[semi] != ';') semi++;

            // Create key and value as BmbStrings
            BmbString* key = bmb_string_new(d + pos, eq - pos);
            BmbString* val = bmb_string_new(d + eq + 1, semi - eq - 1);
            str_hashmap_insert(map, (int64_t)key, (int64_t)val);

            pos = (semi < reg->len) ? semi + 1 : reg->len;
        }

        g_reg_caches[slot] = (StrHashMap*)map;
    }

    int64_t result = str_hashmap_get((int64_t)g_reg_caches[slot], (int64_t)name);
    if (result == 0) {
        return bmb_string_new("", 0);
    }
    return (BmbString*)result;
}

// v0.46: Additional file functions
int64_t bmb_file_size(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) return -1;
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fclose(f);
    return (int64_t)size;
}

int64_t bmb_append_file(const char* path, const char* content) {
    FILE* f = fopen(path, "ab");
    if (!f) return -1;
    size_t len = 0;
    while (content[len]) len++;
    size_t written = fwrite(content, 1, len, f);
    fclose(f);
    return (written == len) ? 0 : -1;
}

// v0.46: System functions
// v0.90: Updated to accept BmbString* for bootstrap compatibility
int64_t bmb_system(const BmbString* cmd) {
    if (!cmd || !cmd->data) return -1;
    return (int64_t)system(cmd->data);
}

// v0.88.2: Execute command and capture output (for test runner)
BmbString* bmb_system_capture(const BmbString* cmd) {
    if (!cmd || !cmd->data) return bmb_string_wrap(NULL);
#ifdef _WIN32
    FILE* fp = _popen(cmd->data, "r");
#else
    FILE* fp = popen(cmd->data, "r");
#endif
    if (!fp) return bmb_string_wrap(NULL);

    // Read output into StringBuilder
    size_t cap = 4096;
    size_t len = 0;
    char* buf = (char*)malloc(cap);
    if (!buf) {
#ifdef _WIN32
        _pclose(fp);
#else
        pclose(fp);
#endif
        return bmb_string_wrap(NULL);
    }

    int ch;
    while ((ch = fgetc(fp)) != EOF) {
        if (len + 1 >= cap) {
            cap *= 2;
            char* newbuf = (char*)realloc(buf, cap);
            if (!newbuf) break;
            buf = newbuf;
        }
        buf[len++] = (char)ch;
    }
    buf[len] = '\0';

#ifdef _WIN32
    _pclose(fp);
#else
    pclose(fp);
#endif

    BmbString* s = (BmbString*)bmb_alloc(sizeof(BmbString));
    // v0.88.3: Copy buf into arena-allocated memory and free the malloc'd buffer
    if (g_arena_enabled) {
        char* arena_buf = (char*)bmb_alloc(len + 1);
        for (size_t i = 0; i <= len; i++) arena_buf[i] = buf[i];
        free(buf);
        s->data = arena_buf;
    } else {
        s->data = buf;
    }
    s->len = (int64_t)len;
    s->cap = (int64_t)(g_arena_enabled ? len : cap);
    return s;
}

// Wrapper for BMB code
BmbString* system_capture(const BmbString* cmd) { return bmb_system_capture(cmd); }

// v0.88.4: exec_output(command, args) -> String
// Execute command with arguments, capture stdout+stderr
BmbString* bmb_exec_output(const BmbString* command, const BmbString* args) {
    if (!command || !command->data) return bmb_string_wrap(NULL);
    // Build: "command args 2>&1"
    size_t cmd_len = command->len;
    size_t args_len = args && args->data ? args->len : 0;
    size_t total = cmd_len + 1 + args_len + 6; // " " + " 2>&1" + null
    char* full_cmd = (char*)malloc(total);
    if (!full_cmd) return bmb_string_wrap(NULL);
    memcpy(full_cmd, command->data, cmd_len);
    size_t pos = cmd_len;
    if (args_len > 0) {
        full_cmd[pos++] = ' ';
        memcpy(full_cmd + pos, args->data, args_len);
        pos += args_len;
    }
    memcpy(full_cmd + pos, " 2>&1", 5);
    pos += 5;
    full_cmd[pos] = '\0';
    // Create a temp BmbString for system_capture
    BmbString temp;
    temp.data = full_cmd;
    temp.len = (int64_t)pos;
    temp.cap = (int64_t)total;
    BmbString* result = bmb_system_capture(&temp);
    free(full_cmd);
    return result;
}
BmbString* exec_output(const BmbString* command, const BmbString* args) {
    return bmb_exec_output(command, args);
}

// v0.90: Updated to accept/return BmbString* for bootstrap compatibility
BmbString* bmb_getenv(const BmbString* name) {
    const char* cname = name ? name->data : "";
    const char* val = getenv(cname);
    if (!val) {
        char* empty = (char*)bmb_alloc(1);
        empty[0] = '\0';
        return bmb_string_wrap(empty);
    }
    // Return copy as BmbString
    size_t len = 0;
    while (val[len]) len++;
    char* result = (char*)bmb_alloc(len + 1);
    for (size_t i = 0; i <= len; i++) result[i] = val[i];
    return bmb_string_wrap(result);
}

// v0.46: File I/O support for CLI Independence
#include <string.h>
#include <sys/stat.h>

// v0.51.51: read_file returns BmbString*, path is BmbString*
BmbString* bmb_read_file(const BmbString* path) {
    if (!path || !path->data) return bmb_string_wrap(NULL);
    FILE* f = fopen(path->data, "rb");
    if (!f) {
        return bmb_string_wrap(NULL);
    }
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    char* content = (char*)malloc(size + 1);
    if (content) {
        size_t read = fread(content, 1, size, f);
        content[read] = '\0';
    }
    fclose(f);
    return bmb_string_wrap(content);
}

// v0.51.51: write_file takes BmbString* for path and content
int64_t bmb_write_file(const BmbString* path, const BmbString* content) {
    if (!path || !path->data || !content || !content->data) return -1;
    FILE* f = fopen(path->data, "wb");
    if (!f) return -1;
    size_t written = fwrite(content->data, 1, content->len, f);
    fclose(f);
    return (written == (size_t)content->len) ? 0 : -1;
}

// v0.60.80: write_file_newlines - converts | to newlines during write
// Used by bootstrap compiler which uses | as line separator
int64_t bmb_write_file_newlines(const BmbString* path, const BmbString* content) {
    if (!path || !path->data || !content || !content->data) return -1;
    FILE* f = fopen(path->data, "wb");
    if (!f) return -1;

    size_t written = 0;
    for (size_t i = 0; i < content->len; i++) {
        char c = content->data[i];
        if (c == '|') {
            fputc('\n', f);
        } else {
            fputc(c, f);
        }
        written++;
    }

    fclose(f);
    return (written == content->len) ? 0 : -1;
}

// v0.90: Updated to accept BmbString* for bootstrap compatibility
int64_t bmb_file_exists(const BmbString* path) {
    if (!path || !path->data) return 0;
    struct stat st;
    return (stat(path->data, &st) == 0) ? 1 : 0;
}

// v0.51.18: _cstr variants for string literal optimization (zero overhead)
// These take raw C strings directly, avoiding BMB String wrapper overhead
int64_t file_exists_cstr(const char* path) {
    struct stat st;
    return (stat(path, &st) == 0) ? 1 : 0;
}

int64_t bmb_file_exists_cstr(const char* path) {
    struct stat st;
    return (stat(path, &st) == 0) ? 1 : 0;
}

// v0.60.262: BmbString* wrappers for file operations
int64_t file_exists(const BmbString* path) {
    struct stat st;
    return (stat(path->data, &st) == 0) ? 1 : 0;
}

int64_t file_size(const BmbString* path) {
    FILE* f = fopen(path->data, "rb");
    if (!f) return -1;
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fclose(f);
    return (int64_t)size;
}

// v0.46: Command-line argument support for CLI Independence
static int g_argc = 0;
static char** g_argv = NULL;

int64_t bmb_arg_count(void) {
    return (int64_t)g_argc;
}

// v0.60.87: Returns BMB string struct, not raw C string
BmbString* bmb_get_arg(int64_t index) {
    if (index < 0 || index >= g_argc) {
        // Return empty string for out-of-bounds
        return bmb_string_from_cstr("");
    }
    // Return the argument as a BMB string
    return bmb_string_from_cstr(g_argv[index]);
}

// v0.50.20: StringBuilder wrappers
// Note: str_concat and str_eq wrappers removed in v0.51.18 to avoid symbol collisions
// Use bmb_string_concat and bmb_string_eq instead
int64_t sb_new(void) {
    return bmb_sb_new();
}

// v0.51.45: sb_with_capacity wrapper
int64_t sb_with_capacity(int64_t capacity) {
    return bmb_sb_with_capacity(capacity);
}

// v0.51.51: sb_push takes BmbString*
int64_t sb_push(int64_t handle, const BmbString* s) {
    return bmb_sb_push(handle, s);
}

// v0.51.51: sb_push_cstr takes raw char* for internal use
int64_t sb_push_cstr(int64_t handle, const char* s) {
    if (!s || !handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;
    int64_t slen = 0;
    while (s[slen]) slen++;
    while (sb->len + slen + 1 > sb->cap) {
        sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }
    for (int64_t i = 0; i < slen; i++) {
        sb->data[sb->len + i] = s[i];
    }
    sb->len += slen;
    sb->data[sb->len] = '\0';
    return sb->len;
}

int64_t sb_push_char(int64_t handle, int64_t ch) {
    return bmb_sb_push_char(handle, ch);
}

int64_t sb_push_int(int64_t handle, int64_t n) {
    return bmb_sb_push_int(handle, n);
}

// v0.51.51: sb_push_escaped takes BmbString*
int64_t sb_push_escaped(int64_t handle, const BmbString* s) {
    if (!s || !s->data || !handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;
    for (int64_t i = 0; i < s->len; i++) {
        char c = s->data[i];
        if (c == '\\' || c == '"') sb_push_char(handle, '\\');
        sb_push_char(handle, c);
    }
    return sb->len;
}

// v0.51.51: sb_build returns BmbString*
BmbString* sb_build(int64_t handle) {
    return bmb_sb_build(handle);
}

int64_t sb_len(int64_t handle) {
    return bmb_sb_len(handle);
}

int64_t sb_clear(int64_t handle) {
    return bmb_sb_clear(handle);
}

int64_t sb_contains(int64_t handle, const BmbString* marker) {
    return bmb_sb_contains(handle, marker);
}

int64_t sb_println(int64_t handle) {
    return bmb_sb_println(handle);
}

int64_t puts_cstr(const char* s) {
    return bmb_puts_cstr(s);
}

// v0.51.51: File I/O wrappers updated for BmbString*
BmbString* read_file(const BmbString* path) {
    return bmb_read_file(path);
}

int64_t write_file(const BmbString* path, const BmbString* content) {
    return bmb_write_file(path, content);
}

// v0.60.80: write_file_newlines wrapper for bootstrap compiler
int64_t write_file_newlines(const BmbString* path, const BmbString* content) {
    return bmb_write_file_newlines(path, content);
}

// v0.50.20: Argument wrappers
int64_t arg_count(void) {
    return bmb_arg_count();
}

BmbString* get_arg(int64_t index) {
    return bmb_get_arg(index);
}

// v0.50.20: Math wrappers (note: abs conflicts with stdlib, named differently)
// v0.51.46: Windows defines min/max as macros, undef them
#ifdef min
#undef min
#endif
#ifdef max
#undef max
#endif
int64_t min(int64_t a, int64_t b) {
    return bmb_min(a, b);
}

int64_t max(int64_t a, int64_t b) {
    return bmb_max(a, b);
}

// v0.50.36: find_close_paren moved to BMB (bootstrap/bmb_unified_cli.bmb)
// This function is no longer needed in the runtime

// ============================================================================
// v0.70: Threading Support
// ============================================================================

// Thread context structure for passing data to spawned threads
typedef struct {
    int64_t (*func)(void*);  // Function pointer to execute
    void* captures;           // Captured variables (packed struct)
    int64_t result;           // Thread result value
#ifdef _WIN32
    HANDLE thread_handle;     // Windows thread handle
#else
    pthread_t thread_id;      // POSIX thread ID
#endif
} BmbThreadContext;

#ifdef _WIN32
// Windows threading implementation using CreateThread

static DWORD WINAPI bmb_thread_entry_win32(LPVOID arg) {
    BmbThreadContext* ctx = (BmbThreadContext*)arg;
    // Call the wrapper function with captures
    if (ctx->func) {
        ctx->result = ctx->func(ctx->captures);
    } else {
        ctx->result = 0;
    }
    return 0;
}

int64_t bmb_spawn(int64_t (*func)(void*), void* captures) {
    BmbThreadContext* ctx = (BmbThreadContext*)malloc(sizeof(BmbThreadContext));
    if (!ctx) return 0;

    ctx->func = func;
    ctx->captures = captures;
    ctx->result = 0;
    ctx->thread_handle = NULL;

    ctx->thread_handle = CreateThread(NULL, 0, bmb_thread_entry_win32, ctx, 0, NULL);
    if (ctx->thread_handle == NULL) {
        free(ctx);
        return 0;
    }

    // Return context pointer as handle (context contains thread handle)
    return (int64_t)(uintptr_t)ctx;
}

int64_t bmb_join(int64_t thread_handle) {
    // Phase 1 fallback: if handle is a small value (not a valid pointer),
    // it's the result itself from synchronous execution
    // Pointers on x64 are typically > 0x10000, so we use that as threshold
    if (thread_handle >= 0 && thread_handle < 0x10000) {
        return thread_handle;  // Phase 1: handle IS the result
    }
    if (thread_handle < 0) {
        return thread_handle;  // Also treat negative values as direct results
    }

    BmbThreadContext* ctx = (BmbThreadContext*)(uintptr_t)thread_handle;

    if (ctx->thread_handle) {
        WaitForSingleObject(ctx->thread_handle, INFINITE);
        CloseHandle(ctx->thread_handle);
    }

    int64_t result = ctx->result;
    free(ctx);

    return result;
}

#else
// POSIX threading implementation using pthreads

static void* bmb_thread_entry_posix(void* arg) {
    BmbThreadContext* ctx = (BmbThreadContext*)arg;
    // Call the wrapper function with captures
    if (ctx->func) {
        ctx->result = ctx->func(ctx->captures);
    } else {
        ctx->result = 0;
    }
    return &ctx->result;
}

int64_t bmb_spawn(int64_t (*func)(void*), void* captures) {
    BmbThreadContext* ctx = (BmbThreadContext*)malloc(sizeof(BmbThreadContext));
    if (!ctx) return 0;
    ctx->func = func;
    ctx->captures = captures;
    ctx->result = 0;

    if (pthread_create(&ctx->thread_id, NULL, bmb_thread_entry_posix, ctx) != 0) {
        free(ctx);
        return 0;
    }

    // Return context pointer as handle (context contains thread ID)
    return (int64_t)(uintptr_t)ctx;
}

int64_t bmb_join(int64_t thread_handle) {
    // Phase 1 fallback: if handle is a small value (not a valid pointer),
    // it's the result itself from synchronous execution
    // Pointers on x64 are typically > 0x10000, so we use that as threshold
    if (thread_handle >= 0 && thread_handle < 0x10000) {
        return thread_handle;  // Phase 1: handle IS the result
    }
    if (thread_handle < 0) {
        return thread_handle;  // Also treat negative values as direct results
    }

    BmbThreadContext* ctx = (BmbThreadContext*)(uintptr_t)thread_handle;
    void* retval = NULL;

    pthread_join(ctx->thread_id, &retval);

    int64_t result = ctx->result;
    free(ctx);

    return result;
}

#endif

// ============================================================================
// v0.71: Mutex Support
// ============================================================================

// Mutex structure: contains platform-specific lock and stored value
typedef struct {
#ifdef _WIN32
    CRITICAL_SECTION lock;
#else
    pthread_mutex_t lock;
#endif
    int64_t data;  // Wrapped value
} BmbMutex;

// Create a new mutex with initial value
int64_t bmb_mutex_new(int64_t initial_value) {
    BmbMutex* m = (BmbMutex*)malloc(sizeof(BmbMutex));
    if (!m) return 0;

#ifdef _WIN32
    InitializeCriticalSection(&m->lock);
#else
    pthread_mutex_init(&m->lock, NULL);
#endif
    m->data = initial_value;

    return (int64_t)m;
}

// Lock the mutex and return the current value
int64_t bmb_mutex_lock(int64_t handle) {
    if (handle == 0) return 0;

    BmbMutex* m = (BmbMutex*)handle;

#ifdef _WIN32
    EnterCriticalSection(&m->lock);
#else
    pthread_mutex_lock(&m->lock);
#endif

    return m->data;
}

// Unlock the mutex and update the stored value
void bmb_mutex_unlock(int64_t handle, int64_t new_value) {
    if (handle == 0) return;

    BmbMutex* m = (BmbMutex*)handle;
    m->data = new_value;

#ifdef _WIN32
    LeaveCriticalSection(&m->lock);
#else
    pthread_mutex_unlock(&m->lock);
#endif
}

// Try to lock the mutex (non-blocking)
// Returns the current value if lock acquired, 0 if contended
// Note: Caller should check if return is meaningful vs just 0 value
int64_t bmb_mutex_try_lock(int64_t handle) {
    if (handle == 0) return 0;

    BmbMutex* m = (BmbMutex*)handle;

#ifdef _WIN32
    if (TryEnterCriticalSection(&m->lock)) {
        return m->data;
    }
#else
    if (pthread_mutex_trylock(&m->lock) == 0) {
        return m->data;
    }
#endif

    return 0;  // Lock failed
}

// Free a mutex
void bmb_mutex_free(int64_t handle) {
    if (handle == 0) return;

    BmbMutex* m = (BmbMutex*)handle;

#ifdef _WIN32
    DeleteCriticalSection(&m->lock);
#else
    pthread_mutex_destroy(&m->lock);
#endif

    free(m);
}

// ============================================================================
// v0.72: Arc Support (Atomic Reference Counting)
// ============================================================================

// Arc structure: [refcount, data]
typedef struct {
    volatile int64_t refcount;  // Atomic reference count
    int64_t data;               // Wrapped value
} BmbArcInner;

// Create a new Arc with initial value
int64_t bmb_arc_new(int64_t value) {
    BmbArcInner* inner = (BmbArcInner*)malloc(sizeof(BmbArcInner));
    if (!inner) return 0;

    inner->refcount = 1;
    inner->data = value;

    return (int64_t)inner;
}

// Clone an Arc (increment reference count)
int64_t bmb_arc_clone(int64_t handle) {
    if (handle == 0) return 0;

    BmbArcInner* inner = (BmbArcInner*)handle;

#ifdef _WIN32
    InterlockedIncrement64(&inner->refcount);
#else
    __sync_fetch_and_add(&inner->refcount, 1);
#endif

    return handle;  // Return same pointer
}

// Get the value stored in an Arc
int64_t bmb_arc_get(int64_t handle) {
    if (handle == 0) return 0;

    BmbArcInner* inner = (BmbArcInner*)handle;
    return inner->data;
}

// Drop an Arc (decrement reference count, free if zero)
void bmb_arc_drop(int64_t handle) {
    if (handle == 0) return;

    BmbArcInner* inner = (BmbArcInner*)handle;

#ifdef _WIN32
    if (InterlockedDecrement64(&inner->refcount) == 0) {
        free(inner);
    }
#else
    if (__sync_sub_and_fetch(&inner->refcount, 1) == 0) {
        free(inner);
    }
#endif
}

// Get the strong count of an Arc
int64_t bmb_arc_strong_count(int64_t handle) {
    if (handle == 0) return 0;

    BmbArcInner* inner = (BmbArcInner*)handle;

#ifdef _WIN32
    return InterlockedCompareExchange64(&inner->refcount, 0, 0);
#else
    return __sync_fetch_and_add(&inner->refcount, 0);
#endif
}

// ============================================================================
// v0.73: Channel support (MPSC - Multiple Producer, Single Consumer)
// ============================================================================

typedef struct {
    int64_t* buffer;           // Ring buffer
    int64_t capacity;
    volatile int64_t head;     // Write position
    volatile int64_t tail;     // Read position
    volatile int64_t count;    // Number of items
    volatile int64_t sender_count;  // Number of senders
    volatile int64_t closed;   // 1 if closed
#ifdef _WIN32
    CRITICAL_SECTION lock;
    CONDITION_VARIABLE not_empty;
    CONDITION_VARIABLE not_full;
#else
    pthread_mutex_t lock;
    pthread_cond_t not_empty;
    pthread_cond_t not_full;
#endif
} BmbChannel;

// Sender is just a pointer to channel
typedef struct {
    BmbChannel* channel;
} BmbSender;

// Receiver is unique owner
typedef struct {
    BmbChannel* channel;
} BmbReceiver;

// Create a new channel with specified capacity
void bmb_channel_new(int64_t capacity, int64_t* sender_out, int64_t* receiver_out) {
    BmbChannel* ch = (BmbChannel*)malloc(sizeof(BmbChannel));
    ch->buffer = (int64_t*)malloc(capacity * sizeof(int64_t));
    ch->capacity = capacity;
    ch->head = 0;
    ch->tail = 0;
    ch->count = 0;
    ch->sender_count = 1;
    ch->closed = 0;

#ifdef _WIN32
    InitializeCriticalSection(&ch->lock);
    InitializeConditionVariable(&ch->not_empty);
    InitializeConditionVariable(&ch->not_full);
#else
    pthread_mutex_init(&ch->lock, NULL);
    pthread_cond_init(&ch->not_empty, NULL);
    pthread_cond_init(&ch->not_full, NULL);
#endif

    BmbSender* sender = (BmbSender*)malloc(sizeof(BmbSender));
    sender->channel = ch;

    BmbReceiver* receiver = (BmbReceiver*)malloc(sizeof(BmbReceiver));
    receiver->channel = ch;

    *sender_out = (int64_t)sender;
    *receiver_out = (int64_t)receiver;
}

// Send a value (blocking if full)
void bmb_channel_send(int64_t sender_handle, int64_t value) {
    BmbSender* sender = (BmbSender*)sender_handle;
    BmbChannel* ch = sender->channel;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
    while (ch->count == ch->capacity && !ch->closed) {
        SleepConditionVariableCS(&ch->not_full, &ch->lock, INFINITE);
    }
#else
    pthread_mutex_lock(&ch->lock);
    while (ch->count == ch->capacity && !ch->closed) {
        pthread_cond_wait(&ch->not_full, &ch->lock);
    }
#endif

    if (!ch->closed) {
        ch->buffer[ch->head] = value;
        ch->head = (ch->head + 1) % ch->capacity;
        ch->count++;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_empty);
#else
        pthread_cond_signal(&ch->not_empty);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif
}

// Receive a value (blocking if empty)
int64_t bmb_channel_recv(int64_t receiver_handle) {
    BmbReceiver* receiver = (BmbReceiver*)receiver_handle;
    BmbChannel* ch = receiver->channel;
    int64_t value = 0;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
    while (ch->count == 0 && !ch->closed) {
        SleepConditionVariableCS(&ch->not_empty, &ch->lock, INFINITE);
    }
#else
    pthread_mutex_lock(&ch->lock);
    while (ch->count == 0 && !ch->closed) {
        pthread_cond_wait(&ch->not_empty, &ch->lock);
    }
#endif

    if (ch->count > 0) {
        value = ch->buffer[ch->tail];
        ch->tail = (ch->tail + 1) % ch->capacity;
        ch->count--;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_full);
#else
        pthread_cond_signal(&ch->not_full);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif

    return value;
}

// Try to send (non-blocking)
int64_t bmb_channel_try_send(int64_t sender_handle, int64_t value) {
    BmbSender* sender = (BmbSender*)sender_handle;
    BmbChannel* ch = sender->channel;
    int64_t success = 0;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
#else
    pthread_mutex_lock(&ch->lock);
#endif

    if (ch->count < ch->capacity && !ch->closed) {
        ch->buffer[ch->head] = value;
        ch->head = (ch->head + 1) % ch->capacity;
        ch->count++;
        success = 1;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_empty);
#else
        pthread_cond_signal(&ch->not_empty);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif

    return success;
}

// Try to receive (non-blocking)
int64_t bmb_channel_try_recv(int64_t receiver_handle, int64_t* value_out) {
    BmbReceiver* receiver = (BmbReceiver*)receiver_handle;
    BmbChannel* ch = receiver->channel;
    int64_t success = 0;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
#else
    pthread_mutex_lock(&ch->lock);
#endif

    if (ch->count > 0) {
        *value_out = ch->buffer[ch->tail];
        ch->tail = (ch->tail + 1) % ch->capacity;
        ch->count--;
        success = 1;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_full);
#else
        pthread_cond_signal(&ch->not_full);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif

    return success;
}

// v0.77: Receive with timeout (blocking up to timeout_ms milliseconds)
// Returns 1 if value received, 0 if timeout or closed
int64_t bmb_channel_recv_timeout(int64_t receiver_handle, int64_t timeout_ms, int64_t* value_out) {
    BmbReceiver* receiver = (BmbReceiver*)receiver_handle;
    BmbChannel* ch = receiver->channel;
    int64_t success = 0;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);

    // Wait with timeout
    DWORD wait_ms = (timeout_ms < 0) ? INFINITE : (DWORD)timeout_ms;
    while (ch->count == 0 && !ch->closed) {
        if (!SleepConditionVariableCS(&ch->not_empty, &ch->lock, wait_ms)) {
            // Timeout or error
            break;
        }
        // If we have a finite timeout, don't loop (could refine with remaining time)
        if (timeout_ms >= 0) break;
    }
#else
    pthread_mutex_lock(&ch->lock);

    if (timeout_ms < 0) {
        // Infinite wait
        while (ch->count == 0 && !ch->closed) {
            pthread_cond_wait(&ch->not_empty, &ch->lock);
        }
    } else {
        // Timed wait
        struct timespec ts;
        clock_gettime(CLOCK_REALTIME, &ts);
        ts.tv_sec += timeout_ms / 1000;
        ts.tv_nsec += (timeout_ms % 1000) * 1000000;
        if (ts.tv_nsec >= 1000000000) {
            ts.tv_sec++;
            ts.tv_nsec -= 1000000000;
        }

        while (ch->count == 0 && !ch->closed) {
            int rc = pthread_cond_timedwait(&ch->not_empty, &ch->lock, &ts);
            if (rc == ETIMEDOUT) break;
        }
    }
#endif

    if (ch->count > 0) {
        *value_out = ch->buffer[ch->tail];
        ch->tail = (ch->tail + 1) % ch->capacity;
        ch->count--;
        success = 1;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_full);
#else
        pthread_cond_signal(&ch->not_full);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif

    return success;
}

// v0.79: Send with timeout (blocking up to timeout_ms milliseconds)
// Returns 1 if value sent, 0 if timeout or closed
int64_t bmb_channel_send_timeout(int64_t sender_handle, int64_t value, int64_t timeout_ms) {
    BmbSender* sender = (BmbSender*)sender_handle;
    BmbChannel* ch = sender->channel;
    int64_t success = 0;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);

    // Wait with timeout for space in buffer
    DWORD wait_ms = (timeout_ms < 0) ? INFINITE : (DWORD)timeout_ms;
    while (ch->count == ch->capacity && !ch->closed) {
        if (!SleepConditionVariableCS(&ch->not_full, &ch->lock, wait_ms)) {
            // Timeout or error
            break;
        }
        // If we have a finite timeout, don't loop (could refine with remaining time)
        if (timeout_ms >= 0) break;
    }
#else
    pthread_mutex_lock(&ch->lock);

    if (timeout_ms < 0) {
        // Infinite wait
        while (ch->count == ch->capacity && !ch->closed) {
            pthread_cond_wait(&ch->not_full, &ch->lock);
        }
    } else {
        // Timed wait
        struct timespec ts;
        clock_gettime(CLOCK_REALTIME, &ts);
        ts.tv_sec += timeout_ms / 1000;
        ts.tv_nsec += (timeout_ms % 1000) * 1000000;
        if (ts.tv_nsec >= 1000000000) {
            ts.tv_sec++;
            ts.tv_nsec -= 1000000000;
        }

        while (ch->count == ch->capacity && !ch->closed) {
            int rc = pthread_cond_timedwait(&ch->not_full, &ch->lock, &ts);
            if (rc == ETIMEDOUT) break;
        }
    }
#endif

    // Try to send if there's space
    if (ch->count < ch->capacity && !ch->closed) {
        ch->buffer[ch->head] = value;
        ch->head = (ch->head + 1) % ch->capacity;
        ch->count++;
        success = 1;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_empty);
#else
        pthread_cond_signal(&ch->not_empty);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif

    return success;
}

// Clone a sender (increment sender count)
int64_t bmb_sender_clone(int64_t sender_handle) {
    BmbSender* sender = (BmbSender*)sender_handle;
    BmbChannel* ch = sender->channel;

#ifdef _WIN32
    InterlockedIncrement64(&ch->sender_count);
#else
    __sync_fetch_and_add(&ch->sender_count, 1);
#endif

    BmbSender* new_sender = (BmbSender*)malloc(sizeof(BmbSender));
    new_sender->channel = ch;
    return (int64_t)new_sender;
}

// v0.80: Close a channel (signal no more values will be sent)
void bmb_channel_close(int64_t sender_handle) {
    BmbSender* sender = (BmbSender*)sender_handle;
    BmbChannel* ch = sender->channel;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
    ch->closed = 1;
    // Wake all waiting receivers
    WakeAllConditionVariable(&ch->not_empty);
    // Wake all waiting senders
    WakeAllConditionVariable(&ch->not_full);
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_lock(&ch->lock);
    ch->closed = 1;
    pthread_cond_broadcast(&ch->not_empty);
    pthread_cond_broadcast(&ch->not_full);
    pthread_mutex_unlock(&ch->lock);
#endif
}

// v0.80: Check if channel is closed
int64_t bmb_channel_is_closed(int64_t receiver_handle) {
    BmbReceiver* receiver = (BmbReceiver*)receiver_handle;
    BmbChannel* ch = receiver->channel;
    int64_t closed;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
    closed = ch->closed;
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_lock(&ch->lock);
    closed = ch->closed;
    pthread_mutex_unlock(&ch->lock);
#endif

    return closed;
}

// v0.80: Receive with closed awareness (returns 1 if value received, 0 if closed and empty)
int64_t bmb_channel_recv_opt(int64_t receiver_handle, int64_t* value_out) {
    BmbReceiver* receiver = (BmbReceiver*)receiver_handle;
    BmbChannel* ch = receiver->channel;
    int64_t success = 0;

#ifdef _WIN32
    EnterCriticalSection(&ch->lock);
    while (ch->count == 0 && !ch->closed) {
        SleepConditionVariableCS(&ch->not_empty, &ch->lock, INFINITE);
    }
#else
    pthread_mutex_lock(&ch->lock);
    while (ch->count == 0 && !ch->closed) {
        pthread_cond_wait(&ch->not_empty, &ch->lock);
    }
#endif

    if (ch->count > 0) {
        *value_out = ch->buffer[ch->tail];
        ch->tail = (ch->tail + 1) % ch->capacity;
        ch->count--;
        success = 1;

#ifdef _WIN32
        WakeConditionVariable(&ch->not_full);
#else
        pthread_cond_signal(&ch->not_full);
#endif
    }

#ifdef _WIN32
    LeaveCriticalSection(&ch->lock);
#else
    pthread_mutex_unlock(&ch->lock);
#endif

    return success;
}

// ============================================================================
// v0.74: RwLock Support (Reader-Writer Lock)
// ============================================================================

typedef struct {
#ifdef _WIN32
    SRWLOCK lock;
#else
    pthread_rwlock_t lock;
#endif
    int64_t data;  // Wrapped value
} BmbRwLock;

// Create a new RwLock with initial value
int64_t bmb_rwlock_new(int64_t initial_value) {
    BmbRwLock* rw = (BmbRwLock*)malloc(sizeof(BmbRwLock));
    if (!rw) return 0;

#ifdef _WIN32
    InitializeSRWLock(&rw->lock);
#else
    pthread_rwlock_init(&rw->lock, NULL);
#endif
    rw->data = initial_value;

    return (int64_t)rw;
}

// Acquire read lock and return the current value
int64_t bmb_rwlock_read(int64_t handle) {
    if (handle == 0) return 0;
    BmbRwLock* rw = (BmbRwLock*)handle;

#ifdef _WIN32
    AcquireSRWLockShared(&rw->lock);
#else
    pthread_rwlock_rdlock(&rw->lock);
#endif

    return rw->data;
}

// Release read lock
void bmb_rwlock_read_unlock(int64_t handle) {
    if (handle == 0) return;
    BmbRwLock* rw = (BmbRwLock*)handle;

#ifdef _WIN32
    ReleaseSRWLockShared(&rw->lock);
#else
    pthread_rwlock_unlock(&rw->lock);
#endif
}

// Acquire write lock and return the current value
int64_t bmb_rwlock_write(int64_t handle) {
    if (handle == 0) return 0;
    BmbRwLock* rw = (BmbRwLock*)handle;

#ifdef _WIN32
    AcquireSRWLockExclusive(&rw->lock);
#else
    pthread_rwlock_wrlock(&rw->lock);
#endif

    return rw->data;
}

// Release write lock and update value
void bmb_rwlock_write_unlock(int64_t handle, int64_t new_value) {
    if (handle == 0) return;
    BmbRwLock* rw = (BmbRwLock*)handle;
    rw->data = new_value;

#ifdef _WIN32
    ReleaseSRWLockExclusive(&rw->lock);
#else
    pthread_rwlock_unlock(&rw->lock);
#endif
}

// Try to acquire read lock (non-blocking)
int64_t bmb_rwlock_try_read(int64_t handle) {
    if (handle == 0) return -1;  // -1 indicates failure
    BmbRwLock* rw = (BmbRwLock*)handle;

#ifdef _WIN32
    if (TryAcquireSRWLockShared(&rw->lock)) {
        return rw->data;
    }
#else
    if (pthread_rwlock_tryrdlock(&rw->lock) == 0) {
        return rw->data;
    }
#endif

    return -1;  // Lock failed (caller should distinguish -1 from valid data)
}

// Try to acquire write lock (non-blocking)
int64_t bmb_rwlock_try_write(int64_t handle) {
    if (handle == 0) return -1;
    BmbRwLock* rw = (BmbRwLock*)handle;

#ifdef _WIN32
    if (TryAcquireSRWLockExclusive(&rw->lock)) {
        return rw->data;
    }
#else
    if (pthread_rwlock_trywrlock(&rw->lock) == 0) {
        return rw->data;
    }
#endif

    return -1;
}

// Free a RwLock
void bmb_rwlock_free(int64_t handle) {
    if (handle == 0) return;
    BmbRwLock* rw = (BmbRwLock*)handle;

#ifdef _WIN32
    // SRWLock doesn't need explicit destruction on Windows
#else
    pthread_rwlock_destroy(&rw->lock);
#endif

    free(rw);
}

// ============================================================================
// v0.74: Barrier Support (Thread Synchronization Barrier)
// ============================================================================

typedef struct {
#ifdef _WIN32
    CRITICAL_SECTION lock;
    CONDITION_VARIABLE cond;
    int64_t threshold;
    int64_t count;
    int64_t generation;
#else
    pthread_mutex_t lock;
    pthread_cond_t cond;
    int64_t threshold;
    int64_t count;
    int64_t generation;
#endif
} BmbBarrier;

// Create a new barrier for N threads
int64_t bmb_barrier_new(int64_t count) {
    if (count <= 0) return 0;
    BmbBarrier* b = (BmbBarrier*)malloc(sizeof(BmbBarrier));
    if (!b) return 0;

#ifdef _WIN32
    InitializeCriticalSection(&b->lock);
    InitializeConditionVariable(&b->cond);
#else
    pthread_mutex_init(&b->lock, NULL);
    pthread_cond_init(&b->cond, NULL);
#endif
    b->threshold = count;
    b->count = 0;
    b->generation = 0;

    return (int64_t)b;
}

// Wait at barrier until all threads arrive
// Returns 1 for the last thread (leader), 0 for others
int64_t bmb_barrier_wait(int64_t handle) {
    if (handle == 0) return 0;
    BmbBarrier* b = (BmbBarrier*)handle;
    int64_t is_leader = 0;

#ifdef _WIN32
    EnterCriticalSection(&b->lock);
    int64_t gen = b->generation;
    b->count++;

    if (b->count == b->threshold) {
        // Last thread: reset and wake all
        b->count = 0;
        b->generation++;
        is_leader = 1;
        WakeAllConditionVariable(&b->cond);
    } else {
        // Wait for generation to change
        while (gen == b->generation) {
            SleepConditionVariableCS(&b->cond, &b->lock, INFINITE);
        }
    }
    LeaveCriticalSection(&b->lock);
#else
    pthread_mutex_lock(&b->lock);
    int64_t gen = b->generation;
    b->count++;

    if (b->count == b->threshold) {
        b->count = 0;
        b->generation++;
        is_leader = 1;
        pthread_cond_broadcast(&b->cond);
    } else {
        while (gen == b->generation) {
            pthread_cond_wait(&b->cond, &b->lock);
        }
    }
    pthread_mutex_unlock(&b->lock);
#endif

    return is_leader;
}

// Free a barrier
void bmb_barrier_free(int64_t handle) {
    if (handle == 0) return;
    BmbBarrier* b = (BmbBarrier*)handle;

#ifdef _WIN32
    DeleteCriticalSection(&b->lock);
#else
    pthread_mutex_destroy(&b->lock);
    pthread_cond_destroy(&b->cond);
#endif

    free(b);
}

// ============================================================================
// v0.74: Condvar Support (Condition Variable)
// ============================================================================

typedef struct {
#ifdef _WIN32
    CONDITION_VARIABLE cond;
#else
    pthread_cond_t cond;
#endif
} BmbCondvar;

// Create a new condition variable
int64_t bmb_condvar_new(void) {
    BmbCondvar* cv = (BmbCondvar*)malloc(sizeof(BmbCondvar));
    if (!cv) return 0;

#ifdef _WIN32
    InitializeConditionVariable(&cv->cond);
#else
    pthread_cond_init(&cv->cond, NULL);
#endif

    return (int64_t)cv;
}

// Wait on condition variable (mutex must be locked)
// Returns the mutex's current value after waking
int64_t bmb_condvar_wait(int64_t cv_handle, int64_t mutex_handle) {
    if (cv_handle == 0 || mutex_handle == 0) return 0;
    BmbCondvar* cv = (BmbCondvar*)cv_handle;
    BmbMutex* m = (BmbMutex*)mutex_handle;

#ifdef _WIN32
    SleepConditionVariableCS(&cv->cond, &m->lock, INFINITE);
#else
    pthread_cond_wait(&cv->cond, &m->lock);
#endif

    return m->data;
}

// Wake one waiting thread
void bmb_condvar_notify_one(int64_t handle) {
    if (handle == 0) return;
    BmbCondvar* cv = (BmbCondvar*)handle;

#ifdef _WIN32
    WakeConditionVariable(&cv->cond);
#else
    pthread_cond_signal(&cv->cond);
#endif
}

// Wake all waiting threads
void bmb_condvar_notify_all(int64_t handle) {
    if (handle == 0) return;
    BmbCondvar* cv = (BmbCondvar*)handle;

#ifdef _WIN32
    WakeAllConditionVariable(&cv->cond);
#else
    pthread_cond_broadcast(&cv->cond);
#endif
}

// Free a condition variable
void bmb_condvar_free(int64_t handle) {
    if (handle == 0) return;
    BmbCondvar* cv = (BmbCondvar*)handle;

#ifdef _WIN32
    // Windows condition variables don't need explicit destruction
#else
    pthread_cond_destroy(&cv->cond);
#endif

    free(cv);
}

// ============================================================================
// v0.75: Async/Await Support (Futures)
// ============================================================================
//
// For Phase v0.75.2, we implement a simple synchronous execution model:
// - Future<T> is represented as T at runtime (no wrapper needed)
// - async fn just executes synchronously and returns its result
// - .await is a no-op (identity function)
//
// This allows the type system's async/await to work while keeping
// runtime simple. A full async executor can be added later.

// __future_await: Block until future completes and return its value
// In the synchronous model, futures are already complete, so this is identity
int64_t __future_await(int64_t future_handle) {
    // In synchronous mode, the future_handle IS the result
    // A proper async runtime would poll the future here
    return future_handle;
}

// Alias without underscore prefix for easier linking
int64_t future_await(int64_t future_handle) {
    return __future_await(future_handle);
}

// ============================================================================
// v0.78: Async Executor
// ============================================================================
//
// A minimal task-based executor for running async computations.
// In the current synchronous model, tasks complete immediately.
// This provides the API foundation for true async execution later.

#define TASK_PENDING 0
#define TASK_RUNNING 1
#define TASK_COMPLETED 2

typedef struct BmbTask {
    int64_t result;         // The computed result
    int32_t state;          // PENDING, RUNNING, COMPLETED
    struct BmbTask* next;   // Next task in queue
} BmbTask;

typedef struct BmbExecutor {
    BmbTask* queue_head;
    BmbTask* queue_tail;
    int64_t task_count;
    int64_t completed_count;
#ifdef _WIN32
    CRITICAL_SECTION lock;
#else
    pthread_mutex_t lock;
#endif
} BmbExecutor;

// Create a new executor
int64_t bmb_executor_new(void) {
    BmbExecutor* exec = (BmbExecutor*)malloc(sizeof(BmbExecutor));
    if (!exec) return 0;

    exec->queue_head = NULL;
    exec->queue_tail = NULL;
    exec->task_count = 0;
    exec->completed_count = 0;

#ifdef _WIN32
    InitializeCriticalSection(&exec->lock);
#else
    pthread_mutex_init(&exec->lock, NULL);
#endif

    return (int64_t)exec;
}

// Create a new task from a future value
int64_t bmb_task_new(int64_t future_value) {
    BmbTask* task = (BmbTask*)malloc(sizeof(BmbTask));
    if (!task) return 0;

    // In synchronous mode, the future_value IS the result
    task->result = future_value;
    task->state = TASK_COMPLETED;  // Already complete in sync mode
    task->next = NULL;

    return (int64_t)task;
}

// Spawn a task onto the executor
void bmb_executor_spawn(int64_t executor_handle, int64_t task_handle) {
    BmbExecutor* exec = (BmbExecutor*)executor_handle;
    BmbTask* task = (BmbTask*)task_handle;
    if (!exec || !task) return;

#ifdef _WIN32
    EnterCriticalSection(&exec->lock);
#else
    pthread_mutex_lock(&exec->lock);
#endif

    // Add to queue tail
    task->next = NULL;
    if (exec->queue_tail) {
        exec->queue_tail->next = task;
    } else {
        exec->queue_head = task;
    }
    exec->queue_tail = task;
    exec->task_count++;

    // In sync mode, tasks are already completed
    if (task->state == TASK_COMPLETED) {
        exec->completed_count++;
    }

#ifdef _WIN32
    LeaveCriticalSection(&exec->lock);
#else
    pthread_mutex_unlock(&exec->lock);
#endif
}

// Run executor until all tasks complete
void bmb_executor_run(int64_t executor_handle) {
    BmbExecutor* exec = (BmbExecutor*)executor_handle;
    if (!exec) return;

    // v0.101: Integrated event loop - poll for I/O events before completing tasks
    // This allows async I/O operations registered with the event loop to make progress

    int max_iterations = 100;  // Safety limit to prevent infinite loops
    int iteration = 0;

    while (iteration < max_iterations) {
        // Poll event loop for pending I/O (non-blocking, 0ms timeout)
        BmbEventLoop* loop = bmb_get_event_loop();
        if (loop) {
            bmb_event_loop_run_once(loop, 0);
        }

#ifdef _WIN32
        EnterCriticalSection(&exec->lock);
#else
        pthread_mutex_lock(&exec->lock);
#endif

        // Check and complete pending tasks
        int has_pending = 0;
        BmbTask* task = exec->queue_head;
        while (task) {
            if (task->state == TASK_PENDING) {
                task->state = TASK_COMPLETED;
                exec->completed_count++;
            }
            task = task->next;
        }

#ifdef _WIN32
        LeaveCriticalSection(&exec->lock);
#else
        pthread_mutex_unlock(&exec->lock);
#endif

        // All tasks completed on first pass in synchronous mode
        break;
    }
}

// Block on a specific future, return its result
int64_t bmb_executor_block_on(int64_t executor_handle, int64_t future_value) {
    BmbExecutor* exec = (BmbExecutor*)executor_handle;
    if (!exec) return future_value;

    // Create task for the future
    int64_t task = bmb_task_new(future_value);
    bmb_executor_spawn(executor_handle, task);

    // Run executor
    bmb_executor_run(executor_handle);

    // Get result (in sync mode, future_value IS the result)
    int64_t result = ((BmbTask*)task)->result;

    return result;
}

// Get task result
int64_t bmb_task_get_result(int64_t task_handle) {
    BmbTask* task = (BmbTask*)task_handle;
    if (!task) return 0;
    return task->result;
}

// Check if task is completed (returns 1 if done, 0 if pending)
int64_t bmb_task_is_completed(int64_t task_handle) {
    BmbTask* task = (BmbTask*)task_handle;
    if (!task) return 1;
    return task->state == TASK_COMPLETED ? 1 : 0;
}

// Get number of completed tasks
int64_t bmb_executor_completed_count(int64_t executor_handle) {
    BmbExecutor* exec = (BmbExecutor*)executor_handle;
    if (!exec) return 0;
    return exec->completed_count;
}

// Free a task
void bmb_task_free(int64_t task_handle) {
    BmbTask* task = (BmbTask*)task_handle;
    if (task) free(task);
}

// Free executor and all its tasks
void bmb_executor_free(int64_t executor_handle) {
    BmbExecutor* exec = (BmbExecutor*)executor_handle;
    if (!exec) return;

    // Free all tasks
    BmbTask* task = exec->queue_head;
    while (task) {
        BmbTask* next = task->next;
        free(task);
        task = next;
    }

#ifdef _WIN32
    DeleteCriticalSection(&exec->lock);
#else
    pthread_mutex_destroy(&exec->lock);
#endif

    free(exec);
}

// Convenience: Create executor, block_on future, return result, free executor
// v0.101: Polls event loop to allow async I/O to complete while waiting
int64_t bmb_block_on(int64_t future_value) {
    // Ensure event loop exists for any pending async operations
    bmb_get_event_loop();

    int64_t exec = bmb_executor_new();
    int64_t result = bmb_executor_block_on(exec, future_value);
    bmb_executor_free(exec);
    return result;
}

// ============================================================================
// v0.60.246: String-key HashMap for O(1) lookups in bootstrap compiler
// ============================================================================

#define STRMAP_INITIAL_CAPACITY 64
#define STRMAP_LOAD_FACTOR 0.75

typedef struct StrMapEntry {
    char* key;
    int64_t value;
    struct StrMapEntry* next;
} StrMapEntry;

typedef struct {
    StrMapEntry** buckets;
    int64_t capacity;
    int64_t size;
} StrMap;

// FNV-1a hash function for string keys
static uint64_t strmap_hash(const char* key, int64_t len) {
    uint64_t hash = 14695981039346656037ULL;
    for (int64_t i = 0; i < len; i++) {
        hash ^= (unsigned char)key[i];
        hash *= 1099511628211ULL;
    }
    return hash;
}

// Create new strmap
int64_t bmb_strmap_new(void) {
    StrMap* map = (StrMap*)malloc(sizeof(StrMap));
    map->capacity = STRMAP_INITIAL_CAPACITY;
    map->size = 0;
    map->buckets = (StrMapEntry**)calloc(map->capacity, sizeof(StrMapEntry*));
    return (int64_t)map;
}

// Free strmap
void bmb_strmap_free(int64_t handle) {
    if (!handle) return;
    StrMap* map = (StrMap*)handle;
    for (int64_t i = 0; i < map->capacity; i++) {
        StrMapEntry* entry = map->buckets[i];
        while (entry) {
            StrMapEntry* next = entry->next;
            free(entry->key);
            free(entry);
            entry = next;
        }
    }
    free(map->buckets);
    free(map);
}

// Resize strmap when load factor exceeded
static void strmap_resize(StrMap* map) {
    int64_t old_capacity = map->capacity;
    StrMapEntry** old_buckets = map->buckets;

    map->capacity *= 2;
    map->buckets = (StrMapEntry**)calloc(map->capacity, sizeof(StrMapEntry*));
    map->size = 0;

    for (int64_t i = 0; i < old_capacity; i++) {
        StrMapEntry* entry = old_buckets[i];
        while (entry) {
            StrMapEntry* next = entry->next;
            // Reinsert into new buckets
            uint64_t hash = strmap_hash(entry->key, strlen(entry->key));
            int64_t idx = hash % map->capacity;
            entry->next = map->buckets[idx];
            map->buckets[idx] = entry;
            map->size++;
            entry = next;
        }
    }
    free(old_buckets);
}

// Insert key-value pair (BmbString* key)
int64_t bmb_strmap_insert(int64_t handle, const BmbString* key, int64_t value) {
    if (!handle || !key || !key->data) return 0;
    StrMap* map = (StrMap*)handle;

    // Check load factor
    if ((double)map->size / map->capacity > STRMAP_LOAD_FACTOR) {
        strmap_resize(map);
    }

    uint64_t hash = strmap_hash(key->data, key->len);
    int64_t idx = hash % map->capacity;

    // Check if key exists
    StrMapEntry* entry = map->buckets[idx];
    while (entry) {
        if (strlen(entry->key) == (size_t)key->len &&
            memcmp(entry->key, key->data, key->len) == 0) {
            entry->value = value;  // Update existing
            return 1;
        }
        entry = entry->next;
    }

    // Insert new entry
    StrMapEntry* new_entry = (StrMapEntry*)malloc(sizeof(StrMapEntry));
    new_entry->key = (char*)malloc(key->len + 1);
    memcpy(new_entry->key, key->data, key->len);
    new_entry->key[key->len] = '\0';
    new_entry->value = value;
    new_entry->next = map->buckets[idx];
    map->buckets[idx] = new_entry;
    map->size++;

    return 1;
}

// Get value by key (returns -1 if not found)
int64_t bmb_strmap_get(int64_t handle, const BmbString* key) {
    if (!handle || !key || !key->data) return -1;
    StrMap* map = (StrMap*)handle;

    uint64_t hash = strmap_hash(key->data, key->len);
    int64_t idx = hash % map->capacity;

    StrMapEntry* entry = map->buckets[idx];
    while (entry) {
        if (strlen(entry->key) == (size_t)key->len &&
            memcmp(entry->key, key->data, key->len) == 0) {
            return entry->value;
        }
        entry = entry->next;
    }
    return -1;
}

// Check if key exists (returns 1 if found, 0 if not)
int64_t bmb_strmap_contains(int64_t handle, const BmbString* key) {
    if (!handle || !key || !key->data) return 0;
    StrMap* map = (StrMap*)handle;

    uint64_t hash = strmap_hash(key->data, key->len);
    int64_t idx = hash % map->capacity;

    StrMapEntry* entry = map->buckets[idx];
    while (entry) {
        if (strlen(entry->key) == (size_t)key->len &&
            memcmp(entry->key, key->data, key->len) == 0) {
            return 1;
        }
        entry = entry->next;
    }
    return 0;
}

// Get strmap size
int64_t bmb_strmap_size(int64_t handle) {
    if (!handle) return 0;
    StrMap* map = (StrMap*)handle;
    return map->size;
}

// Wrapper functions with strmap_ prefix
int64_t strmap_new(void) { return bmb_strmap_new(); }
void strmap_free(int64_t handle) { bmb_strmap_free(handle); }
int64_t strmap_insert(int64_t handle, const BmbString* key, int64_t value) {
    return bmb_strmap_insert(handle, key, value);
}
int64_t strmap_get(int64_t handle, const BmbString* key) {
    return bmb_strmap_get(handle, key);
}
int64_t strmap_contains(int64_t handle, const BmbString* key) {
    return bmb_strmap_contains(handle, key);
}
int64_t strmap_size(int64_t handle) { return bmb_strmap_size(handle); }

// ============================================================================
// v0.63: Timing functions for bmb-bench
// ============================================================================

#ifdef _WIN32
// Windows: Use QueryPerformanceCounter for high-resolution timing
int64_t bmb_time_ns(void) {
    static LARGE_INTEGER freq = {0};
    if (freq.QuadPart == 0) {
        QueryPerformanceFrequency(&freq);
    }
    LARGE_INTEGER counter;
    QueryPerformanceCounter(&counter);
    // Convert to nanoseconds
    return (int64_t)((counter.QuadPart * 1000000000LL) / freq.QuadPart);
}
#else
// Unix: Use clock_gettime for nanosecond precision
#include <time.h>
int64_t bmb_time_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)(ts.tv_sec * 1000000000LL + ts.tv_nsec);
}
#endif

// Alias for compatibility
int64_t time_ns(void) { return bmb_time_ns(); }

// ============================================================================
// v0.83: AsyncFile - Asynchronous File I/O
// ============================================================================
//
// For Phase v0.83, we implement a simple synchronous I/O model:
// - AsyncFile is represented as a FILE* handle
// - Operations complete synchronously (no async executor integration yet)
// - Returns Future<T> where T is the result (already complete)
//
// A proper async I/O implementation would use:
// - Windows: IOCP (I/O Completion Ports)
// - Linux: io_uring or epoll + thread pool
// - macOS: kqueue + thread pool

typedef struct {
    FILE* fp;
    char* path;
    int is_open;
} BmbAsyncFile;

// Open a file asynchronously (synchronous for now)
// Returns a Future<AsyncFile> handle (the AsyncFile handle itself)
int64_t bmb_async_file_open(int64_t path_handle) {
    if (path_handle == 0) return 0;

    const char* path = (const char*)path_handle;

    BmbAsyncFile* af = (BmbAsyncFile*)malloc(sizeof(BmbAsyncFile));
    if (!af) return 0;

    af->fp = fopen(path, "r+");
    if (!af->fp) {
        // Try read-only
        af->fp = fopen(path, "r");
    }
    if (!af->fp) {
        // Try create new
        af->fp = fopen(path, "w+");
    }

    if (!af->fp) {
        free(af);
        return 0;
    }

    af->path = strdup(path);
    af->is_open = 1;

    return (int64_t)af;
}

// Read file content asynchronously (synchronous for now)
// Returns a Future<String> handle (the string handle itself)
int64_t bmb_async_file_read(int64_t file_handle) {
    if (file_handle == 0) return 0;

    BmbAsyncFile* af = (BmbAsyncFile*)file_handle;
    if (!af->is_open || !af->fp) return 0;

    // Get file size
    fseek(af->fp, 0, SEEK_END);
    long size = ftell(af->fp);
    fseek(af->fp, 0, SEEK_SET);

    if (size < 0) return 0;

    char* content = (char*)malloc(size + 1);
    if (!content) return 0;

    size_t read = fread(content, 1, size, af->fp);
    content[read] = '\0';

    return (int64_t)content;
}

// Write content to file asynchronously (synchronous for now)
void bmb_async_file_write(int64_t file_handle, int64_t content_handle) {
    if (file_handle == 0 || content_handle == 0) return;

    BmbAsyncFile* af = (BmbAsyncFile*)file_handle;
    if (!af->is_open || !af->fp) return;

    const char* content = (const char*)content_handle;

    // Truncate and write from beginning
    fseek(af->fp, 0, SEEK_SET);
    fwrite(content, 1, strlen(content), af->fp);
    fflush(af->fp);
}

// Close file asynchronously (synchronous for now)
void bmb_async_file_close(int64_t file_handle) {
    if (file_handle == 0) return;

    BmbAsyncFile* af = (BmbAsyncFile*)file_handle;

    if (af->fp) {
        fclose(af->fp);
        af->fp = NULL;
    }

    if (af->path) {
        free(af->path);
        af->path = NULL;
    }

    af->is_open = 0;
    free(af);
}

// ============================================================================
// v0.100: Non-Blocking File I/O via Thread Pool
// ============================================================================
// File I/O can't be truly async via epoll/IOCP for regular files.
// Instead, we submit file operations to a background thread and signal
// completion via a volatile flag.

typedef struct {
    char* path;
    char* content;       // result for reads, input for writes
    int64_t size;        // result size for reads
    volatile int completed;
    volatile int success;
    int op;              // 0=open, 1=read, 2=write
    BmbAsyncFile* file;
} BmbFileTask;

// File task slot (global, for thread pool's void(void) interface)
// Simple approach: single pending task at a time per slot
#define BMB_MAX_FILE_TASKS 16
static BmbFileTask g_file_tasks[BMB_MAX_FILE_TASKS];
static volatile int g_file_task_count = 0;

static int alloc_file_task(void) {
    for (int i = 0; i < BMB_MAX_FILE_TASKS; i++) {
        if (g_file_tasks[i].completed && g_file_tasks[i].op == -1) {
            g_file_tasks[i].op = 0;
            g_file_tasks[i].completed = 0;
            g_file_tasks[i].success = 0;
            return i;
        }
    }
    // No free slot, use count
    if (g_file_task_count < BMB_MAX_FILE_TASKS) {
        int idx = g_file_task_count++;
        g_file_tasks[idx].op = 0;
        g_file_tasks[idx].completed = 0;
        g_file_tasks[idx].success = 0;
        return idx;
    }
    return -1;
}

// Worker functions for thread pool (void(void) interface)
// Uses global task slots indexed by function pointer encoding

// Non-blocking file read: submit to thread pool, return task handle
// Caller polls task.completed to check when done
int64_t bmb_nb_file_read(int64_t file_handle) {
    if (file_handle == 0) return 0;
    BmbAsyncFile* af = (BmbAsyncFile*)file_handle;
    if (!af->is_open || !af->fp) return 0;

    // For simplicity, do the read synchronously on the calling thread
    // but structured as a task that can be moved to a thread pool
    fseek(af->fp, 0, SEEK_END);
    long size = ftell(af->fp);
    fseek(af->fp, 0, SEEK_SET);

    if (size < 0) return 0;

    char* content = (char*)malloc(size + 1);
    if (!content) return 0;

    size_t read_bytes = fread(content, 1, size, af->fp);
    content[read_bytes] = '\0';

    return (int64_t)content;
}

// Non-blocking file write
void bmb_nb_file_write(int64_t file_handle, int64_t content_handle) {
    if (file_handle == 0 || content_handle == 0) return;
    BmbAsyncFile* af = (BmbAsyncFile*)file_handle;
    if (!af->is_open || !af->fp) return;

    const char* content = (const char*)content_handle;
    fseek(af->fp, 0, SEEK_SET);
    fwrite(content, 1, strlen(content), af->fp);
    fflush(af->fp);
}

// ============================================================================
// v0.83.1: AsyncSocket (TCP)
// ============================================================================
// Foundation for async network I/O.
// Current implementation: synchronous blocking sockets.
// Future: Platform-specific async I/O (IOCP on Windows, io_uring on Linux)

#ifdef _WIN32
// Windows uses WinSock (included at top before windows.h)
#pragma comment(lib, "ws2_32.lib")

typedef struct {
    SOCKET sock;
    char* host;
    int port;
    int is_connected;
} BmbAsyncSocket;

// Initialize Winsock once
static int g_winsock_initialized = 0;
static void ensure_winsock_init(void) {
    if (!g_winsock_initialized) {
        WSADATA wsa;
        WSAStartup(MAKEWORD(2, 2), &wsa);
        g_winsock_initialized = 1;
    }
}

// Connect to a TCP server
int64_t bmb_async_socket_connect(int64_t host_handle, int64_t port) {
    ensure_winsock_init();

    if (host_handle == 0) return 0;
    const char* host = (const char*)host_handle;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)malloc(sizeof(BmbAsyncSocket));
    if (!sock) return 0;

    sock->sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock->sock == INVALID_SOCKET) {
        free(sock);
        return 0;
    }

    struct sockaddr_in server;
    server.sin_family = AF_INET;
    server.sin_port = htons((unsigned short)port);

    // Try to parse as IP address first
    if (inet_pton(AF_INET, host, &server.sin_addr) != 1) {
        // If that fails, try DNS resolution
        struct addrinfo hints = {0};
        struct addrinfo* result = NULL;
        hints.ai_family = AF_INET;
        hints.ai_socktype = SOCK_STREAM;

        if (getaddrinfo(host, NULL, &hints, &result) != 0 || !result) {
            closesocket(sock->sock);
            free(sock);
            return 0;
        }

        struct sockaddr_in* addr = (struct sockaddr_in*)result->ai_addr;
        server.sin_addr = addr->sin_addr;
        freeaddrinfo(result);
    }

    if (connect(sock->sock, (struct sockaddr*)&server, sizeof(server)) == SOCKET_ERROR) {
        closesocket(sock->sock);
        free(sock);
        return 0;
    }

    sock->host = strdup(host);
    sock->port = (int)port;
    sock->is_connected = 1;

    return (int64_t)sock;
}

// Receive data from socket
int64_t bmb_async_socket_read(int64_t socket_handle) {
    if (socket_handle == 0) return 0;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return 0;

    // Read up to 4KB at a time
    char* buffer = (char*)malloc(4096);
    if (!buffer) return 0;

    int received = recv(sock->sock, buffer, 4095, 0);
    if (received <= 0) {
        free(buffer);
        return 0;
    }

    buffer[received] = '\0';
    return (int64_t)buffer;
}

// Send data to socket
void bmb_async_socket_write(int64_t socket_handle, int64_t content_handle) {
    if (socket_handle == 0 || content_handle == 0) return;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return;

    const char* content = (const char*)content_handle;
    send(sock->sock, content, (int)strlen(content), 0);
}

// Close socket
void bmb_async_socket_close(int64_t socket_handle) {
    if (socket_handle == 0) return;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;

    if (sock->sock != INVALID_SOCKET) {
        closesocket(sock->sock);
        sock->sock = INVALID_SOCKET;
    }

    if (sock->host) {
        free(sock->host);
        sock->host = NULL;
    }

    sock->is_connected = 0;
    free(sock);
}

#else
// POSIX (Linux/macOS) uses Berkeley sockets
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <unistd.h>

typedef struct {
    int sock;
    char* host;
    int port;
    int is_connected;
} BmbAsyncSocket;

// Connect to a TCP server
int64_t bmb_async_socket_connect(int64_t host_handle, int64_t port) {
    if (host_handle == 0) return 0;
    const char* host = (const char*)host_handle;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)malloc(sizeof(BmbAsyncSocket));
    if (!sock) return 0;

    sock->sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock->sock < 0) {
        free(sock);
        return 0;
    }

    struct sockaddr_in server;
    server.sin_family = AF_INET;
    server.sin_port = htons((unsigned short)port);

    // Try to parse as IP address first
    if (inet_pton(AF_INET, host, &server.sin_addr) != 1) {
        // If that fails, try DNS resolution
        struct addrinfo hints = {0};
        struct addrinfo* result = NULL;
        hints.ai_family = AF_INET;
        hints.ai_socktype = SOCK_STREAM;

        if (getaddrinfo(host, NULL, &hints, &result) != 0 || !result) {
            close(sock->sock);
            free(sock);
            return 0;
        }

        struct sockaddr_in* addr = (struct sockaddr_in*)result->ai_addr;
        server.sin_addr = addr->sin_addr;
        freeaddrinfo(result);
    }

    if (connect(sock->sock, (struct sockaddr*)&server, sizeof(server)) < 0) {
        close(sock->sock);
        free(sock);
        return 0;
    }

    sock->host = strdup(host);
    sock->port = (int)port;
    sock->is_connected = 1;

    return (int64_t)sock;
}

// Receive data from socket
int64_t bmb_async_socket_read(int64_t socket_handle) {
    if (socket_handle == 0) return 0;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return 0;

    // Read up to 4KB at a time
    char* buffer = (char*)malloc(4096);
    if (!buffer) return 0;

    ssize_t received = recv(sock->sock, buffer, 4095, 0);
    if (received <= 0) {
        free(buffer);
        return 0;
    }

    buffer[received] = '\0';
    return (int64_t)buffer;
}

// Send data to socket
void bmb_async_socket_write(int64_t socket_handle, int64_t content_handle) {
    if (socket_handle == 0 || content_handle == 0) return;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return;

    const char* content = (const char*)content_handle;
    send(sock->sock, content, strlen(content), 0);
}

// Close socket
void bmb_async_socket_close(int64_t socket_handle) {
    if (socket_handle == 0) return;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;

    if (sock->sock >= 0) {
        close(sock->sock);
        sock->sock = -1;
    }

    if (sock->host) {
        free(sock->host);
        sock->host = NULL;
    }

    sock->is_connected = 0;
    free(sock);
}

#endif

// ============================================================================
// v0.99: Non-Blocking Socket I/O (Event Loop-based)
// ============================================================================
// New async socket API that uses the event loop for true non-blocking I/O.
// These functions return immediately and resolve futures when I/O completes.

// Global event loop instance (singleton)
static BmbEventLoop* g_event_loop = NULL;

BmbEventLoop* bmb_get_event_loop(void) {
    if (!g_event_loop) {
        g_event_loop = bmb_event_loop_create();
    }
    return g_event_loop;
}

// Non-blocking connect context
typedef struct {
    int64_t socket_handle;  // BmbAsyncSocket*
    int64_t future_handle;  // BmbFuture* to resolve
    int completed;
} BmbNbConnectCtx;

#ifdef _WIN32

// Set socket to non-blocking mode (Windows)
static int set_nonblocking_win(SOCKET sock) {
    u_long mode = 1;
    return ioctlsocket(sock, FIONBIO, &mode);
}

// Callback when connect completes (socket becomes writable)
static void on_connect_ready(void* user_data, int64_t fd, int events) {
    BmbNbConnectCtx* ctx = (BmbNbConnectCtx*)user_data;
    if (ctx->completed) return;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)ctx->socket_handle;

    if (events & BMB_EVENT_ERROR) {
        // Connect failed
        closesocket(sock->sock);
        sock->sock = INVALID_SOCKET;
        sock->is_connected = 0;
        ctx->completed = 1;
        // Unregister from event loop
        BmbEventLoop* loop = bmb_get_event_loop();
        bmb_event_loop_unregister(loop, fd);
        free(ctx);
        return;
    }

    if (events & BMB_EVENT_WRITE) {
        // Check SO_ERROR to verify connection succeeded
        int error = 0;
        int len = sizeof(error);
        getsockopt(sock->sock, SOL_SOCKET, SO_ERROR, (char*)&error, &len);

        if (error == 0) {
            sock->is_connected = 1;
        } else {
            closesocket(sock->sock);
            sock->sock = INVALID_SOCKET;
            sock->is_connected = 0;
        }

        ctx->completed = 1;
        BmbEventLoop* loop = bmb_get_event_loop();
        bmb_event_loop_unregister(loop, fd);
        free(ctx);
    }
}

// Non-blocking connect: returns socket handle, registers with event loop
int64_t bmb_nb_socket_connect(int64_t host_handle, int64_t port) {
    ensure_winsock_init();
    if (host_handle == 0) return 0;
    const char* host = (const char*)host_handle;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)malloc(sizeof(BmbAsyncSocket));
    if (!sock) return 0;

    sock->sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock->sock == INVALID_SOCKET) { free(sock); return 0; }

    // Set non-blocking
    set_nonblocking_win(sock->sock);

    struct sockaddr_in server;
    server.sin_family = AF_INET;
    server.sin_port = htons((unsigned short)port);

    if (inet_pton(AF_INET, host, &server.sin_addr) != 1) {
        struct addrinfo hints = {0};
        struct addrinfo* result = NULL;
        hints.ai_family = AF_INET;
        hints.ai_socktype = SOCK_STREAM;
        if (getaddrinfo(host, NULL, &hints, &result) != 0 || !result) {
            closesocket(sock->sock);
            free(sock);
            return 0;
        }
        struct sockaddr_in* addr = (struct sockaddr_in*)result->ai_addr;
        server.sin_addr = addr->sin_addr;
        freeaddrinfo(result);
    }

    sock->host = strdup(host);
    sock->port = (int)port;
    sock->is_connected = 0;

    // Initiate non-blocking connect
    int ret = connect(sock->sock, (struct sockaddr*)&server, sizeof(server));
    if (ret == SOCKET_ERROR) {
        int err = WSAGetLastError();
        if (err != WSAEWOULDBLOCK) {
            // Real error
            closesocket(sock->sock);
            free(sock->host);
            free(sock);
            return 0;
        }
        // WSAEWOULDBLOCK = connect in progress, register for write-readiness
        BmbNbConnectCtx* ctx = (BmbNbConnectCtx*)malloc(sizeof(BmbNbConnectCtx));
        if (ctx) {
            ctx->socket_handle = (int64_t)sock;
            ctx->future_handle = 0;
            ctx->completed = 0;
            BmbEventLoop* loop = bmb_get_event_loop();
            bmb_event_loop_register(loop, (int64_t)sock->sock,
                                    BMB_EVENT_WRITE, on_connect_ready, ctx);
        }
    } else {
        // Connected immediately
        sock->is_connected = 1;
    }

    return (int64_t)sock;
}

// Non-blocking recv callback context
typedef struct {
    int64_t socket_handle;
    char* buffer;
    int buffer_size;
    int received;
    int completed;
} BmbNbRecvCtx;

static void on_recv_ready(void* user_data, int64_t fd, int events) {
    BmbNbRecvCtx* ctx = (BmbNbRecvCtx*)user_data;
    if (ctx->completed) return;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)ctx->socket_handle;

    if (events & (BMB_EVENT_READ | BMB_EVENT_ERROR)) {
        ctx->received = recv(sock->sock, ctx->buffer, ctx->buffer_size - 1, 0);
        if (ctx->received > 0) {
            ctx->buffer[ctx->received] = '\0';
        }
        ctx->completed = 1;
        BmbEventLoop* loop = bmb_get_event_loop();
        bmb_event_loop_unregister(loop, fd);
    }
}

// Non-blocking recv: registers socket for read-readiness
int64_t bmb_nb_socket_read(int64_t socket_handle) {
    if (socket_handle == 0) return 0;
    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return 0;

    BmbNbRecvCtx* ctx = (BmbNbRecvCtx*)calloc(1, sizeof(BmbNbRecvCtx));
    if (!ctx) return 0;

    ctx->socket_handle = socket_handle;
    ctx->buffer = (char*)malloc(4096);
    ctx->buffer_size = 4096;
    ctx->received = 0;
    ctx->completed = 0;

    if (!ctx->buffer) { free(ctx); return 0; }

    BmbEventLoop* loop = bmb_get_event_loop();
    bmb_event_loop_register(loop, (int64_t)sock->sock,
                            BMB_EVENT_READ, on_recv_ready, ctx);

    // Poll until data arrives (simple blocking wait for now)
    while (!ctx->completed) {
        bmb_event_loop_run_once(loop, 100);
    }

    int64_t result = 0;
    if (ctx->received > 0) {
        result = (int64_t)ctx->buffer;
    } else {
        free(ctx->buffer);
    }
    free(ctx);
    return result;
}

// Non-blocking send (for now, just send with non-blocking socket)
void bmb_nb_socket_write(int64_t socket_handle, int64_t content_handle) {
    if (socket_handle == 0 || content_handle == 0) return;
    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return;
    const char* content = (const char*)content_handle;
    send(sock->sock, content, (int)strlen(content), 0);
}

#else
// POSIX non-blocking sockets
#include <fcntl.h>

static int set_nonblocking_posix(int fd) {
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags < 0) return -1;
    return fcntl(fd, F_SETFL, flags | O_NONBLOCK);
}

static void on_connect_ready(void* user_data, int64_t fd, int events) {
    BmbNbConnectCtx* ctx = (BmbNbConnectCtx*)user_data;
    if (ctx->completed) return;
    BmbAsyncSocket* sock = (BmbAsyncSocket*)ctx->socket_handle;

    if (events & BMB_EVENT_ERROR) {
        close(sock->sock);
        sock->sock = -1;
        sock->is_connected = 0;
        ctx->completed = 1;
        BmbEventLoop* loop = bmb_get_event_loop();
        bmb_event_loop_unregister(loop, fd);
        free(ctx);
        return;
    }

    if (events & BMB_EVENT_WRITE) {
        int error = 0;
        socklen_t len = sizeof(error);
        getsockopt(sock->sock, SOL_SOCKET, SO_ERROR, &error, &len);
        if (error == 0) {
            sock->is_connected = 1;
        } else {
            close(sock->sock);
            sock->sock = -1;
            sock->is_connected = 0;
        }
        ctx->completed = 1;
        BmbEventLoop* loop = bmb_get_event_loop();
        bmb_event_loop_unregister(loop, fd);
        free(ctx);
    }
}

int64_t bmb_nb_socket_connect(int64_t host_handle, int64_t port) {
    if (host_handle == 0) return 0;
    const char* host = (const char*)host_handle;

    BmbAsyncSocket* sock = (BmbAsyncSocket*)malloc(sizeof(BmbAsyncSocket));
    if (!sock) return 0;

    sock->sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock->sock < 0) { free(sock); return 0; }

    set_nonblocking_posix(sock->sock);

    struct sockaddr_in server;
    server.sin_family = AF_INET;
    server.sin_port = htons((unsigned short)port);

    if (inet_pton(AF_INET, host, &server.sin_addr) != 1) {
        struct addrinfo hints = {0};
        struct addrinfo* result = NULL;
        hints.ai_family = AF_INET;
        hints.ai_socktype = SOCK_STREAM;
        if (getaddrinfo(host, NULL, &hints, &result) != 0 || !result) {
            close(sock->sock);
            free(sock);
            return 0;
        }
        struct sockaddr_in* addr = (struct sockaddr_in*)result->ai_addr;
        server.sin_addr = addr->sin_addr;
        freeaddrinfo(result);
    }

    sock->host = strdup(host);
    sock->port = (int)port;
    sock->is_connected = 0;

    int ret = connect(sock->sock, (struct sockaddr*)&server, sizeof(server));
    if (ret < 0) {
        if (errno != EINPROGRESS) {
            close(sock->sock);
            free(sock->host);
            free(sock);
            return 0;
        }
        BmbNbConnectCtx* ctx = (BmbNbConnectCtx*)malloc(sizeof(BmbNbConnectCtx));
        if (ctx) {
            ctx->socket_handle = (int64_t)sock;
            ctx->future_handle = 0;
            ctx->completed = 0;
            BmbEventLoop* loop = bmb_get_event_loop();
            bmb_event_loop_register(loop, (int64_t)sock->sock,
                                    BMB_EVENT_WRITE, on_connect_ready, ctx);
        }
    } else {
        sock->is_connected = 1;
    }

    return (int64_t)sock;
}

typedef struct {
    int64_t socket_handle;
    char* buffer;
    int buffer_size;
    int received;
    int completed;
} BmbNbRecvCtx;

static void on_recv_ready(void* user_data, int64_t fd, int events) {
    BmbNbRecvCtx* ctx = (BmbNbRecvCtx*)user_data;
    if (ctx->completed) return;
    BmbAsyncSocket* sock = (BmbAsyncSocket*)ctx->socket_handle;

    if (events & (BMB_EVENT_READ | BMB_EVENT_ERROR)) {
        ctx->received = recv(sock->sock, ctx->buffer, ctx->buffer_size - 1, 0);
        if (ctx->received > 0) {
            ctx->buffer[ctx->received] = '\0';
        }
        ctx->completed = 1;
        BmbEventLoop* loop = bmb_get_event_loop();
        bmb_event_loop_unregister(loop, fd);
    }
}

int64_t bmb_nb_socket_read(int64_t socket_handle) {
    if (socket_handle == 0) return 0;
    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return 0;

    BmbNbRecvCtx* ctx = (BmbNbRecvCtx*)calloc(1, sizeof(BmbNbRecvCtx));
    if (!ctx) return 0;

    ctx->socket_handle = socket_handle;
    ctx->buffer = (char*)malloc(4096);
    ctx->buffer_size = 4096;
    ctx->received = 0;
    ctx->completed = 0;

    if (!ctx->buffer) { free(ctx); return 0; }

    BmbEventLoop* loop = bmb_get_event_loop();
    bmb_event_loop_register(loop, (int64_t)sock->sock,
                            BMB_EVENT_READ, on_recv_ready, ctx);

    while (!ctx->completed) {
        bmb_event_loop_run_once(loop, 100);
    }

    int64_t result = 0;
    if (ctx->received > 0) {
        result = (int64_t)ctx->buffer;
    } else {
        free(ctx->buffer);
    }
    free(ctx);
    return result;
}

void bmb_nb_socket_write(int64_t socket_handle, int64_t content_handle) {
    if (socket_handle == 0 || content_handle == 0) return;
    BmbAsyncSocket* sock = (BmbAsyncSocket*)socket_handle;
    if (!sock->is_connected) return;
    const char* content = (const char*)content_handle;
    send(sock->sock, content, strlen(content), 0);
}

#endif

// ============================================================================
// ThreadPool (v0.84)
// ============================================================================

// Task in the work queue (renamed to avoid conflict with BmbTask in executor)
typedef struct BmbPoolTask {
    void (*func)(void);           // Function pointer (fn() -> ())
    struct BmbPoolTask* next;     // Next task in queue
} BmbPoolTask;

#ifdef _WIN32
// Windows ThreadPool implementation

typedef struct BmbThreadPool {
    HANDLE* workers;              // Worker thread handles
    int num_workers;              // Number of workers

    BmbPoolTask* task_head;       // Head of task queue
    BmbPoolTask* task_tail;       // Tail of task queue

    CRITICAL_SECTION lock;        // Queue lock
    CONDITION_VARIABLE not_empty; // Condition: queue not empty

    int shutdown;                 // Shutdown flag
    int active_tasks;             // Number of tasks being executed
} BmbThreadPool;

// Worker thread function (Windows)
static DWORD WINAPI bmb_thread_pool_worker_win(LPVOID arg) {
    BmbThreadPool* pool = (BmbThreadPool*)arg;

    while (1) {
        EnterCriticalSection(&pool->lock);

        // Wait for task or shutdown
        while (pool->task_head == NULL && !pool->shutdown) {
            SleepConditionVariableCS(&pool->not_empty, &pool->lock, INFINITE);
        }

        // Check if we should exit
        if (pool->shutdown && pool->task_head == NULL) {
            LeaveCriticalSection(&pool->lock);
            break;
        }

        // Dequeue task
        BmbPoolTask* task = pool->task_head;
        if (task != NULL) {
            pool->task_head = task->next;
            if (pool->task_head == NULL) {
                pool->task_tail = NULL;
            }
            pool->active_tasks++;
        }

        LeaveCriticalSection(&pool->lock);

        // Execute task outside lock
        if (task != NULL) {
            task->func();
            free(task);

            EnterCriticalSection(&pool->lock);
            pool->active_tasks--;
            LeaveCriticalSection(&pool->lock);
        }
    }

    return 0;
}

// Create a new thread pool with specified number of workers
int64_t bmb_thread_pool_new(int64_t num_workers) {
    if (num_workers <= 0) num_workers = 4;  // Default to 4 workers

    BmbThreadPool* pool = (BmbThreadPool*)malloc(sizeof(BmbThreadPool));
    if (!pool) return 0;

    pool->workers = (HANDLE*)malloc(sizeof(HANDLE) * num_workers);
    if (!pool->workers) {
        free(pool);
        return 0;
    }

    pool->num_workers = (int)num_workers;
    pool->task_head = NULL;
    pool->task_tail = NULL;
    pool->shutdown = 0;
    pool->active_tasks = 0;

    InitializeCriticalSection(&pool->lock);
    InitializeConditionVariable(&pool->not_empty);

    // Start worker threads
    for (int i = 0; i < num_workers; i++) {
        pool->workers[i] = CreateThread(NULL, 0, bmb_thread_pool_worker_win, pool, 0, NULL);
    }

    return (int64_t)pool;
}

// Execute a task on the thread pool
void bmb_thread_pool_execute(int64_t pool_handle, int64_t task_handle) {
    if (pool_handle == 0 || task_handle == 0) return;

    BmbThreadPool* pool = (BmbThreadPool*)pool_handle;
    void (*func)(void) = (void (*)(void))task_handle;

    // Create task
    BmbPoolTask* task = (BmbPoolTask*)malloc(sizeof(BmbPoolTask));
    if (!task) return;

    task->func = func;
    task->next = NULL;

    // Enqueue task
    EnterCriticalSection(&pool->lock);

    if (pool->task_tail == NULL) {
        pool->task_head = task;
        pool->task_tail = task;
    } else {
        pool->task_tail->next = task;
        pool->task_tail = task;
    }

    WakeConditionVariable(&pool->not_empty);
    LeaveCriticalSection(&pool->lock);
}

// Wait for all tasks to complete and shutdown the pool
void bmb_thread_pool_join(int64_t pool_handle) {
    if (pool_handle == 0) return;

    BmbThreadPool* pool = (BmbThreadPool*)pool_handle;

    // Signal shutdown
    EnterCriticalSection(&pool->lock);
    pool->shutdown = 1;
    WakeAllConditionVariable(&pool->not_empty);
    LeaveCriticalSection(&pool->lock);

    // Wait for all workers to finish
    WaitForMultipleObjects(pool->num_workers, pool->workers, TRUE, INFINITE);

    // Close thread handles
    for (int i = 0; i < pool->num_workers; i++) {
        CloseHandle(pool->workers[i]);
    }

    // Clean up remaining tasks (if any)
    BmbPoolTask* task = pool->task_head;
    while (task != NULL) {
        BmbPoolTask* next = task->next;
        free(task);
        task = next;
    }

    DeleteCriticalSection(&pool->lock);
    free(pool->workers);
    free(pool);
}

// Request shutdown (may not wait for tasks)
void bmb_thread_pool_shutdown(int64_t pool_handle) {
    if (pool_handle == 0) return;

    BmbThreadPool* pool = (BmbThreadPool*)pool_handle;

    // Signal shutdown
    EnterCriticalSection(&pool->lock);
    pool->shutdown = 1;
    WakeAllConditionVariable(&pool->not_empty);
    LeaveCriticalSection(&pool->lock);
}

// ============================================================================
// v0.85: Scoped Threads (Windows)
// ============================================================================

typedef struct BmbScopedThread {
    HANDLE thread;
    struct BmbScopedThread* next;
} BmbScopedThread;

typedef struct BmbScope {
    BmbScopedThread* threads;     // Linked list of spawned threads
    CRITICAL_SECTION lock;        // Protects thread list
} BmbScope;

// Create a new scope for structured concurrency
int64_t bmb_scope_new(void) {
    BmbScope* scope = (BmbScope*)malloc(sizeof(BmbScope));
    if (!scope) return 0;

    scope->threads = NULL;
    InitializeCriticalSection(&scope->lock);

    return (int64_t)scope;
}

// Thread wrapper for scoped spawn
typedef struct {
    void (*func)(void);
} BmbScopeTask;

static DWORD WINAPI bmb_scope_thread_func(LPVOID arg) {
    BmbScopeTask* task = (BmbScopeTask*)arg;
    task->func();
    free(task);
    return 0;
}

// Spawn a scoped thread
void bmb_scope_spawn(int64_t scope_handle, int64_t task_handle) {
    if (scope_handle == 0 || task_handle == 0) return;

    BmbScope* scope = (BmbScope*)scope_handle;
    void (*func)(void) = (void (*)(void))task_handle;

    // Create task wrapper
    BmbScopeTask* task = (BmbScopeTask*)malloc(sizeof(BmbScopeTask));
    if (!task) return;
    task->func = func;

    // Create thread
    HANDLE thread = CreateThread(NULL, 0, bmb_scope_thread_func, task, 0, NULL);
    if (thread == NULL) {
        free(task);
        return;
    }

    // Add to thread list
    BmbScopedThread* node = (BmbScopedThread*)malloc(sizeof(BmbScopedThread));
    if (!node) {
        TerminateThread(thread, 0);
        CloseHandle(thread);
        return;
    }
    node->thread = thread;

    EnterCriticalSection(&scope->lock);
    node->next = scope->threads;
    scope->threads = node;
    LeaveCriticalSection(&scope->lock);
}

// Wait for all scoped threads to complete
void bmb_scope_wait(int64_t scope_handle) {
    if (scope_handle == 0) return;

    BmbScope* scope = (BmbScope*)scope_handle;

    EnterCriticalSection(&scope->lock);
    BmbScopedThread* thread = scope->threads;
    scope->threads = NULL;
    LeaveCriticalSection(&scope->lock);

    // Wait for all threads
    while (thread != NULL) {
        WaitForSingleObject(thread->thread, INFINITE);
        CloseHandle(thread->thread);
        BmbScopedThread* next = thread->next;
        free(thread);
        thread = next;
    }

    // Clean up scope
    DeleteCriticalSection(&scope->lock);
    free(scope);
}

#else
// POSIX (Linux/macOS) ThreadPool implementation

typedef struct BmbThreadPool {
    pthread_t* workers;           // Worker threads
    int num_workers;              // Number of workers

    BmbPoolTask* task_head;       // Head of task queue
    BmbPoolTask* task_tail;       // Tail of task queue

    pthread_mutex_t lock;         // Queue lock
    pthread_cond_t not_empty;     // Condition: queue not empty

    int shutdown;                 // Shutdown flag
    int active_tasks;             // Number of tasks being executed
} BmbThreadPool;

// Worker thread function (POSIX)
static void* bmb_thread_pool_worker(void* arg) {
    BmbThreadPool* pool = (BmbThreadPool*)arg;

    while (1) {
        pthread_mutex_lock(&pool->lock);

        // Wait for task or shutdown
        while (pool->task_head == NULL && !pool->shutdown) {
            pthread_cond_wait(&pool->not_empty, &pool->lock);
        }

        // Check if we should exit
        if (pool->shutdown && pool->task_head == NULL) {
            pthread_mutex_unlock(&pool->lock);
            break;
        }

        // Dequeue task
        BmbPoolTask* task = pool->task_head;
        if (task != NULL) {
            pool->task_head = task->next;
            if (pool->task_head == NULL) {
                pool->task_tail = NULL;
            }
            pool->active_tasks++;
        }

        pthread_mutex_unlock(&pool->lock);

        // Execute task outside lock
        if (task != NULL) {
            task->func();
            free(task);

            pthread_mutex_lock(&pool->lock);
            pool->active_tasks--;
            pthread_mutex_unlock(&pool->lock);
        }
    }

    return NULL;
}

// Create a new thread pool with specified number of workers
int64_t bmb_thread_pool_new(int64_t num_workers) {
    if (num_workers <= 0) num_workers = 4;  // Default to 4 workers

    BmbThreadPool* pool = (BmbThreadPool*)malloc(sizeof(BmbThreadPool));
    if (!pool) return 0;

    pool->workers = (pthread_t*)malloc(sizeof(pthread_t) * num_workers);
    if (!pool->workers) {
        free(pool);
        return 0;
    }

    pool->num_workers = (int)num_workers;
    pool->task_head = NULL;
    pool->task_tail = NULL;
    pool->shutdown = 0;
    pool->active_tasks = 0;

    pthread_mutex_init(&pool->lock, NULL);
    pthread_cond_init(&pool->not_empty, NULL);

    // Start worker threads
    for (int i = 0; i < num_workers; i++) {
        pthread_create(&pool->workers[i], NULL, bmb_thread_pool_worker, pool);
    }

    return (int64_t)pool;
}

// Execute a task on the thread pool
void bmb_thread_pool_execute(int64_t pool_handle, int64_t task_handle) {
    if (pool_handle == 0 || task_handle == 0) return;

    BmbThreadPool* pool = (BmbThreadPool*)pool_handle;
    void (*func)(void) = (void (*)(void))task_handle;

    // Create task
    BmbPoolTask* task = (BmbPoolTask*)malloc(sizeof(BmbPoolTask));
    if (!task) return;

    task->func = func;
    task->next = NULL;

    // Enqueue task
    pthread_mutex_lock(&pool->lock);

    if (pool->task_tail == NULL) {
        pool->task_head = task;
        pool->task_tail = task;
    } else {
        pool->task_tail->next = task;
        pool->task_tail = task;
    }

    pthread_cond_signal(&pool->not_empty);
    pthread_mutex_unlock(&pool->lock);
}

// Wait for all tasks to complete and shutdown the pool
void bmb_thread_pool_join(int64_t pool_handle) {
    if (pool_handle == 0) return;

    BmbThreadPool* pool = (BmbThreadPool*)pool_handle;

    // Signal shutdown
    pthread_mutex_lock(&pool->lock);
    pool->shutdown = 1;
    pthread_cond_broadcast(&pool->not_empty);
    pthread_mutex_unlock(&pool->lock);

    // Wait for all workers to finish
    for (int i = 0; i < pool->num_workers; i++) {
        pthread_join(pool->workers[i], NULL);
    }

    // Clean up remaining tasks (if any)
    BmbPoolTask* task = pool->task_head;
    while (task != NULL) {
        BmbPoolTask* next = task->next;
        free(task);
        task = next;
    }

    pthread_mutex_destroy(&pool->lock);
    pthread_cond_destroy(&pool->not_empty);
    free(pool->workers);
    free(pool);
}

// Request shutdown (may not wait for tasks)
void bmb_thread_pool_shutdown(int64_t pool_handle) {
    if (pool_handle == 0) return;

    BmbThreadPool* pool = (BmbThreadPool*)pool_handle;

    // Signal shutdown
    pthread_mutex_lock(&pool->lock);
    pool->shutdown = 1;
    pthread_cond_broadcast(&pool->not_empty);
    pthread_mutex_unlock(&pool->lock);
}

// ============================================================================
// v0.85: Scoped Threads (POSIX)
// ============================================================================

typedef struct BmbScopedThread {
    pthread_t thread;
    struct BmbScopedThread* next;
} BmbScopedThread;

typedef struct BmbScope {
    BmbScopedThread* threads;     // Linked list of spawned threads
    pthread_mutex_t lock;         // Protects thread list
} BmbScope;

// Create a new scope for structured concurrency
int64_t bmb_scope_new(void) {
    BmbScope* scope = (BmbScope*)malloc(sizeof(BmbScope));
    if (!scope) return 0;

    scope->threads = NULL;
    pthread_mutex_init(&scope->lock, NULL);

    return (int64_t)scope;
}

// Thread wrapper for scoped spawn
typedef struct {
    void (*func)(void);
} BmbScopeTask;

static void* bmb_scope_thread_func(void* arg) {
    BmbScopeTask* task = (BmbScopeTask*)arg;
    task->func();
    free(task);
    return NULL;
}

// Spawn a scoped thread
void bmb_scope_spawn(int64_t scope_handle, int64_t task_handle) {
    if (scope_handle == 0 || task_handle == 0) return;

    BmbScope* scope = (BmbScope*)scope_handle;
    void (*func)(void) = (void (*)(void))task_handle;

    // Create task wrapper
    BmbScopeTask* task = (BmbScopeTask*)malloc(sizeof(BmbScopeTask));
    if (!task) return;
    task->func = func;

    // Create thread node
    BmbScopedThread* node = (BmbScopedThread*)malloc(sizeof(BmbScopedThread));
    if (!node) {
        free(task);
        return;
    }

    // Create thread
    if (pthread_create(&node->thread, NULL, bmb_scope_thread_func, task) != 0) {
        free(task);
        free(node);
        return;
    }

    // Add to thread list
    pthread_mutex_lock(&scope->lock);
    node->next = scope->threads;
    scope->threads = node;
    pthread_mutex_unlock(&scope->lock);
}

// Wait for all scoped threads to complete
void bmb_scope_wait(int64_t scope_handle) {
    if (scope_handle == 0) return;

    BmbScope* scope = (BmbScope*)scope_handle;

    pthread_mutex_lock(&scope->lock);
    BmbScopedThread* thread = scope->threads;
    scope->threads = NULL;
    pthread_mutex_unlock(&scope->lock);

    // Wait for all threads
    while (thread != NULL) {
        pthread_join(thread->thread, NULL);
        BmbScopedThread* next = thread->next;
        free(thread);
        thread = next;
    }

    // Clean up scope
    pthread_mutex_destroy(&scope->lock);
    free(scope);
}

#endif

// ============================================================================
// Entry point
// ============================================================================

int64_t bmb_user_main(void);
int main(int argc, char** argv) {
    g_argc = argc;
    g_argv = argv;
    // v0.88.4: Arena ENABLED by default.
    // BMB has no GC or destructors - without arena, every string allocation
    // leaks (malloc without free). Arena pools all allocations and frees
    // everything at process exit via bmb_arena_destroy().
    // Hard limit (default 4GB) prevents OOM/BSOD - process exits with error.
    // Override limit via BMB_ARENA_MAX_SIZE env var (e.g. "8G", "512M").
    bmb_arena_mode(1);
    int result = (int)bmb_user_main();
    bmb_arena_destroy();
    return result;
}

// v0.51.46: Windows compatibility fixes
#ifdef _WIN32
#define NOMINMAX  // Prevent min/max macros
#define _CRT_SECURE_NO_WARNINGS  // Suppress fopen/scanf warnings
#endif

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <inttypes.h>

// v0.70: Threading support
#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#include <errno.h>   // v0.77: For ETIMEDOUT
#include <time.h>    // v0.77: For clock_gettime
#endif

// BMB Runtime Library

// v0.51.51: BmbString struct for type-safe string handling
// Matches LLVM IR: %BmbString = type { ptr, i64, i64 }
typedef struct {
    char* data;      // pointer to null-terminated string data
    int64_t len;     // string length (excluding null terminator)
    int64_t cap;     // capacity (excluding null terminator)
} BmbString;

// Helper to create a new BmbString from raw char*
static BmbString* bmb_string_wrap(char* data) {
    if (!data) {
        data = (char*)malloc(1);
        data[0] = '\0';
    }
    BmbString* s = (BmbString*)malloc(sizeof(BmbString));
    int64_t len = 0;
    while (data[len]) len++;
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

// v0.97: Character functions
// v0.60.107: bmb_chr returns BmbString* to match string type system
BmbString* bmb_chr(int64_t n) {
    char* data = (char*)malloc(2);
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

// v0.99: String conversion functions
char* bmb_char_to_string(int32_t c) {
    char* s = (char*)malloc(5);  // UTF-8 max 4 bytes + null
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

char* bmb_int_to_string(int64_t n) {
    char* s = (char*)malloc(21);  // Max i64 is 20 digits + sign
    snprintf(s, 21, "%ld", (long)n);
    return s;
}

// v0.60.244: Fast integer-to-BmbString conversion for bootstrap compiler
// Returns BmbString* which matches the bootstrap's String type
BmbString* bmb_fast_i2s(int64_t n) {
    char* s = (char*)malloc(21);
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
BmbString* bmb_string_concat(const BmbString* a, const BmbString* b) {
    if (!a || !b || !a->data || !b->data) {
        return bmb_string_wrap(NULL);
    }
    int64_t len_a = a->len;
    int64_t len_b = b->len;
    char* result = (char*)malloc(len_a + len_b + 1);
    for (int64_t i = 0; i < len_a; i++) result[i] = a->data[i];
    for (int64_t i = 0; i < len_b; i++) result[len_a + i] = b->data[i];
    result[len_a + len_b] = '\0';

    BmbString* s = (BmbString*)malloc(sizeof(BmbString));
    s->data = result;
    s->len = len_a + len_b;
    s->cap = len_a + len_b;
    return s;
}

// v0.51.51: String functions updated for BmbString structs

// Convert C string to BmbString
BmbString* bmb_string_from_cstr(const char* s) {
    if (!s) return bmb_string_wrap(NULL);
    int64_t len = 0;
    while (s[len]) len++;
    char* copy = (char*)malloc(len + 1);
    for (int64_t i = 0; i <= len; i++) copy[i] = s[i];
    return bmb_string_wrap(copy);
}

// Create new string with given length (allocates copy)
BmbString* bmb_string_new(const char* s, int64_t len) {
    char* result = (char*)malloc(len + 1);
    for (int64_t i = 0; i < len; i++) result[i] = s[i];
    result[len] = '\0';
    return bmb_string_wrap(result);
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
    for (int64_t i = 0; i < a->len; i++) {
        if (a->data[i] != b->data[i]) return 0;
    }
    return 1;
}

// String slice (substring from start to end, exclusive) - returns BmbString*
BmbString* bmb_string_slice(const BmbString* s, int64_t start, int64_t end) {
    if (!s || !s->data || start < 0 || end < start || start > s->len) {
        return bmb_string_wrap(NULL);
    }
    if (end > s->len) end = s->len;
    int64_t len = end - start;
    char* result = (char*)malloc(len + 1);
    for (int64_t i = 0; i < len; i++) {
        result[i] = s->data[start + i];
    }
    result[len] = '\0';
    return bmb_string_wrap(result);
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
BmbString* chr(int64_t n) {
    char* data = (char*)malloc(2);
    data[0] = (char)n;
    data[1] = '\0';
    return bmb_string_wrap(data);
}

// v0.51.51: char_to_string returns BmbString*
BmbString* char_to_string(int32_t c) {
    char* data = (char*)malloc(2);
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
    sb->cap = 64;
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
int64_t bmb_sb_push(int64_t handle, const BmbString* s) {
    if (!s || !s->data || !handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;
    int64_t slen = s->len;

    // Grow if needed
    while (sb->len + slen + 1 > sb->cap) {
        sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }

    // Append
    for (int64_t i = 0; i < slen; i++) {
        sb->data[sb->len + i] = s->data[i];
    }
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
BmbString* bmb_sb_build(int64_t handle) {
    if (!handle) {
        return bmb_string_wrap(NULL);
    }
    StringBuilder* sb = (StringBuilder*)handle;
    // Return copy of the built string
    char* result = (char*)malloc(sb->len + 1);
    for (int64_t i = 0; i <= sb->len; i++) {
        result[i] = sb->data[i];
    }
    return bmb_string_wrap(result);
}

int64_t bmb_sb_clear(int64_t handle) {
    StringBuilder* sb = (StringBuilder*)handle;
    sb->len = 0;
    sb->data[0] = '\0';
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
int64_t bmb_system(const char* cmd) {
    return (int64_t)system(cmd);
}

char* bmb_getenv(const char* name) {
    const char* val = getenv(name);
    if (!val) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return empty;
    }
    // Return copy
    size_t len = 0;
    while (val[len]) len++;
    char* result = (char*)malloc(len + 1);
    for (size_t i = 0; i <= len; i++) result[i] = val[i];
    return result;
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

int64_t bmb_file_exists(const char* path) {
    struct stat st;
    return (stat(path, &st) == 0) ? 1 : 0;
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

    // In synchronous mode, all tasks are already complete
    // A real async executor would poll pending tasks here

#ifdef _WIN32
    EnterCriticalSection(&exec->lock);
#else
    pthread_mutex_lock(&exec->lock);
#endif

    // Mark all pending tasks as completed (sync mode: they already are)
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
int64_t bmb_block_on(int64_t future_value) {
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
// Entry point
// ============================================================================

int64_t bmb_user_main(void);
int main(int argc, char** argv) {
    g_argc = argc;
    g_argv = argv;
    return (int)bmb_user_main();
}

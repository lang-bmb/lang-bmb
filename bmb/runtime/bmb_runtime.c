// v0.51.46: Windows compatibility fixes
#ifdef _WIN32
#define NOMINMAX  // Prevent min/max macros
#define _CRT_SECURE_NO_WARNINGS  // Suppress fopen/scanf warnings
#endif

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <inttypes.h>

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
// v0.46: bmb_chr returns char* (string) to match LLVM codegen expectations
char* bmb_chr(int64_t n) {
    char* s = (char*)malloc(2);
    s[0] = (char)n;
    s[1] = '\0';
    return s;
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

// v0.46: Command-line argument support for CLI Independence
static int g_argc = 0;
static char** g_argv = NULL;

int64_t bmb_arg_count(void) {
    return (int64_t)g_argc;
}

char* bmb_get_arg(int64_t index) {
    if (index < 0 || index >= g_argc) {
        // Return empty string for out-of-bounds
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return empty;
    }
    // Return a copy of the argument
    const char* arg = g_argv[index];
    size_t len = 0;
    while (arg[len]) len++;
    char* result = (char*)malloc(len + 1);
    for (size_t i = 0; i <= len; i++) result[i] = arg[i];
    return result;
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

// v0.51.51: File I/O wrappers updated for BmbString*
BmbString* read_file(const BmbString* path) {
    return bmb_read_file(path);
}

int64_t write_file(const BmbString* path, const BmbString* content) {
    return bmb_write_file(path, content);
}

// v0.50.20: Argument wrappers
int64_t arg_count(void) {
    return bmb_arg_count();
}

char* get_arg(int64_t index) {
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

// Entry point
int64_t bmb_user_main(void);
int main(int argc, char** argv) {
    g_argc = argc;
    g_argv = argv;
    return (int)bmb_user_main();
}

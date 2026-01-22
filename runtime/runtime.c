// BMB Runtime Library
// Provides basic I/O functions for BMB programs

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// Windows binary mode support
#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif

// Initialize stdout to binary mode on Windows (prevents LF -> CRLF conversion)
static void init_binary_stdout(void) {
#ifdef _WIN32
    _setmode(_fileno(stdout), _O_BINARY);
#endif
}

// Print i64 without newline
void bmb_print_i64(int64_t x) {
    printf("%lld", (long long)x);
}

// Print i64 with newline
void bmb_println_i64(int64_t x) {
    printf("%lld\n", (long long)x);
}

// Print f64 without newline
void bmb_print_f64(double x) {
    printf("%g", x);
}

// Print f64 with newline
void bmb_println_f64(double x) {
    printf("%g\n", x);
}

// Print boolean
void bmb_println_bool(int b) {
    printf("%s\n", b ? "true" : "false");
}

// Assert condition
void bmb_assert(int cond, const char* msg) {
    if (!cond) {
        fprintf(stderr, "Assertion failed: %s\n", msg);
        exit(1);
    }
}

// Panic with message
void bmb_panic(const char* msg) {
    fprintf(stderr, "panic: %s\n", msg);
    exit(1);
}

// ===================================================
// Bootstrap Runtime Functions
// These match the declarations in bootstrap/llvm_ir.bmb
// Using bmb_ prefix to avoid conflicts with stdlib/Windows
// ===================================================

// Undefine Windows macros that conflict with our function names
#ifdef min
#undef min
#endif
#ifdef max
#undef max
#endif

// println(i64) - Print i64 with newline (bootstrap version)
void println(int64_t x) {
    printf("%lld\n", (long long)x);
}

// print(i64) - Print i64 without newline
void print(int64_t x) {
    printf("%lld", (long long)x);
}

// read_int() - Read i64 from stdin
int64_t read_int(void) {
    int64_t x;
    if (scanf("%lld", (long long*)&x) != 1) {
        fprintf(stderr, "Error: failed to read integer\n");
        exit(1);
    }
    return x;
}

// assert(i1) - Assert condition is true
void assert(int cond) {
    if (!cond) {
        fprintf(stderr, "Assertion failed\n");
        exit(1);
    }
}

// abs(i64) - Absolute value
int64_t bmb_abs(int64_t x) {
    return x < 0 ? -x : x;
}

// min(i64, i64) - Minimum of two values
int64_t min(int64_t a, int64_t b) {
    return a < b ? a : b;
}

// max(i64, i64) - Maximum of two values
int64_t max(int64_t a, int64_t b) {
    return a > b ? a : b;
}

// ===================================================
// String Runtime Functions (Phase 32.3)
// For native Bootstrap compiler support
// ===================================================

#include <string.h>
#include <sys/stat.h>

// String type in BMB native runtime: heap-allocated char* with length
// Uses refcount=1 always (simplified GC model)

typedef struct {
    char* data;
    int64_t len;
    int64_t cap;
} BmbString;

// Global string pool for interning (simplified)
#define MAX_STRINGS 65536
static BmbString* string_pool[MAX_STRINGS];
static int64_t string_pool_count = 0;

// Allocate new string
BmbString* bmb_string_new(const char* data, int64_t len) {
    BmbString* s = (BmbString*)malloc(sizeof(BmbString));
    s->data = (char*)malloc(len + 1);
    memcpy(s->data, data, len);
    s->data[len] = '\0';
    s->len = len;
    s->cap = len + 1;
    if (string_pool_count < MAX_STRINGS) {
        string_pool[string_pool_count++] = s;
    }
    return s;
}

// String from C string literal
BmbString* bmb_string_from_cstr(const char* cstr) {
    return bmb_string_new(cstr, strlen(cstr));
}

// Get string length
int64_t bmb_string_len(BmbString* s) {
    return s ? s->len : 0;
}

// Get character at index (as ASCII code)
int64_t bmb_string_char_at(BmbString* s, int64_t idx) {
    if (!s || idx < 0 || idx >= s->len) return 0;
    return (int64_t)(unsigned char)s->data[idx];
}

// Slice string [start, end)
BmbString* bmb_string_slice(BmbString* s, int64_t start, int64_t end) {
    if (!s) return bmb_string_new("", 0);
    if (start < 0) start = 0;
    if (end > s->len) end = s->len;
    if (start >= end) return bmb_string_new("", 0);
    return bmb_string_new(s->data + start, end - start);
}

// Concatenate two strings
BmbString* bmb_string_concat(BmbString* a, BmbString* b) {
    if (!a && !b) return bmb_string_new("", 0);
    if (!a) return bmb_string_new(b->data, b->len);
    if (!b) return bmb_string_new(a->data, a->len);

    int64_t newlen = a->len + b->len;
    char* data = (char*)malloc(newlen + 1);
    memcpy(data, a->data, a->len);
    memcpy(data + a->len, b->data, b->len);
    data[newlen] = '\0';
    BmbString* result = (BmbString*)malloc(sizeof(BmbString));
    result->data = data;
    result->len = newlen;
    result->cap = newlen + 1;
    if (string_pool_count < MAX_STRINGS) {
        string_pool[string_pool_count++] = result;
    }
    return result;
}

// String equality (for BmbString pointers)
int64_t bmb_string_eq(BmbString* a, BmbString* b) {
    if (!a && !b) return 1;
    if (!a || !b) return 0;
    if (a->len != b->len) return 0;
    return memcmp(a->data, b->data, a->len) == 0 ? 1 : 0;
}

// v0.46: Raw C string equality (for string literals)
// This is used when comparing string literals directly
int64_t bmb_cstr_eq(const char* a, const char* b) {
    if (!a && !b) return 1;
    if (!a || !b) return 0;
    return strcmp(a, b) == 0 ? 1 : 0;
}

// chr(i64) -> String: ASCII code to single character string
BmbString* bmb_chr(int64_t code) {
    char buf[2] = { (char)code, '\0' };
    return bmb_string_new(buf, 1);
}

// ord(String) -> i64: First character's ASCII code
int64_t bmb_ord(BmbString* s) {
    if (!s || s->len == 0) return 0;
    return (int64_t)(unsigned char)s->data[0];
}

// Flag to track if binary mode has been initialized
static int binary_mode_initialized = 0;

// Print string without newline
void bmb_print_str(BmbString* s) {
    // Initialize binary mode on first call (prevents LF -> CRLF on Windows)
    if (!binary_mode_initialized) {
        init_binary_stdout();
        binary_mode_initialized = 1;
    }
    if (s && s->data) {
        fwrite(s->data, 1, s->len, stdout);
    }
}

// Print string with newline
void bmb_println_str(BmbString* s) {
    if (s && s->data) {
        fwrite(s->data, 1, s->len, stdout);
    }
    putchar('\n');
}

// ===================================================
// File I/O Runtime Functions (Phase 32.3)
// ===================================================

// v0.51.2: Use _stat64 on Windows for better performance
// clang's default stat() maps to _stat64i32 which is 3x slower than _stat64
#ifdef _WIN32
#define BMB_STAT _stat64
#define BMB_STAT_STRUCT struct __stat64
#else
#define BMB_STAT stat
#define BMB_STAT_STRUCT struct stat
#endif

// Check if file exists (returns 1 if exists, 0 otherwise)
int64_t bmb_file_exists(BmbString* path) {
    if (!path) return 0;
    BMB_STAT_STRUCT st;
    return BMB_STAT(path->data, &st) == 0 ? 1 : 0;
}

// v0.51.2: Direct cstr version for string literal optimization
// Avoids BmbString wrapper overhead when called with constant strings
int64_t bmb_file_exists_cstr(const char* path) {
    if (!path) return 0;
    BMB_STAT_STRUCT st;
    return BMB_STAT(path, &st) == 0 ? 1 : 0;
}

// Get file size (-1 on error)
int64_t bmb_file_size(BmbString* path) {
    if (!path) return -1;
    BMB_STAT_STRUCT st;
    if (BMB_STAT(path->data, &st) != 0) return -1;
    return (int64_t)st.st_size;
}

// Read entire file to string
BmbString* bmb_read_file(BmbString* path) {
    if (!path) return bmb_string_new("", 0);
    FILE* f = fopen(path->data, "rb");
    if (!f) return bmb_string_new("", 0);

    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    char* data = (char*)malloc(size + 1);
    size_t read = fread(data, 1, size, f);
    data[read] = '\0';
    fclose(f);

    BmbString* result = (BmbString*)malloc(sizeof(BmbString));
    result->data = data;
    result->len = read;
    result->cap = size + 1;
    if (string_pool_count < MAX_STRINGS) {
        string_pool[string_pool_count++] = result;
    }
    return result;
}

// Write string to file (returns 0 on success, -1 on error)
int64_t bmb_write_file(BmbString* path, BmbString* content) {
    if (!path || !content) return -1;
    FILE* f = fopen(path->data, "wb");
    if (!f) return -1;
    fwrite(content->data, 1, content->len, f);
    fclose(f);
    return 0;
}

// Append string to file (returns 0 on success, -1 on error)
int64_t bmb_append_file(BmbString* path, BmbString* content) {
    if (!path || !content) return -1;
    FILE* f = fopen(path->data, "ab");
    if (!f) return -1;
    fwrite(content->data, 1, content->len, f);
    fclose(f);
    return 0;
}

// ===================================================
// StringBuilder Runtime Functions (Phase 32.3)
// ===================================================

typedef struct {
    char** fragments;
    int64_t* lengths;
    int64_t count;
    int64_t capacity;
} StringBuilder;

#define MAX_STRING_BUILDERS 8192
static StringBuilder* builders[MAX_STRING_BUILDERS];
static int64_t builder_count = 0;

// Create new StringBuilder, return handle (index)
int64_t bmb_sb_new(void) {
    if (builder_count >= MAX_STRING_BUILDERS) return -1;
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    sb->fragments = (char**)malloc(64 * sizeof(char*));
    sb->lengths = (int64_t*)malloc(64 * sizeof(int64_t));
    sb->count = 0;
    sb->capacity = 64;
    builders[builder_count] = sb;
    return builder_count++;
}

// Push string to StringBuilder
int64_t bmb_sb_push(int64_t handle, BmbString* s) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb || !s) return -1;

    if (sb->count >= sb->capacity) {
        sb->capacity *= 2;
        sb->fragments = (char**)realloc(sb->fragments, sb->capacity * sizeof(char*));
        sb->lengths = (int64_t*)realloc(sb->lengths, sb->capacity * sizeof(int64_t));
    }

    char* copy = (char*)malloc(s->len + 1);
    memcpy(copy, s->data, s->len);
    copy[s->len] = '\0';
    sb->fragments[sb->count] = copy;
    sb->lengths[sb->count] = s->len;
    sb->count++;
    return 0;
}

// Get total length
int64_t bmb_sb_len(int64_t handle) {
    if (handle < 0 || handle >= builder_count) return 0;
    StringBuilder* sb = builders[handle];
    if (!sb) return 0;
    int64_t total = 0;
    for (int64_t i = 0; i < sb->count; i++) {
        total += sb->lengths[i];
    }
    return total;
}

// Build final string
BmbString* bmb_sb_build(int64_t handle) {
    if (handle < 0 || handle >= builder_count) return bmb_string_new("", 0);
    StringBuilder* sb = builders[handle];
    if (!sb) return bmb_string_new("", 0);

    int64_t total = bmb_sb_len(handle);
    char* data = (char*)malloc(total + 1);
    int64_t pos = 0;
    for (int64_t i = 0; i < sb->count; i++) {
        memcpy(data + pos, sb->fragments[i], sb->lengths[i]);
        pos += sb->lengths[i];
    }
    data[total] = '\0';

    BmbString* result = (BmbString*)malloc(sizeof(BmbString));
    result->data = data;
    result->len = total;
    result->cap = total + 1;
    if (string_pool_count < MAX_STRINGS) {
        string_pool[string_pool_count++] = result;
    }
    return result;
}

// Clear StringBuilder
int64_t bmb_sb_clear(int64_t handle) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb) return -1;
    for (int64_t i = 0; i < sb->count; i++) {
        free(sb->fragments[i]);
    }
    sb->count = 0;
    return 0;
}

// ===================================================
// Process Execution Runtime Functions (Phase 32.3)
// ===================================================

// Execute shell command (returns exit code)
int64_t bmb_system(BmbString* cmd) {
    if (!cmd) return -1;
    return system(cmd->data);
}

// Get environment variable
BmbString* bmb_getenv(BmbString* name) {
    if (!name) return bmb_string_new("", 0);
    char* val = getenv(name->data);
    if (!val) return bmb_string_new("", 0);
    return bmb_string_from_cstr(val);
}

// ===================================================
// Method Name Wrappers (Phase 32.3)
// BMB method calls like s.len() generate calls to @len
// These wrappers provide the simple names
// ===================================================

// String method wrappers
int64_t len(BmbString* s) {
    return bmb_string_len(s);
}

int64_t char_at(BmbString* s, int64_t idx) {
    return bmb_string_char_at(s, idx);
}

// v0.46: byte_at is the preferred name (clarity: returns byte, not Unicode char)
int64_t byte_at(BmbString* s, int64_t idx) {
    return bmb_string_char_at(s, idx);
}

BmbString* slice(BmbString* s, int64_t start, int64_t end) {
    return bmb_string_slice(s, start, end);
}

// chr() for character code to string
BmbString* chr(int64_t code) {
    return bmb_chr(code);
}

// ord() for string to character code
int64_t ord(BmbString* s) {
    return bmb_ord(s);
}

// File I/O wrappers
// v0.51.2: Inlined for performance - avoid indirect call overhead
int64_t file_exists(BmbString* path) {
    if (!path) return 0;
    BMB_STAT_STRUCT st;
    return BMB_STAT(path->data, &st) == 0 ? 1 : 0;
}

// v0.51.2: cstr version for string literal optimization
int64_t file_exists_cstr(const char* path) {
    return bmb_file_exists_cstr(path);
}

int64_t file_size(BmbString* path) {
    return bmb_file_size(path);
}

BmbString* read_file(BmbString* path) {
    return bmb_read_file(path);
}

int64_t write_file(BmbString* path, BmbString* content) {
    return bmb_write_file(path, content);
}

int64_t append_file(BmbString* path, BmbString* content) {
    return bmb_append_file(path, content);
}

// StringBuilder wrappers
int64_t sb_new(void) {
    return bmb_sb_new();
}

int64_t sb_push(int64_t handle, BmbString* s) {
    return bmb_sb_push(handle, s);
}

int64_t sb_len(int64_t handle) {
    return bmb_sb_len(handle);
}

BmbString* sb_build(int64_t handle) {
    return bmb_sb_build(handle);
}

int64_t sb_clear(int64_t handle) {
    return bmb_sb_clear(handle);
}

// Print string wrapper
void print_str(BmbString* s) {
    bmb_print_str(s);
}

// Print string with newline wrapper (v0.50.49)
void println_str(BmbString* s) {
    bmb_println_str(s);
}

// ===================================================
// Command-line Argument Runtime Functions (v0.31.23)
// Phase 32.3.G: CLI Independence
// ===================================================

// Global storage for command-line arguments
static int bmb_argc = 0;
static char** bmb_argv = NULL;

// Initialize argv (called from real main() wrapper)
void bmb_init_argv(int argc, char** argv) {
    bmb_argc = argc;
    bmb_argv = argv;
}

// Get argument count
int64_t arg_count(void) {
    return (int64_t)bmb_argc;
}

// bmb_ prefixed version for LLVM codegen
int64_t bmb_arg_count(void) {
    return (int64_t)bmb_argc;
}

// Get argument at index (returns empty string if out of bounds)
BmbString* get_arg(int64_t idx) {
    if (idx < 0 || idx >= bmb_argc || !bmb_argv) {
        return bmb_string_new("", 0);
    }
    return bmb_string_from_cstr(bmb_argv[idx]);
}

// bmb_ prefixed version for LLVM codegen
BmbString* bmb_get_arg(int64_t idx) {
    if (idx < 0 || idx >= bmb_argc || !bmb_argv) {
        return bmb_string_new("", 0);
    }
    return bmb_string_from_cstr(bmb_argv[idx]);
}

// ===================================================
// Hashmap Implementation (v0.50.64)
// Simple open-addressing hash table with linear probing
// Keys and values are i64
// ===================================================

#define HASHMAP_INITIAL_CAPACITY 16
#define HASHMAP_LOAD_FACTOR 0.75
#define HASHMAP_NOT_FOUND (INT64_MIN)
#define HASHMAP_EMPTY_KEY (INT64_MIN)
#define HASHMAP_DELETED_KEY (INT64_MIN + 1)

typedef struct {
    int64_t* keys;
    int64_t* values;
    int64_t capacity;
    int64_t size;
} BmbHashmap;

static inline uint64_t hashmap_hash(int64_t key) {
    // FNV-1a inspired hash
    uint64_t h = (uint64_t)key;
    h ^= h >> 33;
    h *= 0xff51afd7ed558ccdULL;
    h ^= h >> 33;
    h *= 0xc4ceb9fe1a85ec53ULL;
    h ^= h >> 33;
    return h;
}

static void hashmap_resize(BmbHashmap* map, int64_t new_cap) {
    int64_t* old_keys = map->keys;
    int64_t* old_values = map->values;
    int64_t old_cap = map->capacity;

    map->keys = (int64_t*)malloc(new_cap * sizeof(int64_t));
    map->values = (int64_t*)malloc(new_cap * sizeof(int64_t));
    map->capacity = new_cap;
    map->size = 0;

    // Initialize all slots as empty
    for (int64_t i = 0; i < new_cap; i++) {
        map->keys[i] = HASHMAP_EMPTY_KEY;
    }

    // Reinsert all existing entries
    for (int64_t i = 0; i < old_cap; i++) {
        if (old_keys[i] != HASHMAP_EMPTY_KEY && old_keys[i] != HASHMAP_DELETED_KEY) {
            uint64_t h = hashmap_hash(old_keys[i]);
            int64_t idx = h % new_cap;
            while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
                idx = (idx + 1) % new_cap;
            }
            map->keys[idx] = old_keys[i];
            map->values[idx] = old_values[i];
            map->size++;
        }
    }

    free(old_keys);
    free(old_values);
}

int64_t hashmap_new(void) {
    BmbHashmap* map = (BmbHashmap*)malloc(sizeof(BmbHashmap));
    map->capacity = HASHMAP_INITIAL_CAPACITY;
    map->size = 0;
    map->keys = (int64_t*)malloc(HASHMAP_INITIAL_CAPACITY * sizeof(int64_t));
    map->values = (int64_t*)malloc(HASHMAP_INITIAL_CAPACITY * sizeof(int64_t));

    for (int64_t i = 0; i < HASHMAP_INITIAL_CAPACITY; i++) {
        map->keys[i] = HASHMAP_EMPTY_KEY;
    }

    return (int64_t)(uintptr_t)map;
}

int64_t hashmap_insert(int64_t handle, int64_t key, int64_t value) {
    BmbHashmap* map = (BmbHashmap*)(uintptr_t)handle;

    // Resize if load factor exceeded
    if ((double)map->size / map->capacity >= HASHMAP_LOAD_FACTOR) {
        hashmap_resize(map, map->capacity * 2);
    }

    uint64_t h = hashmap_hash(key);
    int64_t idx = h % map->capacity;
    int64_t first_deleted = -1;

    while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
        if (map->keys[idx] == key) {
            // Key exists, update value
            map->values[idx] = value;
            return 0;
        }
        if (map->keys[idx] == HASHMAP_DELETED_KEY && first_deleted < 0) {
            first_deleted = idx;
        }
        idx = (idx + 1) % map->capacity;
    }

    // Insert at deleted slot if found, otherwise at empty slot
    if (first_deleted >= 0) {
        idx = first_deleted;
    }

    map->keys[idx] = key;
    map->values[idx] = value;
    map->size++;
    return 0;
}

int64_t hashmap_get(int64_t handle, int64_t key) {
    BmbHashmap* map = (BmbHashmap*)(uintptr_t)handle;

    uint64_t h = hashmap_hash(key);
    int64_t idx = h % map->capacity;

    while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
        if (map->keys[idx] == key) {
            return map->values[idx];
        }
        idx = (idx + 1) % map->capacity;
    }

    return HASHMAP_NOT_FOUND;
}

int64_t hashmap_remove(int64_t handle, int64_t key) {
    BmbHashmap* map = (BmbHashmap*)(uintptr_t)handle;

    uint64_t h = hashmap_hash(key);
    int64_t idx = h % map->capacity;

    while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
        if (map->keys[idx] == key) {
            int64_t old_value = map->values[idx];
            map->keys[idx] = HASHMAP_DELETED_KEY;
            map->size--;
            return old_value;
        }
        idx = (idx + 1) % map->capacity;
    }

    return HASHMAP_NOT_FOUND;
}

int64_t hashmap_len(int64_t handle) {
    BmbHashmap* map = (BmbHashmap*)(uintptr_t)handle;
    return map->size;
}

void hashmap_free(int64_t handle) {
    BmbHashmap* map = (BmbHashmap*)(uintptr_t)handle;
    free(map->keys);
    free(map->values);
    free(map);
}

// ===================================================
// Vector Runtime Functions (v0.50.70)
// Vec is a dynamic array: [ptr to data, len, cap]
// Header is 3 x i64 = 24 bytes
// ===================================================

// vec_push(vec_handle, value) - Push value to vector with auto-grow
void bmb_vec_push(int64_t vec_handle, int64_t value) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    int64_t ptr = header[0];
    int64_t len = header[1];
    int64_t cap = header[2];

    // Check if need to grow
    if (len >= cap) {
        int64_t new_cap = (cap == 0) ? 4 : cap * 2;
        int64_t new_size = new_cap * sizeof(int64_t);
        int64_t* new_data;
        if (ptr == 0) {
            new_data = (int64_t*)malloc(new_size);
        } else {
            new_data = (int64_t*)realloc((void*)(uintptr_t)ptr, new_size);
        }
        header[0] = (int64_t)(uintptr_t)new_data;
        header[2] = new_cap;
        ptr = header[0];
    }

    // Store value and increment len
    int64_t* data = (int64_t*)(uintptr_t)ptr;
    data[len] = value;
    header[1] = len + 1;
}

// ===================================================
// Entry Point Wrapper (v0.31.23)
// BMB's main() is renamed to bmb_user_main() in codegen
// This wrapper provides the real main() that initializes argv
// ===================================================

// Forward declaration of BMB user main
int64_t bmb_user_main(void);

// Real main entry point
int main(int argc, char** argv) {
    bmb_init_argv(argc, argv);
    return (int)bmb_user_main();
}

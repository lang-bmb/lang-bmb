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
// Memory Access Functions (v0.50.75)
// For direct memory read/write in BMB programs
// ===================================================

// store_i64(ptr, value) - Store 64-bit value at memory address
void bmb_store_i64(int64_t ptr, int64_t value) {
    *((int64_t*)ptr) = value;
}

// load_i64(ptr) - Load 64-bit value from memory address
int64_t bmb_load_i64(int64_t ptr) {
    return *((int64_t*)ptr);
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

// v0.51.14: String constant cache - avoids repeated allocation for string literals
// Uses C string pointer as key (string literals have constant addresses)
#define STRING_CONST_CACHE_SIZE 1024
typedef struct {
    const char* cstr_ptr;  // Key: pointer to C string literal
    BmbString* bmb_str;    // Value: cached BmbString
} StringConstEntry;
static StringConstEntry string_const_cache[STRING_CONST_CACHE_SIZE];
static int64_t string_const_count = 0;

// Lookup cached string by C string pointer
static BmbString* string_const_cache_get(const char* cstr) {
    // Linear search is fast for small caches and avoids hash collisions
    for (int64_t i = 0; i < string_const_count; i++) {
        if (string_const_cache[i].cstr_ptr == cstr) {
            return string_const_cache[i].bmb_str;
        }
    }
    return NULL;
}

// Add string to cache
static void string_const_cache_put(const char* cstr, BmbString* s) {
    if (string_const_count < STRING_CONST_CACHE_SIZE) {
        string_const_cache[string_const_count].cstr_ptr = cstr;
        string_const_cache[string_const_count].bmb_str = s;
        string_const_count++;
    }
}

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
// v0.51.14: Uses cache to avoid repeated allocation for the same literal
BmbString* bmb_string_from_cstr(const char* cstr) {
    // Check cache first (fast path for repeated string literal access)
    BmbString* cached = string_const_cache_get(cstr);
    if (cached) {
        return cached;
    }
    // Cache miss: allocate new string and cache it
    BmbString* s = bmb_string_new(cstr, strlen(cstr));
    string_const_cache_put(cstr, s);
    return s;
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
// v0.50.73: Refactored to single growable buffer for O(log n) allocations
// ===================================================

typedef struct {
    char* buffer;       // Single growable buffer
    int64_t length;     // Current content length
    int64_t capacity;   // Buffer capacity
} StringBuilder;

#define MAX_STRING_BUILDERS 8192
#define SB_INITIAL_CAPACITY 256  // Start with reasonable size to reduce reallocs
static StringBuilder* builders[MAX_STRING_BUILDERS];
static int64_t builder_count = 0;

// Helper: ensure StringBuilder has space for additional bytes
static void sb_ensure_capacity(StringBuilder* sb, int64_t additional) {
    int64_t required = sb->length + additional + 1;  // +1 for null terminator
    if (required > sb->capacity) {
        while (sb->capacity < required) {
            sb->capacity *= 2;
        }
        sb->buffer = (char*)realloc(sb->buffer, sb->capacity);
    }
}

// Create new StringBuilder, return handle (index)
int64_t bmb_sb_new(void) {
    if (builder_count >= MAX_STRING_BUILDERS) return -1;
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    sb->buffer = (char*)malloc(SB_INITIAL_CAPACITY);
    sb->buffer[0] = '\0';
    sb->length = 0;
    sb->capacity = SB_INITIAL_CAPACITY;
    builders[builder_count] = sb;
    return builder_count++;
}

// Push string to StringBuilder - direct append, no fragment allocation
int64_t bmb_sb_push(int64_t handle, BmbString* s) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb || !s) return -1;

    sb_ensure_capacity(sb, s->len);
    memcpy(sb->buffer + sb->length, s->data, s->len);
    sb->length += s->len;
    sb->buffer[sb->length] = '\0';
    return 0;
}

// v0.50.77: Push C string literal directly - ZERO ALLOCATION
// Used for string literals passed to sb_push, avoiding bmb_string_from_cstr overhead
int64_t bmb_sb_push_cstr(int64_t handle, const char* cstr) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb || !cstr) return -1;

    int64_t len = strlen(cstr);
    sb_ensure_capacity(sb, len);
    memcpy(sb->buffer + sb->length, cstr, len);
    sb->length += len;
    sb->buffer[sb->length] = '\0';
    return 0;
}

// Push single character to StringBuilder - direct append
int64_t bmb_sb_push_char(int64_t handle, int64_t char_code) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb) return -1;

    sb_ensure_capacity(sb, 1);
    sb->buffer[sb->length++] = (char)char_code;
    sb->buffer[sb->length] = '\0';
    return 0;
}

// v0.50.74: Push JSON-escaped string to StringBuilder in one call
// Eliminates per-character function call overhead (0 calls vs 3-4 per char)
int64_t bmb_sb_push_escaped(int64_t handle, BmbString* s) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb || !s) return -1;

    // Worst case: every char becomes \uXXXX (6 chars)
    sb_ensure_capacity(sb, s->len * 6);

    for (int64_t i = 0; i < s->len; i++) {
        unsigned char c = (unsigned char)s->data[i];
        if (c == '"') {
            sb->buffer[sb->length++] = '\\';
            sb->buffer[sb->length++] = '"';
        } else if (c == '\\') {
            sb->buffer[sb->length++] = '\\';
            sb->buffer[sb->length++] = '\\';
        } else if (c == '\n') {
            sb->buffer[sb->length++] = '\\';
            sb->buffer[sb->length++] = 'n';
        } else if (c == '\r') {
            sb->buffer[sb->length++] = '\\';
            sb->buffer[sb->length++] = 'r';
        } else if (c == '\t') {
            sb->buffer[sb->length++] = '\\';
            sb->buffer[sb->length++] = 't';
        } else if (c >= 32 && c < 127) {
            // Printable ASCII - direct copy
            sb->buffer[sb->length++] = c;
        } else {
            // \uXXXX escape for other bytes
            static const char hex[] = "0123456789abcdef";
            sb->buffer[sb->length++] = '\\';
            sb->buffer[sb->length++] = 'u';
            sb->buffer[sb->length++] = '0';
            sb->buffer[sb->length++] = '0';
            sb->buffer[sb->length++] = hex[(c >> 4) & 0xF];
            sb->buffer[sb->length++] = hex[c & 0xF];
        }
    }
    sb->buffer[sb->length] = '\0';
    return 0;
}

// v0.50.73: Push integer directly to StringBuilder
int64_t bmb_sb_push_int(int64_t handle, int64_t n) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb) return -1;

    // Convert integer to string (max 20 digits for int64 + sign)
    char buf[24];
    int len = 0;
    int64_t abs_n = n;
    int negative = 0;

    if (n < 0) {
        negative = 1;
        abs_n = -n;
    }

    // Handle zero case
    if (abs_n == 0) {
        buf[len++] = '0';
    } else {
        // Build digits in reverse
        char temp[24];
        int temp_len = 0;
        while (abs_n > 0) {
            temp[temp_len++] = '0' + (abs_n % 10);
            abs_n /= 10;
        }
        // Add sign if negative
        if (negative) {
            buf[len++] = '-';
        }
        // Reverse digits into buf
        for (int i = temp_len - 1; i >= 0; i--) {
            buf[len++] = temp[i];
        }
    }
    buf[len] = '\0';

    // Append to buffer directly (no fragment allocation)
    sb_ensure_capacity(sb, len);
    memcpy(sb->buffer + sb->length, buf, len);
    sb->length += len;
    sb->buffer[sb->length] = '\0';
    return 0;
}

// Get total length - now O(1) instead of O(n)
int64_t bmb_sb_len(int64_t handle) {
    if (handle < 0 || handle >= builder_count) return 0;
    StringBuilder* sb = builders[handle];
    if (!sb) return 0;
    return sb->length;
}

// Build final string - minimal allocation, just wrap buffer
BmbString* bmb_sb_build(int64_t handle) {
    if (handle < 0 || handle >= builder_count) return bmb_string_new("", 0);
    StringBuilder* sb = builders[handle];
    if (!sb) return bmb_string_new("", 0);

    // Copy buffer to new string (we can't transfer ownership as sb may be reused)
    char* data = (char*)malloc(sb->length + 1);
    memcpy(data, sb->buffer, sb->length + 1);

    BmbString* result = (BmbString*)malloc(sizeof(BmbString));
    result->data = data;
    result->len = sb->length;
    result->cap = sb->length + 1;
    if (string_pool_count < MAX_STRINGS) {
        string_pool[string_pool_count++] = result;
    }
    return result;
}

// Clear StringBuilder - reset length, keep buffer (reuse memory)
int64_t bmb_sb_clear(int64_t handle) {
    if (handle < 0 || handle >= builder_count) return -1;
    StringBuilder* sb = builders[handle];
    if (!sb) return -1;
    sb->length = 0;
    sb->buffer[0] = '\0';
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

// v0.50.77: Push C string literal directly (wrapper)
int64_t sb_push_cstr(int64_t handle, const char* cstr) {
    return bmb_sb_push_cstr(handle, cstr);
}

int64_t sb_push_char(int64_t handle, int64_t char_code) {
    return bmb_sb_push_char(handle, char_code);
}

// v0.50.73: Push integer directly (wrapper)
int64_t sb_push_int(int64_t handle, int64_t n) {
    return bmb_sb_push_int(handle, n);
}

// v0.50.74: Push JSON-escaped string (wrapper)
int64_t sb_push_escaped(int64_t handle, BmbString* s) {
    return bmb_sb_push_escaped(handle, s);
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
            int64_t idx = h & (new_cap - 1);
            while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
                idx = (idx + 1) & (new_cap - 1);
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
    int64_t idx = h & (map->capacity - 1);
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
        idx = (idx + 1) & (map->capacity - 1);
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
    int64_t idx = h & (map->capacity - 1);

    while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
        if (map->keys[idx] == key) {
            return map->values[idx];
        }
        idx = (idx + 1) & (map->capacity - 1);
    }

    return HASHMAP_NOT_FOUND;
}

int64_t hashmap_remove(int64_t handle, int64_t key) {
    BmbHashmap* map = (BmbHashmap*)(uintptr_t)handle;

    uint64_t h = hashmap_hash(key);
    int64_t idx = h & (map->capacity - 1);

    while (map->keys[idx] != HASHMAP_EMPTY_KEY) {
        if (map->keys[idx] == key) {
            int64_t old_value = map->values[idx];
            map->keys[idx] = HASHMAP_DELETED_KEY;
            map->size--;
            return old_value;
        }
        idx = (idx + 1) & (map->capacity - 1);
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
// Vector Runtime Functions (v0.50.75)
// Vec is a dynamic array: [ptr to data, len, cap]
// Header is 3 x i64 = 24 bytes
// ===================================================

// vec_new() - Create new empty vector, returns handle (pointer to header)
int64_t bmb_vec_new(void) {
    int64_t* header = (int64_t*)malloc(24);  // 3 x i64
    header[0] = 0;  // ptr = null
    header[1] = 0;  // len = 0
    header[2] = 0;  // cap = 0
    return (int64_t)(uintptr_t)header;
}

// vec_with_capacity(cap) - Create vector with pre-allocated capacity
int64_t bmb_vec_with_capacity(int64_t cap) {
    int64_t* header = (int64_t*)malloc(24);
    int64_t* data = (cap > 0) ? (int64_t*)malloc(cap * sizeof(int64_t)) : NULL;
    header[0] = (int64_t)(uintptr_t)data;
    header[1] = 0;  // len = 0
    header[2] = cap;
    return (int64_t)(uintptr_t)header;
}

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

// vec_pop(vec_handle) - Remove and return last element
int64_t bmb_vec_pop(int64_t vec_handle) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    int64_t len = header[1];
    if (len == 0) return 0;  // Empty vector
    int64_t* data = (int64_t*)(uintptr_t)header[0];
    header[1] = len - 1;
    return data[len - 1];
}

// vec_get(vec_handle, index) - Get element at index
int64_t bmb_vec_get(int64_t vec_handle, int64_t index) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    int64_t* data = (int64_t*)(uintptr_t)header[0];
    return data[index];
}

// vec_set(vec_handle, index, value) - Set element at index
void bmb_vec_set(int64_t vec_handle, int64_t index, int64_t value) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    int64_t* data = (int64_t*)(uintptr_t)header[0];
    data[index] = value;
}

// vec_len(vec_handle) - Get length
int64_t bmb_vec_len(int64_t vec_handle) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    return header[1];
}

// vec_cap(vec_handle) - Get capacity
int64_t bmb_vec_cap(int64_t vec_handle) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    return header[2];
}

// vec_free(vec_handle) - Free vector and its data
void bmb_vec_free(int64_t vec_handle) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    int64_t ptr = header[0];
    if (ptr != 0) {
        free((void*)(uintptr_t)ptr);
    }
    free(header);
}

// vec_clear(vec_handle) - Clear all elements (keep capacity)
void bmb_vec_clear(int64_t vec_handle) {
    int64_t* header = (int64_t*)(uintptr_t)vec_handle;
    header[1] = 0;  // Set len to 0
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

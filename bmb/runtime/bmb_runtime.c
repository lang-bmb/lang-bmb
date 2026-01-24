#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

// BMB Runtime Library
void bmb_println_i64(int64_t n) { printf("%ld\n", n); }
void bmb_print_i64(int64_t n) { printf("%ld", n); }
int64_t bmb_read_int() { int64_t n; scanf("%ld", &n); return n; }
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

// v0.97: String functions
void bmb_print_str(const char* s) { printf("%s", s); }
void bmb_println_str(const char* s) { printf("%s\n", s); }
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

// v0.100: String concatenation
char* bmb_string_concat(const char* a, const char* b) {
    if (!a || !b) {
        char* result = (char*)malloc(1);
        result[0] = '\0';
        return result;
    }
    size_t len_a = 0, len_b = 0;
    while (a[len_a]) len_a++;
    while (b[len_b]) len_b++;
    char* result = (char*)malloc(len_a + len_b + 1);
    for (size_t i = 0; i < len_a; i++) result[i] = a[i];
    for (size_t i = 0; i < len_b; i++) result[len_a + i] = b[i];
    result[len_a + len_b] = '\0';
    return result;
}

// v0.46: String functions for native compilation
// Convert C string to BmbString (identity in our simple runtime)
char* bmb_string_from_cstr(const char* s) {
    return (char*)s;
}

// Create new string with given length (allocates copy)
char* bmb_string_new(const char* s, int64_t len) {
    char* result = (char*)malloc(len + 1);
    for (int64_t i = 0; i < len; i++) result[i] = s[i];
    result[len] = '\0';
    return result;
}

// String length (alias for bmb_str_len with ptr signature)
int64_t bmb_string_len(const char* s) {
    if (!s) return 0;
    int64_t len = 0;
    while (s[len]) len++;
    return len;
}

// Get byte at index (NOT Unicode character)
int64_t bmb_string_char_at(const char* s, int64_t index) {
    if (!s || index < 0) return 0;
    return (int64_t)(unsigned char)s[index];
}

// String equality comparison
int64_t bmb_string_eq(const char* a, const char* b) {
    if (a == b) return 1;  // Same pointer
    if (!a || !b) return 0;  // One is null
    while (*a && *b) {
        if (*a != *b) return 0;
        a++; b++;
    }
    return (*a == *b) ? 1 : 0;  // Both should be '\0'
}

// String slice (substring from start to end, exclusive)
char* bmb_string_slice(const char* s, int64_t start, int64_t end) {
    if (!s || start < 0 || end < start) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return empty;
    }
    int64_t len = end - start;
    char* result = (char*)malloc(len + 1);
    for (int64_t i = 0; i < len; i++) {
        result[i] = s[start + i];
    }
    result[len] = '\0';
    return result;
}

// v0.46: Wrapper functions (LLVM codegen uses short names)
// These call the bmb_string_* functions with matching signatures
char* slice(const char* s, int64_t start, int64_t end) {
    return bmb_string_slice(s, start, end);
}

int64_t byte_at(const char* s, int64_t index) {
    return bmb_string_char_at(s, index);
}

// v0.50.18: Additional wrapper functions for bootstrap compiler
int64_t len(const char* s) {
    return bmb_string_len(s);
}

char* chr(int64_t n) {
    return bmb_chr(n);
}

char* char_to_string(int32_t c) {
    return bmb_char_to_string(c);
}

int64_t ord(const char* s) {
    return bmb_ord(s);
}

void print_str(const char* s) {
    bmb_print_str(s);
}

void println(int64_t n) {
    bmb_println_i64(n);
}

void println_str(const char* s) {
    bmb_println_str(s);
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

int64_t bmb_sb_push(int64_t handle, const char* s) {
    if (!s || !handle) return 0;
    StringBuilder* sb = (StringBuilder*)handle;
    int64_t slen = 0;
    while (s[slen]) slen++;

    // Grow if needed
    while (sb->len + slen + 1 > sb->cap) {
        sb->cap *= 2;
        sb->data = (char*)realloc(sb->data, sb->cap);
    }

    // Append
    for (int64_t i = 0; i < slen; i++) {
        sb->data[sb->len + i] = s[i];
    }
    sb->len += slen;
    sb->data[sb->len] = '\0';
    return sb->len;
}

int64_t bmb_sb_len(int64_t handle) {
    StringBuilder* sb = (StringBuilder*)handle;
    return sb->len;
}

char* bmb_sb_build(int64_t handle) {
    if (!handle) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return empty;
    }
    StringBuilder* sb = (StringBuilder*)handle;
    // Return copy of the built string
    char* result = (char*)malloc(sb->len + 1);
    for (int64_t i = 0; i <= sb->len; i++) {
        result[i] = sb->data[i];
    }
    return result;
}

int64_t bmb_sb_clear(int64_t handle) {
    StringBuilder* sb = (StringBuilder*)handle;
    sb->len = 0;
    sb->data[0] = '\0';
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

char* bmb_read_file(const char* path) {
    FILE* f = fopen(path, "rb");
    if (!f) {
        char* empty = (char*)malloc(1);
        empty[0] = '\0';
        return empty;
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
    return content ? content : "";
}

int64_t bmb_write_file(const char* path, const char* content) {
    FILE* f = fopen(path, "wb");
    if (!f) return -1;
    size_t len = strlen(content);
    size_t written = fwrite(content, 1, len, f);
    fclose(f);
    return (written == len) ? 0 : -1;
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

// v0.50.20: String operation wrappers for bootstrap compiler
char* str_concat(const char* a, const char* b) {
    return bmb_string_concat(a, b);
}

int64_t str_eq(const char* a, const char* b) {
    return bmb_string_eq(a, b);
}

// v0.50.20: StringBuilder wrappers
int64_t sb_new(void) {
    return bmb_sb_new();
}

int64_t sb_push(int64_t handle, const char* s) {
    return bmb_sb_push(handle, s);
}

char* sb_build(int64_t handle) {
    return bmb_sb_build(handle);
}

int64_t sb_len(int64_t handle) {
    return bmb_sb_len(handle);
}

int64_t sb_clear(int64_t handle) {
    return bmb_sb_clear(handle);
}

// v0.50.20: File I/O wrappers
char* read_file(const char* path) {
    return bmb_read_file(path);
}

int64_t write_file(const char* path, const char* content) {
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

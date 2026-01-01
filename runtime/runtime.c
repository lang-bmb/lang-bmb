// BMB Runtime Library
// Provides basic I/O functions for BMB programs

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

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

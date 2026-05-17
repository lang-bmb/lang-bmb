/**
 * bmb-json C bindings — test suite
 *
 * Tests all 12 exported functions. Exit code 0 = all pass.
 *
 * Arena rule: @export String return values are arena-backed.
 *   - Read data before bmb_ffi_end().
 *   - DO NOT call bmb_ffi_free_string on output strings.
 *   - Only free inputs from bmb_ffi_cstr_to_string.
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_json.h"

static int g_pass = 0, g_fail = 0;

#define ASSERT_EQ(label, got, expected) do { \
    int64_t _g = (int64_t)(got); \
    int64_t _e = (int64_t)(expected); \
    if (_g == _e) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: got %lld, expected %lld\n", label, (long long)_g, (long long)_e); g_fail++; \
    } \
} while(0)

#define ASSERT_STR(label, got, expected) do { \
    if (strcmp((got), (expected)) == 0) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: got '%s', expected '%s'\n", label, (got), (expected)); g_fail++; \
    } \
} while(0)

#define ASSERT_TRUE(label, cond) do { \
    if (cond) { \
        printf("[PASS] %s\n", label); g_pass++; \
    } else { \
        printf("[FAIL] %s: condition false\n", label); g_fail++; \
    } \
} while(0)

#define FFI_END_CHECK(label) do { \
    if (bmb_ffi_has_error()) { \
        printf("[FAIL] FFI error in %s: %s\n", label, bmb_ffi_error_message()); g_fail++; \
    } \
    bmb_ffi_end(); \
} while(0)

static const char *OBJ  = "{\"name\":\"Alice\",\"age\":30,\"active\":true,\"score\":9}";
static const char *ARR  = "[10,20,30,40,50]";
static const char *STR  = "\"hello\"";
static const char *NUM  = "42";
static const char *BOOL = "true";
static const char *NUL  = "null";
static const char *BAD  = "{bad json}";

/* ── Validation ──────────────────────────────────────────────────────── */
static void test_validate(void) {
    bmb_ffi_begin();
    ASSERT_EQ("validate(object)=1", bmb_json_validate(bmb_ffi_cstr_to_string(OBJ)),  1);
    ASSERT_EQ("validate(array)=1",  bmb_json_validate(bmb_ffi_cstr_to_string(ARR)),  1);
    ASSERT_EQ("validate(number)=1", bmb_json_validate(bmb_ffi_cstr_to_string(NUM)),  1);
    ASSERT_EQ("validate(bool)=1",   bmb_json_validate(bmb_ffi_cstr_to_string(BOOL)), 1);
    ASSERT_EQ("validate(null)=1",   bmb_json_validate(bmb_ffi_cstr_to_string(NUL)),  1);
    ASSERT_EQ("validate(bad)=0",    bmb_json_validate(bmb_ffi_cstr_to_string(BAD)),  0);
    FFI_END_CHECK("validate");
}

/* ── Type detection ──────────────────────────────────────────────────── */
static void test_type(void) {
    bmb_ffi_begin();
    ASSERT_STR("type(object)=object",
               bmb_ffi_string_data(bmb_json_type(bmb_ffi_cstr_to_string(OBJ))),  "object");
    ASSERT_STR("type(array)=array",
               bmb_ffi_string_data(bmb_json_type(bmb_ffi_cstr_to_string(ARR))),  "array");
    ASSERT_STR("type(string)=string",
               bmb_ffi_string_data(bmb_json_type(bmb_ffi_cstr_to_string(STR))),  "string");
    ASSERT_STR("type(number)=number",
               bmb_ffi_string_data(bmb_json_type(bmb_ffi_cstr_to_string(NUM))),  "number");
    ASSERT_STR("type(bool)=bool",
               bmb_ffi_string_data(bmb_json_type(bmb_ffi_cstr_to_string(BOOL))), "bool");
    ASSERT_STR("type(null)=null",
               bmb_ffi_string_data(bmb_json_type(bmb_ffi_cstr_to_string(NUL))),  "null");
    FFI_END_CHECK("type");
}

/* ── Object access ───────────────────────────────────────────────────── */
static void test_object(void) {
    bmb_ffi_begin();
    void *obj    = bmb_ffi_cstr_to_string(OBJ);
    void *k_name = bmb_ffi_cstr_to_string("name");
    void *k_age  = bmb_ffi_cstr_to_string("age");
    void *k_act  = bmb_ffi_cstr_to_string("active");
    void *k_miss = bmb_ffi_cstr_to_string("missing");

    /* get returns JSON representation (string with quotes) */
    ASSERT_STR("json_get(name)=Alice(json)",
               bmb_ffi_string_data(bmb_json_get(obj, k_name)), "\"Alice\"");
    /* get_string returns raw string (no quotes) */
    ASSERT_STR("json_get_string(name)=Alice",
               bmb_ffi_string_data(bmb_json_get_string(obj, k_name)), "Alice");
    ASSERT_EQ("json_get_number(age)=30",
              bmb_json_get_number(obj, k_age), 30);
    ASSERT_EQ("json_get_bool(active)=1",
              bmb_json_get_bool(obj, k_act), 1);
    ASSERT_EQ("json_get_bool(missing)=-1",
              bmb_json_get_bool(obj, k_miss), -1);
    ASSERT_EQ("json_has_key(name)=1",
              bmb_json_has_key(obj, k_name), 1);
    ASSERT_EQ("json_has_key(missing)=0",
              bmb_json_has_key(obj, k_miss), 0);
    ASSERT_EQ("json_object_len=4",
              bmb_json_object_len(obj), 4);
    /* json_count counts all nodes recursively (root + keys + values) */
    ASSERT_EQ("json_count=9",
              bmb_json_count(obj), 9); /* 1 root + 4 keys + 4 values */

    bmb_ffi_free_string(obj);
    bmb_ffi_free_string(k_name); bmb_ffi_free_string(k_age);
    bmb_ffi_free_string(k_act);  bmb_ffi_free_string(k_miss);
    FFI_END_CHECK("object");
}

/* ── Array access ────────────────────────────────────────────────────── */
static void test_array(void) {
    bmb_ffi_begin();
    void *arr = bmb_ffi_cstr_to_string(ARR);

    ASSERT_EQ("json_array_len=5",     bmb_json_array_len(arr), 5);
    ASSERT_EQ("json_array_len(obj)=-1",
              bmb_json_array_len(bmb_ffi_cstr_to_string(OBJ)), -1);
    ASSERT_STR("json_array_get(0)=10",
               bmb_ffi_string_data(bmb_json_array_get(arr, 0)), "10");
    ASSERT_STR("json_array_get(4)=50",
               bmb_ffi_string_data(bmb_json_array_get(arr, 4)), "50");
    ASSERT_EQ("json_count(array)=6",  bmb_json_count(arr), 6); /* 1 root + 5 elements */

    bmb_ffi_free_string(arr);
    FFI_END_CHECK("array");
}

/* ── Stringify ───────────────────────────────────────────────────────── */
static void test_stringify(void) {
    bmb_ffi_begin();
    void *raw = bmb_ffi_cstr_to_string("{\"x\":1,\"y\":2}");
    void *out = bmb_json_stringify(raw);
    ASSERT_TRUE("stringify non-empty", strlen(bmb_ffi_string_data(out)) > 0);
    ASSERT_EQ("validate(stringified)=1",
              bmb_json_validate(bmb_ffi_cstr_to_string(
                  bmb_ffi_string_data(out))), 1);
    bmb_ffi_free_string(raw);
    FFI_END_CHECK("stringify");
}

int main(void) {
    printf("=== bmb-json C binding tests ===\n\n");

    test_validate();
    test_type();
    test_object();
    test_array();
    test_stringify();

    printf("\n=== Results: %d passed, %d failed ===\n", g_pass, g_fail);
    return g_fail > 0 ? 1 : 0;
}

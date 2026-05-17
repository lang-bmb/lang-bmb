/**
 * bmb-json C bindings — example usage
 *
 * Build:
 *   Windows: gcc -O2 -I../../include -o example example.c -L../.. -l:bmb_json.dll
 *   Linux:   gcc -O2 -I../../include -o example example.c -L../.. -lbmb_json -Wl,-rpath,../..
 */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "bmb_json.h"

/*
 * Arena rule: @export String return values are arena-backed.
 * Read data before bmb_ffi_end(). DO NOT call bmb_ffi_free_string on outputs.
 * Only free inputs from bmb_ffi_cstr_to_string.
 */

int main(void) {
    const char *obj_json  = "{\"name\":\"Alice\",\"age\":30,\"active\":true}";
    const char *arr_json  = "[1,2,3,4,5]";
    const char *num_json  = "42";
    const char *bool_json = "true";
    const char *null_json = "null";

    /* ── Validation & type ──────────────────────────────────── */
    bmb_ffi_begin();
    void *obj = bmb_ffi_cstr_to_string(obj_json);
    void *arr = bmb_ffi_cstr_to_string(arr_json);

    printf("validate(object) = %lld\n", (long long)bmb_json_validate(obj));
    printf("validate(array)  = %lld\n", (long long)bmb_json_validate(arr));
    printf("validate(bad)    = %lld\n",
           (long long)bmb_json_validate(bmb_ffi_cstr_to_string("{bad}")));

    printf("type(object)  = %s\n", bmb_ffi_string_data(bmb_json_type(obj)));
    printf("type(array)   = %s\n", bmb_ffi_string_data(bmb_json_type(arr)));
    printf("type(number)  = %s\n", bmb_ffi_string_data(
        bmb_json_type(bmb_ffi_cstr_to_string(num_json))));
    printf("type(bool)    = %s\n", bmb_ffi_string_data(
        bmb_json_type(bmb_ffi_cstr_to_string(bool_json))));
    printf("type(null)    = %s\n", bmb_ffi_string_data(
        bmb_json_type(bmb_ffi_cstr_to_string(null_json))));
    bmb_ffi_free_string(obj);
    bmb_ffi_free_string(arr);
    bmb_ffi_end();

    /* ── Object access ──────────────────────────────────────── */
    bmb_ffi_begin();
    obj = bmb_ffi_cstr_to_string(obj_json);
    void *k_name   = bmb_ffi_cstr_to_string("name");
    void *k_age    = bmb_ffi_cstr_to_string("age");
    void *k_active = bmb_ffi_cstr_to_string("active");

    printf("get(name)        = %s\n", bmb_ffi_string_data(bmb_json_get(obj, k_name)));
    printf("get_string(name) = %s\n", bmb_ffi_string_data(bmb_json_get_string(obj, k_name)));
    printf("get_number(age)  = %lld\n", (long long)bmb_json_get_number(obj, k_age));
    printf("get_bool(active) = %lld\n", (long long)bmb_json_get_bool(obj, k_active));
    printf("has_key(name)    = %lld\n", (long long)bmb_json_has_key(obj, k_name));
    printf("has_key(missing) = %lld\n",
           (long long)bmb_json_has_key(obj, bmb_ffi_cstr_to_string("missing")));
    printf("object_len       = %lld\n", (long long)bmb_json_object_len(obj));
    printf("count(object)    = %lld\n", (long long)bmb_json_count(obj));

    bmb_ffi_free_string(obj);
    bmb_ffi_free_string(k_name);
    bmb_ffi_free_string(k_age);
    bmb_ffi_free_string(k_active);
    bmb_ffi_end();

    /* ── Array access ───────────────────────────────────────── */
    bmb_ffi_begin();
    arr = bmb_ffi_cstr_to_string(arr_json);

    printf("array_len        = %lld\n", (long long)bmb_json_array_len(arr));
    printf("array_get(0)     = %s\n",   bmb_ffi_string_data(bmb_json_array_get(arr, 0)));
    printf("array_get(4)     = %s\n",   bmb_ffi_string_data(bmb_json_array_get(arr, 4)));
    printf("count(array)     = %lld\n", (long long)bmb_json_count(arr));

    bmb_ffi_free_string(arr);
    bmb_ffi_end();

    /* ── Stringify (roundtrip normalization) ─────────────────── */
    bmb_ffi_begin();
    void *raw = bmb_ffi_cstr_to_string("{  \"x\":  1,  \"y\":  2 }");
    void *normalized = bmb_json_stringify(raw);
    printf("stringify        = %s\n", bmb_ffi_string_data(normalized));
    bmb_ffi_free_string(raw);
    bmb_ffi_end();

    printf("All examples passed.\n");
    return 0;
}

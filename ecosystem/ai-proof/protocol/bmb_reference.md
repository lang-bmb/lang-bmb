# BMB Language Quick Reference

## Basics
```bmb
fn main() -> i64 = { /* body */ 0 };
fn add(a: i64, b: i64) -> i64 = a + b;
```

## Variables
```bmb
let x: i64 = 10;
let mut y: i64 = 0;
set y = 5;          // reassign mutable
```

## Types
`i64`, `f64`, `bool`, `&str`

## Control Flow
```bmb
if x > 0 { x } else { 0 }           // expression
while cond { set i = i + 1; }
```

## I/O
```bmb
println(42);             // print i64 + newline
print(42);               // print i64, no newline
println_str("hello");    // print string + newline
let n: i64 = read_int(); // read i64 from stdin
```

## Dynamic Arrays (vec)
```bmb
let v: i64 = vec_new();
vec_push(v, 42);
let val: i64 = vec_get(v, 0);   // 0-indexed
let len: i64 = vec_len(v);
vec_set(v, idx, val);
vec_free(v);
```

## Contracts
```bmb
fn safe_get(arr: i64, idx: i64, len: i64) -> i64
    pre idx >= 0 and idx < len
    post ret >= 0
= vec_get(arr, idx);
```

## Functions
```bmb
fn factorial(n: i64) -> i64 =
    if n <= 1 { 1 } else { n * factorial(n - 1) };
```

## Pattern: Read array from stdin
```bmb
fn read_array(n: i64) -> i64 = {
    let arr: i64 = vec_new();
    let mut i: i64 = 0;
    while i < n {
        vec_push(arr, read_int());
        set i = i + 1;
    };
    arr
};
```

## Pattern: Main with stdin
```bmb
fn main() -> i64 = {
    let n: i64 = read_int();
    let arr: i64 = read_array(n);
    // ... process ...
    let _r: i64 = println(result);
    vec_free(arr);
    0
};
```

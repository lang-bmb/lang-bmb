# BMB Language Quick Reference

## Basics
```bmb
fn main() -> i64 = { /* body */ 0 };
fn add(a: i64, b: i64) -> i64 = a + b;
```

## Variables
```bmb
let x: i64 = 10;          // explicit type
let y = 42;                // type inference
let mut z: i64 = 0;
z = 5;                     // reassignment (set z = 5 also works)
```

## Types
`i64`, `f64`, `bool`, `&str`

## Control Flow
```bmb
if x > 0 { x } else { 0 }           // expression — returns value
while cond { body; }                 // while loop
for i in 0..n { body; }             // for loop with range
break;                               // exit loop early
continue;                            // skip to next iteration
return expr;                         // early return from function
```

## I/O
```bmb
println(42);             // print i64 + newline
print(42);               // print i64, no newline
println_str("hello");    // print string + newline
print_str(" ");          // print string, no newline
let n: i64 = read_int(); // read i64 from stdin
```

## Dynamic Arrays (vec)
```bmb
let v: i64 = vec_new();
vec_push(v, 42);
let val: i64 = vec_get(v, 0);   // 0-indexed
let len: i64 = vec_len(v);
vec_set(v, idx, val);
vec_pop(v);              // remove last element
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

## if-else Rules
```bmb
// if used as value: MUST have else
let x: i64 = if a > b { a } else { b };

// if used as statement: MUST have else { () }
if count > 0 {
    println(count);
    ()
} else { () };
```

## Pattern: Read array from stdin
```bmb
fn read_array(n: i64) -> i64 = {
    let arr: i64 = vec_new();
    for i in 0..n {
        vec_push(arr, read_int());
        ()
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
    println(result);
    vec_free(arr);
    0
};
```

## Pattern: Print space-separated array
```bmb
for i in 0..n {
    if i > 0 { print_str(" "); () } else { () };
    print(vec_get(arr, i));
    ()
};
println_str("");
```

## Common Pitfalls
- `println()` returns `()`, not `i64` — wrap: `let _r = println(x);`
- `vec_push()` returns `()` — wrap: `let _p = vec_push(v, val);`
- No closures, iterators, or method calls (use free functions)
- Blocks end with `;` after `}` in while/if/for contexts
- No `impl` blocks — use free functions
- Vec handle type is `i64`, not `Vec<T>`

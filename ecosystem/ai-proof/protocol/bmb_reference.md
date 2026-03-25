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
if x > 0 { x } else { 0 }           // expression — returns value
while cond { set i = i + 1; }       // statement — requires ; after }
```

## Important: No for loops, no break, no continue, no return
```bmb
// WRONG: for i in 0..n { ... }
// CORRECT:
let mut i: i64 = 0;
while i < n {
    // body
    set i = i + 1
};

// WRONG: while true { if done { break; } }
// CORRECT (use flag variable):
let mut running: i64 = 1;
while running == 1 {
    if done_cond { set running = 0 } else { /* continue body */ }
};
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

## Negative Numbers
```bmb
// WRONG: let x: i64 = -1;
// CORRECT:
let x: i64 = 0 - 1;
```

## if-else Rules
```bmb
// if used as value: MUST have else
let x: i64 = if a > b { a } else { b };

// if used as statement: MUST have else { () }
if count > 0 {
    let _p = println(count);
    ()
} else { () };
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

## Pattern: Print space-separated array
```bmb
let mut i: i64 = 0;
while i < n {
    if i > 0 {
        let _s = print_str(" ");
        ()
    } else { () };
    let _p = print(vec_get(arr, i));
    set i = i + 1
};
let _nl = println_str("");
```

## Pattern: Simulate break in while loop
```bmb
let mut found: i64 = 0;
let mut i: i64 = 0;
while i < n {
    if found == 0 {
        if vec_get(arr, i) == target {
            set found = 1
        } else {
            set i = i + 1
        }
    } else {
        set i = n  // force exit
    }
};
```

## Common Pitfalls
- `println()` returns `()`, not `i64` — wrap: `let _r: i64 = println(x);`
- `vec_push()` returns `()` — wrap: `let _p = vec_push(v, val);`
- All `let` bindings need explicit type annotations
- `set` keyword required for reassignment (not `=`)
- No `for`, `break`, `continue`, `return` keywords
- No closures, iterators, or method calls
- Blocks end with `;` after `}` in while/if contexts

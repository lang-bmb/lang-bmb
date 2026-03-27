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
z = 5;                     // reassignment
```

## Types
`i64`, `f64`, `bool`, `&str`

## Control Flow
```bmb
if x > 0 { x } else { 0 }           // expression — returns value
while cond { body; }                 // while loop
for i in 0..n { body; }             // for loop with range [0, n)
break;                               // exit loop early
continue;                            // skip to next iteration
return expr;                         // early return from function
```

## I/O
```bmb
let _p = println(42);           // print i64 + newline
let _p = print(42);             // print i64, no newline
let _p = println_str("hello");  // print string + newline
let _p = print_str(" ");        // print string, no newline
let n = read_int();              // read i64 from stdin
```

## Dynamic Arrays (vec)
```bmb
let v = vec_new();
let _p = vec_push(v, 42);       // append
let val = vec_get(v, 0);        // read at index (0-indexed)
let _s = vec_set(v, idx, val);  // write at index
let len = vec_len(v);           // length
let _p = vec_pop(v);            // remove last
let _f = vec_free(v);           // deallocate
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

## CRITICAL: if-else Rules
```bmb
// if as VALUE: MUST have else
let x = if a > b { a } else { b };

// if as STATEMENT: MUST have else { () }
if count > 0 {
    let _p = println(count);
    ()
} else { () };
```

## CRITICAL: All function calls need let binding
```bmb
// WRONG: println(42);      — type error
// RIGHT:
let _p = println(42);
let _s = vec_push(v, val);
let _f = vec_free(v);
```

## Pattern: Read array
```bmb
let n = read_int();
let v = vec_new();
for _i in 0..n {
    let val = read_int();
    let _p = vec_push(v, val)
};
```

## Pattern: Print space-separated array
```bmb
for i in 0..n {
    if i > 0 { let _s = print_str(" "); () } else { () };
    let _p = print(vec_get(v, i))
};
let _nl = println_str("");
```

## Pattern: Update vec element (v[i] += k)
```bmb
// There is no v[i] syntax for vec. Use vec_get/vec_set:
let _s = vec_set(v, i, vec_get(v, i) + k);
```

## Pattern: Swap two vec elements
```bmb
let tmp = vec_get(v, i);
let _s1 = vec_set(v, i, vec_get(v, j));
let _s2 = vec_set(v, j, tmp);
```

## Pattern: Multi-way dispatch (op codes)
```bmb
// Use chained if-else for dispatch:
let result = if op == 1 { a + b }
    else if op == 2 { a - b }
    else if op == 3 { a * b }
    else { a / b };

// For side-effect dispatch (void operations):
if op == 1 {
    let k = read_int();
    for j in 0..n {
        let _s = vec_set(v, j, vec_get(v, j) + k)
    }
} else if op == 2 {
    // ... another operation ...
    ()
} else { () };
```

## Pattern: Key-value store (linear scan)
```bmb
let keys = vec_new();
let vals = vec_new();

// Set: search for existing key, update or append
let len = vec_len(keys);
let mut found = 0;
for j in 0..len {
    if vec_get(keys, j) == key {
        let _s = vec_set(vals, j, value);
        found = 1;
        break
    } else { () }
};
if found == 0 {
    let _pk = vec_push(keys, key);
    let _pv = vec_push(vals, value);
    ()
} else { () };

// Get: search and return value or default
let mut result = -1;  // default
for j in 0..len {
    if vec_get(keys, j) == key {
        result = vec_get(vals, j);
        break
    } else { () }
};
```

## Pattern: Stack (push/pop/top)
```bmb
let stack = vec_new();
// push
let _p = vec_push(stack, value);
// top
let len = vec_len(stack);
let top = vec_get(stack, len - 1);
// pop
let _p = vec_pop(stack);
```

## Pattern: Queue (FIFO with front pointer)
```bmb
let queue = vec_new();
let mut front = 0;
// enqueue
let _p = vec_push(queue, value);
// dequeue
let val = vec_get(queue, front);
front = front + 1;
```

## Common Pitfalls
- `println()` returns `()`, not `i64` — always wrap: `let _r = println(x);`
- `vec_push()/vec_set()/vec_free()` all return `()` — always wrap with `let _`
- No `v[i]` for vec — use `vec_get(v, i)` and `vec_set(v, i, val)`
- No closures, iterators, or method calls — use free functions
- No `impl` blocks — use free functions
- Blocks must end with `;` after `}` in while/if/for
- Vec handle type is `i64`, not `Vec<T>`
- if/else used as statement MUST have `else { () }`
- Use `0 - x` instead of `-x` for negation

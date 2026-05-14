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
if x > 0 { do_something() }         // if without else — returns unit (v0.98.3)
while cond { body; }                 // while loop
for i in 0..n { body; }             // for loop with range [0, n)
break;                               // exit loop early
continue;                            // skip to next iteration
return expr;                         // early return from function

// while let — pattern-matching loop (v0.98.3, interpreter-only)
// Exits when pattern doesn't match. Requires enum-variant pattern (not bare variable).
// enum Opt { None, Some(i64) }
// fn next(n: i64) -> Opt = if n > 0 { Opt::Some(n - 1) } else { Opt::None };
// while let Opt::Some(v) = next(count) {
//     count = v;
//     sum = sum + v
// };
```

## I/O
```bmb
let _p = println(42);           // print i64 + newline
let _p = print(42);             // print i64, no newline
let _p = println_str("hello");  // print string + newline
let _p = print_str(" ");        // print string, no newline
let n = read_int();              // read i64 from stdin
let line = read_line();          // read a line from stdin (String)
```

## String Operations
```bmb
// Concatenation
let s = "hello" + " " + "world";      // String + String → String
let s2 = "num=" + int_to_string(42);  // int → String then concat

// String builtins
let len = str_len(s);              // character count
let c = char_at(s, 0);            // character at index (char type)
let b = s.byte_at(0);             // byte value at index (i64)
let n = ord(c);                   // char → i64 (ASCII code)
let ch = chr(65);                 // i64 → char ('A')
let si = int_to_string(42);       // i64 → String ("42")
let fi = i64_to_f64(n);           // i64 → f64
let ni = f64_to_i64(3.7);         // f64 → i64 (truncates toward zero)

// String search / manipulation (v0.98.1+)
let found = str_contains(s, "sub");     // 1 if contains, 0 if not
let ok = str_starts_with(s, "pre");    // 1 if starts with prefix
let ok2 = str_ends_with(s, "suf");    // 1 if ends with suffix
let idx = str_find(s, "needle");       // first byte index, or -1
let sub = str_substr(s, 6, 5);         // substring (start, len)
let trimmed = str_trim(s);             // trim leading/trailing whitespace
let n2 = str_to_int("42");             // parse integer string → i64 (0 on failure)

// Generic to_string (v0.98.2+)
let s1 = to_string(42);               // i64 → String ("42")
let s2 = to_string(3.14);             // f64 → String ("3.14")
let s3 = to_string("hello");          // String → String (identity, no extra quotes)
// Use instead of int_to_string when type is not statically known
let msg = "value=" + to_string(n);    // concatenation with any type

// String split (v0.98.3+, interpreter-only)
// fn example() -> i64 = {
//   let parts = str_split("a,b,c", ",");  // → opaque handle (i64)
//   let n = svec_len(parts);              // number of parts (3)
//   let first = svec_get(parts, 0);       // get string at index ("a")
//   let _f = svec_free(parts);            // release (use "let _f =" to discard Unit)
//   n
// };
// Note: str_split("abc", "") splits into individual characters
// Note: Use "let _f = svec_free(parts)" not "svec_free(parts);" (no standalone expr stmts)

// Positional string formatting (v0.98.3+, interpreter-only)
// format(template, arg0, arg1, ...) → String
// let s = format("{0} + {1} = {2}", to_string(a), to_string(b), to_string(a+b));
// let msg = format("name={0}, age={1}", name, to_string(age));
// Placeholders: {0}, {1}, {2}, ... replaced by args in order.
// Any type accepted; non-String args are formatted via their Display representation.
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

## HashMap (key→value store)
```bmb
let m = hashmap_new();               // create map (key=i64, value=i64)
let _i = hashmap_insert(m, k, v);   // insert or overwrite
let ok = hashmap_contains(m, k);    // 1 if exists, 0 otherwise
let val = hashmap_get(m, k);        // value or i64::MIN if not found
let n = hashmap_len(m);             // number of distinct keys
let _f = hashmap_free(m);           // deallocate
```

Usage pattern for get-with-default:
```bmb
let result = if hashmap_contains(m, key) == 1 { hashmap_get(m, key) } else { -1 };
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
// if as VALUE: MUST have else (to return a value)
let x = if a > b { a } else { b };

// if as STATEMENT: else is OPTIONAL (v0.98.1+)
if count > 0 {
    let _p = println(count);
};

// else-if chain: final else is OPTIONAL (v0.98.1+)
if x == 1 { let _p = println(10); }
else if x == 2 { let _p = println(20); }
else if x == 3 { let _p = println(30); };

// else-if chain as VALUE: still needs final else
let label = if x == 1 { "one" }
    else if x == 2 { "two" }
    else { "other" };
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
    if i > 0 { let _s = print_str(" ") };
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
// Use chained if-else for dispatch (as VALUE — needs final else):
let result = if op == 1 { a + b }
    else if op == 2 { a - b }
    else if op == 3 { a * b }
    else { a / b };

// For side-effect dispatch (void operations — final else optional):
if op == 1 {
    let k = read_int();
    for j in 0..n {
        let _s = vec_set(v, j, vec_get(v, j) + k)
    }
} else if op == 2 {
    // ... another operation ...
    ()
};
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
    }
};
if found == 0 {
    let _pk = vec_push(keys, key);
    let _pv = vec_push(vals, value)
};

// Get: search and return value or default
let mut result = -1;  // default
for j in 0..len {
    if vec_get(keys, j) == key {
        result = vec_get(vals, j);
        break
    }
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

## Pattern: Selection sort
```bmb
// Sort v[0..n] ascending in-place
for i in 0..n {
    let mut min_idx = i;
    for j in (i+1)..n {
        if vec_get(v, j) < vec_get(v, min_idx) { min_idx = j }
    };
    if min_idx != i {
        let tmp = vec_get(v, i);
        let _s1 = vec_set(v, i, vec_get(v, min_idx));
        let _s2 = vec_set(v, min_idx, tmp)
    }
};
```

## Math Builtins
```bmb
// Integer math
let a = abs(-5);          // → 5    (works for i64 and f64)
let s = sign(-3);         // → -1   (1 / 0 / -1 for i64)
let m = min(3, 7);        // → 3
let x = max(3, 7);        // → 7
let p = pow(2, 10);       // → 1024 (i64^i64 → i64)
let c = clamp(x, 0, 100); // → clamped x in [0, 100]

// Float math
let r = sqrt(2.0);        // → 1.414...
let f = floor(3.7);       // → 3.0
let ce = ceil(3.2);       // → 4.0
let l = log(2.718);       // → natural log
let e = exp(1.0);         // → e (~2.718)
```

## Pattern: Absolute value and GCD
```bmb
// Absolute value: use abs() builtin
let abs_x = abs(x);       // works for both i64 and f64
// Negation: -x syntax works
let neg_x = -x;

// GCD (Euclidean)
fn gcd(a: i64, b: i64) -> i64 =
    if b == 0 { a } else { gcd(b, a - (a / b) * b) };
```

## Pattern: 2D array (vec of rows)
```bmb
// Create n×m grid, initialized to 0
let rows = vec_new();
for _i in 0..n {
    let row = vec_new();
    for _j in 0..m {
        let _p = vec_push(row, 0)
    };
    let _r = vec_push(rows, row)
};
// Access grid[i][j]
let row_i = vec_get(rows, i);
let val = vec_get(row_i, j);
// Set grid[i][j] = v
let row_i2 = vec_get(rows, i);
let _s = vec_set(row_i2, j, v);
```

## Pattern: Read until n commands with variable args
```bmb
let n = read_int();
for _i in 0..n {
    let op = read_int();
    if op == 1 {
        let k = read_int();
        let v = read_int();
        // set operation
        ()
    } else if op == 2 {
        let k = read_int();
        // get operation — print result
        let _p = println(result)
    } else {
        // count operation — print count
        let _p = println(count)
    }
};
```

## Pattern: Find max/min in vec
```bmb
// Max value in v[0..n]
let mut best = vec_get(v, 0);
for i in 1..n {
    if vec_get(v, i) > best { best = vec_get(v, i) }
};
```

## Pattern: Prefix sum (cumulative sum array)
```bmb
// Build prefix[i] = v[0] + v[1] + ... + v[i-1], prefix[0] = 0
let prefix = vec_new();
let _p0 = vec_push(prefix, 0);
let mut acc = 0;
for i in 0..n {
    acc = acc + vec_get(v, i);
    let _p = vec_push(prefix, acc)
};
// Range sum [l, r) = prefix[r] - prefix[l]
let range_sum = vec_get(prefix, r) - vec_get(prefix, l);
```

## Pattern: BFS (shortest path on unweighted graph)
```bmb
// Assumes: adj_list stored as flat vec with offset array, or simple grid BFS
let dist = vec_new();
for _i in 0..n { let _p = vec_push(dist, -1) };
let queue = vec_new();
let _p = vec_push(queue, src);
let _d = vec_set(dist, src, 0);
let mut front = 0;
while front < vec_len(queue) {
    let u = vec_get(queue, front);
    front = front + 1;
    let nb_start = vec_get(adj_start, u);
    let nb_end = vec_get(adj_start, u + 1);
    for j in nb_start..nb_end {
        let nb = vec_get(neighbors, j);
        if vec_get(dist, nb) == -1 {
            let _d2 = vec_set(dist, nb, vec_get(dist, u) + 1);
            let _q = vec_push(queue, nb)
        }
    }
};
```

## Pattern: Iterate vec by index (no for-in-vec)
```bmb
// BMB for loop only supports ranges. To iterate a vec:
let n = vec_len(v);
for i in 0..n {
    let val = vec_get(v, i);
    // use val
    ()
};
```

## Pattern: Count occurrences with HashMap
```bmb
let counts = hashmap_new();
for i in 0..n {
    let x = vec_get(v, i);
    let prev = if hashmap_contains(counts, x) == 1 { hashmap_get(counts, x) } else { 0 };
    let _ins = hashmap_insert(counts, x, prev + 1)
};
```

## Common Pitfalls
- `println()` returns `()`, not `i64` — always wrap: `let _r = println(x);`
- `vec_push()/vec_set()/vec_free()` all return `()` — always wrap with `let _`
- No `v[i]` for vec — use `vec_get(v, i)` and `vec_set(v, i, val)`
- No closures, iterators, or method calls — use free functions
- No `impl` blocks — use free functions
- Blocks must end with `;` after `}` in while/if/for
- Vec handle type is `i64`, not `Vec<T>`
- `if` as VALUE (used in `let x = if ...`) MUST have `else`
- `if` as STATEMENT (result discarded) — `else` is optional (v0.98.1+)
- `-x` (unary minus) works for negation — `0 - x` is unnecessary
- `for j in (i+1)..n` — parentheses required when start is an expression
- `for` loop only supports ranges (`0..n`, `a..=b`) — NOT arbitrary iterators; use index loop
- `for x in my_vec` does NOT work — vec is an i64 handle, use `for i in 0..vec_len(v)`
- `hashmap_get` returns `i64::MIN` (not 0) when key is absent — always check `hashmap_contains` first
- `to_string(x)` converts any value to String without extra quotes (v0.98.2+)
- `int_to_string(n)` is i64-only; use `to_string(n)` when type may vary
- String builtins (`str_contains`, `str_find`, `str_substr`, `str_trim`, `str_to_int`, `to_string`, `str_split`, `svec_*`) work with `bmb run` only — `bmb build` (native) will fail with linker errors for these

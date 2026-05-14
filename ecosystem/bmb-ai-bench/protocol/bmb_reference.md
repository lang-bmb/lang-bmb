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
let found = str_contains(s, "sub");         // 1 if contains, 0 if not
let ok = str_starts_with(s, "pre");        // 1 if starts with prefix
let ok2 = str_ends_with(s, "suf");         // 1 if ends with suffix
let idx = str_find(s, "needle");            // first byte index, or -1
let sub = str_substr(s, 6, 5);             // substring (start, len)
let trimmed = str_trim(s);                 // trim leading/trailing whitespace
let n2 = str_to_int("42");                 // parse integer string → i64 (0 on failure)
let replaced = str_replace(s, "x", "y");  // replace all "x" with "y" (v0.98.3, interp-only)
let rep = str_repeat("ab", 3);             // "ababab" — repeat n times (v0.98.3, interp-only)

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
//   let j = svec_join(parts, "-");       // join all parts with delimiter (v0.98.3, interp-only)
// Build svec manually (v0.98.5+, interpreter-only):
//   let sv = svec_new();                 // create empty svec
//   let _p = svec_push(sv, "item");      // append string
//   let _f = svec_free(sv);              // release

// Positional string formatting (v0.98.3+, interpreter-only)
// format(template, arg0, arg1, ...) → String
// let s = format("{0} + {1} = {2}", to_string(a), to_string(b), to_string(a+b));
// let msg = format("name={0}, age={1}", name, to_string(age));

// String interpolation (v0.98.4+, interpreter-only)
// "Hello {name}" → automatically desugars to format("Hello {0}", name)
// Supports: idents, arithmetic ({n+1}), field access ({p.x}), unary ({-n}), parens
// Numeric {0} kept as-is (format-arg style). Function calls not supported inside {}.
// let greeting = "Hello {name}";    // → format("Hello {0}", name)
// let info = "n+1 = {n + 1}";       // → format("n+1 = {0}", n + 1)
// let fp = "x={p.x}";              // → format("x={0}", p.x)
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
let _c = vec_clear(v);          // set length to 0 (keep capacity)
let _f = vec_free(v);           // deallocate

// Aggregate operations (interpreter-only, v0.98.3):
let s = vec_sum(v);             // sum all elements (i64)
let mx = vec_max(v);            // maximum element (error on empty)
let mn = vec_min(v);            // minimum element (error on empty)
let _o = vec_sort(v);           // sort ascending in-place
let ok = vec_contains(v, 42);  // 1 if 42 found, 0 otherwise
let i = vec_index_of(v, 42);   // first index of 42, or -1 if not found
```

## HashMap (i64 key→value store)
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

## String HashMap (String key→i64 store, interpreter-only v0.98.5+)
```bmb
let m = str_hashmap_new();                         // create map (key=String, value=i64)
let _i = str_hashmap_insert(m, "word", 42);        // insert or overwrite
let ok = str_hashmap_contains(m, "word");          // 1 if exists, 0 otherwise
let val = str_hashmap_get(m, "word");              // value or i64::MIN if not found
let n = str_hashmap_len(m);                        // number of distinct keys
let keys = str_hashmap_keys(m);                    // svec handle of all keys (unordered)
let skeys = str_hashmap_sorted_keys(m);            // svec handle of keys (sorted a-z, v0.98.5+)
let _i = str_hashmap_inc(m, "word", 1);            // increment value by 1 (insert 0 if absent, v0.98.5+)
let _f = str_hashmap_free(m);                      // deallocate
// After using keys/skeys: svec_free(keys) / svec_free(skeys)
```

Usage pattern for get-with-default (string keys):
```bmb
let result = if str_hashmap_contains(m, key) == 1 { str_hashmap_get(m, key) } else { 0 };
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

## Pattern: Iterate vec directly (for-in-vec, v0.98.4+, interpreter-only)
```bmb
// Direct vec iteration — elem type is i64 (same as vec element type)
for x in v {
    // use x (i64 element value)
    ()
};
```

## Pattern: Iterate vec by index (works everywhere)
```bmb
// Use when you need the index, or for bmb build (native):
let n = vec_len(v);
for i in 0..n {
    let val = vec_get(v, i);
    // use val and i
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

## Pattern: String word frequency (str_hashmap, interpreter-only v0.98.5+)
```bmb
// Count occurrences of string keys (e.g., word → count)
fn count_word(counts: i64, word: String) -> i64 = {
    let prev = if str_hashmap_contains(counts, word) == 1 {
        str_hashmap_get(counts, word)
    } else { 0 };
    let _i = str_hashmap_insert(counts, word, prev + 1);
    0
};
// fn main() -> i64 = {
//     let counts = str_hashmap_new();
//     let _c1 = count_word(counts, "apple");
//     let _c2 = count_word(counts, "banana");
//     let _c3 = count_word(counts, "apple");
//     let apple_count = str_hashmap_get(counts, "apple");  // → 2
//     let _f = str_hashmap_free(counts);
//     apple_count
// };
```

## Pattern: Iterate str_hashmap keys (v0.98.5+, interpreter-only)
```bmb
// Iterate over all key-value pairs using str_hashmap_sorted_keys
fn main() -> i64 = {
    let m = str_hashmap_new();
    let _a = str_hashmap_insert(m, "c", 30);
    let _b = str_hashmap_insert(m, "a", 10);
    let _c = str_hashmap_insert(m, "b", 20);

    let keys = str_hashmap_sorted_keys(m);   // sorted: ["a","b","c"]
    let n = svec_len(keys);
    let sum = 0;
    for i in 0..n {
        let key = svec_get(keys, i);
        let val = str_hashmap_get(m, key);
        sum += val
    };
    let _fk = svec_free(keys);
    let _fm = str_hashmap_free(m);
    sum  // 60
};
// str_hashmap_keys returns unordered keys; str_hashmap_sorted_keys returns alphabetical order
// Use svec_len + svec_get loop (for-in-svec not yet supported)
```

## Pattern: Binary search
```bmb
// Find leftmost index where v[i] >= target in sorted v[0..n]. Returns n if all < target.
let mut lo = 0;
let mut hi = n;
while lo < hi {
    let mid = lo + (hi - lo) / 2;
    if vec_get(v, mid) < target { lo = mid + 1 } else { hi = mid }
};
// lo is the answer. Check lo < n && vec_get(v, lo) == target for exact match.
```

## Pattern: DFS (depth-first search)
```bmb
// Iterative DFS using a stack (avoids deep recursion)
let visited = vec_new();
for _i in 0..n { let _p = vec_push(visited, 0) };
let stk = vec_new();
let _p = vec_push(stk, start);
while vec_len(stk) > 0 {
    let top = vec_get(stk, vec_len(stk) - 1);
    let _pop = vec_pop(stk);
    if vec_get(visited, top) == 0 {
        let _v = vec_set(visited, top, 1);
        // Process node `top` here
        let nb_start = vec_get(adj_start, top);
        let nb_end = vec_get(adj_start, top + 1);
        for j in nb_start..nb_end {
            let nb = vec_get(neighbors, j);
            if vec_get(visited, nb) == 0 { let _p = vec_push(stk, nb) }
        }
    }
};
```

## Pattern: String accumulation (build result string)
```bmb
// Build "1,2,3,4,5" from a vec
let mut result = "";
for i in 0..n {
    if i > 0 { result = result + "," };
    result = result + to_string(vec_get(v, i))
};
let _p = println_str(result);
```

## Pattern: while let with enum (safe iteration)
```bmb
// Iterate over an enum-based optional sequence
enum Opt { None, Some(i64) }
fn next_val(n: i64) -> Opt = if n > 0 { Opt::Some(n - 1) } else { Opt::None };
// fn main() -> i64 = {
//     let mut cursor = 5;
//     let mut sum = 0;
//     while let Opt::Some(v) = next_val(cursor) {
//         cursor = v;
//         sum = sum + v
//     };
//     sum   // = 0+1+2+3+4 = 10
// };
```

## Pattern: Number to string conversions
```bmb
let si = to_string(42);        // i64 → "42"
let sf = to_string(3.14);      // f64 → "3.14"
let sb = to_string(true);      // bool → "true"
// Positional format string (interpreter-only):
let msg = format("{0}+{1}={2}", to_string(a), to_string(b), to_string(a+b));
// Output "3+4=7" for a=3, b=4
```

## Pattern: Memoization (DP with HashMap cache)
```bmb
// Fibonacci with memoization — O(n) vs O(2^n) naive recursion
fn fib_memo(n: i64, memo: i64) -> i64 = {
    if hashmap_contains(memo, n) == 1 { hashmap_get(memo, n) }
    else {
        let result = if n <= 1 { n }
                     else { fib_memo(n - 1, memo) + fib_memo(n - 2, memo) };
        let _i = hashmap_insert(memo, n, result);
        result
    }
};
fn main() -> i64 = {
    let memo = hashmap_new();
    let r = fib_memo(10, memo);
    let _f = hashmap_free(memo);
    r     // 55
};
```

## Pattern: Two-pointer technique
```bmb
// Find pair that sums to target in sorted array
fn find_pair(v: i64, target: i64) -> i64 = {
    let lo = 0;
    let hi = vec_len(v) - 1;
    let found = 0;
    while lo < hi {
        let s = vec_get(v, lo) + vec_get(v, hi);
        if s == target { found = 1; lo = hi }
        else { if s < target { lo = lo + 1 } else { hi = hi - 1 } }
    };
    found
};
```

## Pattern: Kadane's algorithm (maximum subarray sum)
```bmb
fn max_subarray(v: i64) -> i64 = {
    let n = vec_len(v);
    let best = vec_get(v, 0);
    let cur = best;
    let i = 1;
    while i < n {
        let x = vec_get(v, i);
        cur = if cur + x > x { cur + x } else { x };
        best = if cur > best { cur } else { best };
        i = i + 1
    };
    best
};
```

## Pattern: String processing pipeline (split + transform + join)
```bmb
// Replace all commas with semicolons in CSV-like string (interpreter-only)
fn csv_to_ssv(s: String) -> String = {
    let parts = str_split(s, ",");
    let n = svec_len(parts);
    let result = svec_join(parts, ";");
    let _f = svec_free(parts);
    result
};
// Or use str_replace directly:
fn csv_to_ssv_v2(s: String) -> String = str_replace(s, ",", ";");
```

## Pattern: Frequency count on string characters
```bmb
// Count occurrences of each ASCII character in a string
fn char_freq(s: String, freq: i64) -> i64 = {
    // freq is a vec of size 128 (initialized to 0)
    let i = 0;
    let n = str_len(s);
    while i < n {
        let b = s.byte_at(i);
        let c = vec_get(freq, b);
        let _s = vec_set(freq, b, c + 1);
        i = i + 1
    };
    0
};
// Query: how many times does 'a' appear?
// let count = vec_get(freq, ord('a'));
```

## Pattern: Vec iteration and transformation (interpreter-only)
```bmb
// Sum all elements in a vec
fn vec_total(v: i64) -> i64 = {
    let s = 0;
    for x in v {
        s = s + x
    };
    s
};

// Filter and collect elements > threshold into new vec
fn filter_gt(v: i64, thresh: i64) -> i64 = {
    let out = vec_new();
    for x in v {
        if x > thresh { let _p = vec_push(out, x); }
    };
    out
};
```

## Pattern: String expression interpolation (v0.98.5+, interpreter-only)
```bmb
// {expr} inside strings — supports arithmetic, field access, unary minus, parens
fn main() -> i64 = {
    let n = 5;
    let s1 = "n+1 = {n + 1}";       // → "n+1 = 6"
    let s2 = "{n * 2} doubled";      // → "10 doubled"

    // Field access in interpolation
    // struct Pt { x: i64, y: i64 }
    // let p: Pt = new Pt { x: 3, y: 7 };
    // let s3 = "x={p.x}, y={p.y}";  // → "x=3, y=7"

    // Mixed: ident + expr
    let a = 4; let b = 6;
    let s4 = "{a} + {b} = {a + b}";  // → "4 + 6 = 10"

    str_len(s4)  // 10
};
// Supported inside {}: arithmetic (+,-,*,/,%), field chains (a.b.c), unary minus, parens
// NOT supported: function calls ({to_string(n)} won't work — use let binding instead)
// {{  }} remain literal brace escapes
```

## Pattern: String interpolation (interpreter-only)
```bmb
// Simple variable interpolation in string literals
fn greet(name: String, age: i64) -> String = {
    let s = "Hello {name}, you are {age} years old";
    // Equivalent to: format("Hello {0}, you are {1} years old", name, age)
    s
};

// Combine with other string ops
fn build_label(key: String, val: i64) -> String = {
    let v = to_string(val);
    "{key}: {v}"   // → format("{0}: {1}", key, v)
};
```

## Pattern: Compound assignment operators (v0.98.4+)
```bmb
// +=, -=, *=, /= desugar to x = x op expr at parse time
fn accumulate(n: i64) -> i64 = {
    let sum = 0;
    for i in 0..n { sum += i; };
    sum
};

fn scale_down(v: i64, factor: i64) -> i64 = {
    let r = v;
    r /= factor;
    r
};
// Note: works everywhere set/assignment works (interpreter + native)
```

## Pattern: Field compound assignment (v0.98.5+, interpreter-only)
```bmb
// set obj.field += e  desugars to  set obj.field = obj.field op e
struct Stats { total: i64, count: i64 }

fn add_sample(s: Stats, v: i64) -> i64 = {
    set s.total += v;
    set s.count += 1;
    0  // returns unit sentinel
};

fn main() -> i64 = {
    let mut st: Stats = new Stats { total: 0, count: 0 };
    let _a = add_sample(st, 10);  // note: BMB passes structs by copy; mutation in callee won't propagate
    // Mutate directly:
    set st.total += 100;
    set st.total += 200;
    set st.count += 1;
    set st.count += 1;
    st.total + st.count  // 302
};
// Supports all 5 operators: +=  -=  *=  /=  %=
// Interpreter-only (bmb run); bmb build (native) unsupported in v0.98.5
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
- `for` loop supports ranges (`0..n`, `a..=b`) AND vec handles (`for x in v {}` — interpreter-only, v0.98.4+)
- `for x in my_vec` works with interpreter only — for bmb build (native) use index loop `for i in 0..vec_len(v)`
- `hashmap_get` and `str_hashmap_get` return `i64::MIN` (not 0) when key is absent — always check `*_contains` first
- `str_hashmap_*` builtins use String keys; interpreter-only (`bmb run`) — `bmb build` (native) unsupported (v0.98.5+)
- `to_string(x)` converts any value to String without extra quotes (v0.98.2+)
- `int_to_string(n)` is i64-only; use `to_string(n)` when type may vary
- `while let` only supports enum-variant patterns (e.g., `Opt::Some(x)`) — bare `while let x = e` not supported (would infinite-loop anyway)
- `format()`, `while let`, `for x in vec` and string interpolation `"Hello {name}"` are interpreter-only (`bmb run`) — `bmb build` (native) doesn't support them yet
- In string interpolation, `{{` → literal `{` and `}}` → literal `}` (v0.98.5+). Example: `"{{key}}: {val}"` → `"{key}: <value>"`
- String interpolation `{expr}` supports arithmetic/field access but NOT function calls — use `let tmp = fn(x); "{tmp}"` instead (v0.98.5+)
- `+=`, `-=`, `*=`, `/=`, `%=` compound assignment operators available (v0.98.4+) — desugars to `x = x op e`; also available on struct fields: `set obj.field += e` (v0.98.5+, interpreter-only)
- String builtins (`str_contains`, `str_find`, `str_substr`, `str_trim`, `str_to_int`, `to_string`, `str_split`, `svec_*`, `str_replace`, `str_repeat`, `format`) work with `bmb run` only — `bmb build` (native) will fail for these
- Vec aggregate/search builtins (`vec_sum`, `vec_max`, `vec_min`, `vec_sort`, `vec_contains`, `vec_index_of`) are interpreter-only (`bmb run`) — `bmb build` (native) unsupported

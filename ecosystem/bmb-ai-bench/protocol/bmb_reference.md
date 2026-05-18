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

// while let — pattern-matching loop (v0.98.3+, native v0.98.9+)
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
let tl = str_trim_left(s);                // trim leading whitespace only (v0.98.7+)
let tr = str_trim_right(s);               // trim trailing whitespace only (v0.98.7+)
let n2 = str_to_int("42");                 // parse integer string → i64 (0 on failure)
let h = int_to_hex(255);                   // → "ff"  (lowercase hex, v0.98.7+)
let b = int_to_bin(10);                    // → "1010" (binary, v0.98.7+)
let replaced = str_replace(s, "x", "y");  // replace all "x" with "y" (v0.98.3+)
let rep = str_repeat("ab", 3);             // "ababab" — repeat n times (v0.98.3+)
let cnt = str_count("aababc", "ab");       // 2 — count substring occurrences (v0.98.6+)
let pl = str_pad_left("42", 5, "0");       // "00042" — left-pad to width (v0.98.6+)
let pr = str_pad_right("hi", 5, " ");      // "hi   " — right-pad to width (v0.98.6+)
let up = str_to_upper("hello");           // "HELLO" — Unicode uppercase (v0.98.6+)
let lo = str_to_lower("WORLD");           // "world" — Unicode lowercase (v0.98.6+)
let ch = str_char_at("hello", 1);         // "e" — single-char String at index (v0.98.6+, native v0.98.9+)
let rv = str_reverse("hello");            // "olleh" — reverse string (v0.98.7+)
let fv = str_to_f64("3.14");              // parse float string → f64 (0.0 on failure, v0.98.7+)
let fv2 = read_f64();                     // read f64 from stdin (v0.98.7+)
let empty = str_is_empty(s);             // 1 if s == "", 0 otherwise (v0.98.9+)
// str_lines(s) → SvecHandle — split by '\n', strip '\r' (v0.98.7+, native v0.98.9+)
// let lines = str_lines("a\nb\nc");  // 3 elements; use svec_* to iterate/access

// Generic to_string (v0.98.2+)
let s1 = to_string(42);               // i64 → String ("42")
let s2 = to_string(3.14);             // f64 → String ("3.14")
let s3 = to_string("hello");          // String → String (identity, no extra quotes)
// Use instead of int_to_string when type is not statically known
let msg = "value=" + to_string(n);    // concatenation with any type

// String split (v0.98.3+, native v0.98.9+)
// fn example() -> i64 = {
//   let parts = str_split("a,b,c", ",");  // → SvecHandle (opaque)
//   let n = svec_len(parts);              // number of parts (3)
//   let first = svec_get(parts, 0);       // get string at index ("a")
//   let _f = svec_free(parts);            // release (use "let _f =" to discard Unit)
//   n
// };
// str_split_whitespace(s) → SvecHandle — split by whitespace, skip empty tokens (v0.98.7+, native v0.98.9+)
// let tokens = str_split_whitespace("  1 2   3  ");  // → ["1","2","3"] (3 elements)
// Note: str_split("abc", "") splits into individual characters
// Note: Use "let _f = svec_free(parts)" not "svec_free(parts);" (no standalone expr stmts)
//   let j = svec_join(parts, "-");       // join all parts with delimiter (v0.98.3, native v0.98.9+)
// Build svec manually (v0.98.5+, native v0.98.9+/v0.98.10+):
//   let sv = svec_new();                 // create empty svec
//   let _p = svec_push(sv, "item");      // append string
//   let _so = svec_sort(sv);             // sort lexicographically in-place (v0.98.6+, native v0.98.10+)
//   let ok = svec_contains(sv, "item");  // 1 if found, 0 otherwise (v0.98.6+)
//   let i = svec_index_of(sv, "item");  // first index or -1 (v0.98.7+)
//   let _rm = svec_remove(sv, 0);        // remove element at index (v0.98.6+, native v0.98.10+)
//   let _cl = svec_clear(sv);            // remove all elements (v0.98.6+, native v0.98.10+)
//   let _f = svec_free(sv);              // release

// Positional string formatting (v0.98.3+, native v0.98.9+)
// format(template, arg0, arg1, ...) → String
// let s = format("{0} + {1} = {2}", to_string(a), to_string(b), to_string(a+b));
// let msg = format("name={0}, age={1}", name, to_string(age));

// String interpolation (v0.98.4+, native v0.98.9+)
// "Hello {name}" → automatically desugars to format("Hello {0}", name)
// Supports: idents, arithmetic ({n+1}), field access ({p.x}), unary ({-n}), parens, function calls ({fn(args)} v0.98.6+)
// Numeric {0} kept as-is (format-arg style).
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

// Aggregate operations (v0.98.3+):
let s = vec_sum(v);             // sum all elements (i64)
let mx = vec_max(v);            // maximum element (error on empty)
let mn = vec_min(v);            // minimum element (error on empty)
let _o = vec_sort(v);           // sort ascending in-place
let ok = vec_contains(v, 42);  // 1 if 42 found, 0 otherwise
let i = vec_index_of(v, 42);   // first index of 42, or -1 if not found
let rm = vec_remove(v, 2);     // remove at index 2, shift left, return removed value (v0.98.6+)
let _rv = vec_reverse(v);      // reverse elements in-place (v0.98.6+)
let _fi = vec_fill(v, 0);      // set all elements to 0 (v0.98.6+)
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

## String HashMap (String key→i64 store, native-supported v0.98.9+/v0.98.10+)
```bmb
let m = str_hashmap_new();                         // create map (key=String, value=i64)
let _i = str_hashmap_insert(m, "word", 42);        // insert or overwrite
let ok = str_hashmap_contains(m, "word");          // 1 if exists, 0 otherwise
let val = str_hashmap_get(m, "word");              // value or i64::MIN if not found
let n = str_hashmap_len(m);                        // number of distinct keys
let keys = str_hashmap_keys(m);                    // svec handle of all keys (unordered)
let skeys = str_hashmap_sorted_keys(m);            // svec handle of keys (sorted a-z, v0.98.5+)
let _i = str_hashmap_inc(m, "word", 1);            // increment value by 1 (insert 0 if absent, v0.98.5+)
let _d = str_hashmap_delete(m, "key");             // remove key (no-op if absent, v0.98.6+)
let _u = str_hashmap_update(m, "key", 99);         // overwrite value (v0.98.6+)
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
// CRITICAL: vec_pop returns () (unit), NOT the removed value.
// Always vec_get the value BEFORE calling vec_pop.
let stack = vec_new();
// push
let _p = vec_push(stack, value);
// read top (get before pop)
let len = vec_len(stack);
let top = vec_get(stack, len - 1);   // read
let _p = vec_pop(stack);              // discard (returns ())
// pop two and compute (stack pattern):
let b = vec_get(stack, vec_len(stack) - 1); let _pb = vec_pop(stack);
let a = vec_get(stack, vec_len(stack) - 1); let _pa = vec_pop(stack);
let _r = vec_push(stack, a + b);
```

## Pattern: Queue (FIFO with front pointer)
```bmb
// CRITICAL: vec_pop removes from the BACK. For FIFO dequeue, use front index + size, NOT vec_pop.
let queue = vec_new();
let mut front = 0;
let mut size = 0;
// enqueue
let _p = vec_push(queue, value);
size = size + 1;
// dequeue (check size > 0 first)
let val = vec_get(queue, front);
front = front + 1;
size = size - 1;
// check if empty
// if size == 0 { /* empty */ }
```

## Pattern: Bounded queue (FIFO, capacity limit)
```bmb
// Queue with max capacity. Enqueue to full = overflow (-1). Uses front+size.
let q = vec_new();
let mut front = 0;
let mut size = 0;
let cap = read_int();
// enqueue val (if not full)
if size < cap {
    let _p = vec_push(q, val);
    size = size + 1
} else {
    let _p = println(-1)  // overflow
};
// dequeue (if not empty)
if size > 0 {
    let val = vec_get(q, front);
    let _p = println(val);
    front = front + 1;
    size = size - 1
} else {
    let _p = println(-1)  // underflow
};
// size query: println(size)
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
let a = abs(-5);               // → 5    (works for i64 and f64)
let s = sign(-3);              // → -1   (method: n.sign()) (1 / 0 / -1 for i64)
let m = min(3, 7);             // → 3    (free function, i64 only — use min_f64 for f64)
let x = max(3, 7);             // → 7    (free function, i64 only — use max_f64 for f64)
let p = pow_i64(2, 10);        // → 1024 (i64^i64 → i64, v0.98.6+)
let c = clamp_i64(x, 0, 100); // → clamped x in [0, 100] (v0.98.6+)
let g = gcd_i64(48, 18);       // → 6   (Euclidean GCD, v0.98.6+)
let pc = popcount(7);           // → 3   (count set bits, 7 = 0b111, v0.98.7+)
// Float min/max/clamp (v0.98.7+)
let mf = min_f64(3.14, 2.72);          // → 2.72
let xf = max_f64(3.14, 2.72);          // → 3.14
let cf = clamp_f64(v, 0.0, 1.0);       // → clamped v in [0.0, 1.0]

// Bitwise operators (infix keywords, i64 only)
// NOTE: BMB uses keywords, NOT &/|/^ symbols
let ba = a band b;     // bitwise AND
let bo = a bor b;      // bitwise OR
let bx = a bxor b;     // bitwise XOR
let sl = a << 3;       // left shift by 3
let sr = a >> 1;       // right shift by 1 (arithmetic)
// n.bit_not() → bitwise NOT (method only)
// n.bit_count() / popcount(n) → count set bits (v0.98.7+)

// Float math
let r = sqrt(2.0);         // → 1.414...  (f64 → f64)
let f = floor(3.7);        // → 3.0
let ce = ceil(3.2);        // → 4.0
let rn = round(3.7);       // → 4.0  (v0.98.7+)
let ab = fabs(-3.5);       // → 3.5  (float absolute value)
let l = log(2.718282);     // → ~1.0  (natural log, v0.98.7+)
let l2 = log2(8.0);        // → 3.0  (base-2 log, v0.98.7+)
let l10 = log10(100.0);    // → 2.0  (base-10 log, v0.98.7+)
let e = exp(1.0);          // → ~2.718  (e^x, v0.98.7+)
let s = sin(0.0);          // → 0.0
let c = cos(0.0);          // → 1.0
let t = tan(0.0);          // → 0.0  (v0.98.7+)
let a = atan(1.0);         // → ~0.785 (π/4, v0.98.7+)
let a2 = atan2(1.0, 1.0);  // → ~0.785 (2-arg atan, v0.98.7+)
let p = pow_f64(2.0, 10.0); // → 1024.0
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

## Pattern: Iterate vec directly (for-in-vec, v0.98.4+, native-supported v0.98.9+)
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

## Pattern: String word frequency (str_hashmap, native-supported v0.98.9+)
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

## Pattern: Iterate str_hashmap keys (v0.98.5+, native-supported v0.98.9+)
```bmb
// Iterate over all key-value pairs using str_hashmap_sorted_keys
fn main() -> i64 = {
    let m = str_hashmap_new();
    let _a = str_hashmap_insert(m, "c", 30);
    let _b = str_hashmap_insert(m, "a", 10);
    let _c = str_hashmap_insert(m, "b", 20);

    let keys = str_hashmap_sorted_keys(m);   // sorted: ["a","b","c"]
    let sum = 0;
    // v0.98.7+: for-in-svec directly iterates String elements
    for key in keys {
        let val = str_hashmap_get(m, key);
        sum += val
    };
    let _fk = svec_free(keys);
    let _fm = str_hashmap_free(m);
    sum  // 60
};
// str_hashmap_keys returns unordered keys; str_hashmap_sorted_keys returns alphabetical order
// for-in-svec (v0.98.7+): iterates String elements directly; native-supported v0.98.9+
```

## Pattern: Integer-keyed registry/map using str_hashmap (native-supported v0.98.9+)
```bmb
// BMB has no dedicated int→int map. Use str_hashmap with to_string() for int keys.
fn main() -> i64 = {
    let m = str_hashmap_new();

    // set key=42 to value=100
    let key = to_string(42);
    let _s = str_hashmap_insert(m, key, 100);

    // get value for key=42 (-1 if not found)
    let lookup_key = to_string(42);
    let val = if str_hashmap_contains(m, lookup_key) == 1 {
        str_hashmap_get(m, lookup_key)
    } else { -1 };

    // overwrite: str_hashmap_insert on existing key replaces value
    let _upd = str_hashmap_insert(m, to_string(42), 200);

    // count distinct keys
    let count = str_hashmap_len(m);

    let _fm = str_hashmap_free(m);
    count  // 1
};
// Key rule: always convert integer keys to strings via to_string() before insert/get/contains
// This pattern is the standard BMB approach for int-keyed maps/registries/memoization tables
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
// Positional format string (native-supported v0.98.9+):
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
// Replace all commas with semicolons in CSV-like string (native-supported v0.98.9+)
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

## Pattern: Vec iteration and transformation (native-supported v0.98.9+)
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

## Pattern: String expression interpolation (v0.98.5+, native-supported v0.98.9+)
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
// Supported inside {}: arithmetic (+,-,*,/,%), field chains (a.b.c), unary minus, parens, function calls (v0.98.6+)
// Example with fn call: "result: {to_string(n)}", "upper: {str_to_upper(s)}"
// {{  }} remain literal brace escapes
```

## Pattern: String interpolation (native-supported v0.98.9+)
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

## Pattern: Field compound assignment (v0.98.5+, native-supported v0.98.9+)
```bmb
// set obj.field += e  desugars to  set obj.field = obj.field op e
struct Stats { total: i64, count: i64 }

fn main() -> i64 = {
    let st = new Stats { total: 0, count: 0 };
    set st.total += 100;
    set st.total += 200;
    set st.count += 1;
    set st.count += 1;
    st.total + st.count  // 302
};
// Supports all 5 operators: +=  -=  *=  /=  %=
// Works in both bmb run (interpreter) and bmb build (native) for local variables (v0.98.9+)
// Note: struct parameters passed to functions behave as references in native mode —
//   mutations inside callee WILL propagate back (unlike interpreter pass-by-value)
```

## Pattern: Palindrome check (v0.98.7+, native-supported v0.98.9+)
```bmb
fn is_palindrome(s: String) -> i64 =
    if s == str_reverse(s) { 1 } else { 0 };
```

## Pattern: Whitespace-tokenized input (v0.98.7+, native-supported v0.98.9+)
```bmb
// Parse "3 1 4 1 5" from a single line into a vec of i64
fn parse_ints(line: String) -> i64 = {
    let tokens = str_split_whitespace(line);
    let n = svec_len(tokens);
    let v = vec_new();
    let i = 0;
    while i < n {
        let t = svec_get(tokens, i);
        let _p = vec_push(v, str_to_int(t));
        i += 1
    };
    let _f = svec_free(tokens);
    v  // returns i64 vec handle
};
// Usage:
// let v = parse_ints(read_line());
// let first = vec_get(v, 0);
```

## Pattern: Float parsing and line-by-line text (v0.98.7+, native-supported v0.98.9+)
```bmb
// Parse float from string (str_to_f64)
fn parse_two_floats(line: String) -> f64 = {
    // e.g. line = "3.14 2.72"
    let parts = str_split(line, " ");
    let a = str_to_f64(svec_get(parts, 0));
    let b = str_to_f64(svec_get(parts, 1));
    let _f = svec_free(parts);
    a + b
};

// Process multi-line text with str_lines
fn count_nonempty_lines(text: String) -> i64 = {
    let lines = str_lines(text);
    let n = 0;
    for line in lines {
        if str_len(line) > 0 { n += 1; }
    };
    let _f = svec_free(lines);
    n
};

// Read multiple float inputs from stdin
fn sum_n_floats(count: i64) -> f64 = {
    let total = 0.0;
    let i = 0;
    while i < count {
        let v = read_f64();
        total = total + v;
        i += 1
    };
    total
};
// Note: str_lines strips \r\n (handles Windows/Unix line endings)
// Note: str_to_f64 returns 0.0 on parse failure
```

## Reserved stdlib function names — DO NOT redefine
The following names are defined in BMB's standard prelude and will cause `invalid redefinition` errors if you define them yourself. Use a different name (e.g., `clamp_val` instead of `clamp`):
- `clamp(x, lo, hi)` — clamped x in [lo,hi] (use `clamp_val` or `clamp_i64` as your function name)
- `min(a, b)`, `max(a, b)` — integer min/max (use `min_val` / `max_val` as your function name)
- `abs(x)` — absolute value
- `gcd_i64(a, b)` — GCD

## Common Pitfalls
- `println()` returns `()`, not `i64` — always wrap: `let _r = println(x);`
- `vec_push()/vec_set()/vec_free()` all return `()` — always wrap with `let _`
- No `v[i]` for vec — use `vec_get(v, i)` and `vec_set(v, i, val)`
- No closures, iterators, or method calls — use free functions
- No `impl` blocks — use free functions
- Blocks must end with `;` after `}` in while/if/for — including early-return `if` blocks: `if n < 2 { return 0 };` (not `if n < 2 { return 0 }`)
- `vec_pop(v)` returns `()`, NOT the removed value — always `vec_get` the value before calling `vec_pop`
- Vec handle type is `i64`, not `Vec<T>`
- Bitwise operators use **keywords** `band`/`bor`/`bxor`, NOT `&`/`|`/`^` symbols (those are parse errors)
- `str_reverse(s)` reverses the string (v0.98.7+); `popcount(n)` counts set bits (v0.98.7+)
- `if` as VALUE (used in `let x = if ...`) MUST have `else`
- `if` as STATEMENT (result discarded) — `else` is optional (v0.98.1+)
- `-x` (unary minus) works for negation — `0 - x` is unnecessary
- `for j in (i+1)..n` — parentheses required when start is an expression
- `for` loop supports ranges (`0..n`, `a..=b`), vec handles (`for x in v {}` — native-supported v0.98.9+), and svec handles (`for s in sv {}` — native-supported v0.98.9+)
- `for x in my_vec` is **native-supported** (v0.98.9+) when `my_vec` was assigned from `vec_new()` or `vec_with_capacity()` — generates index loop internally. Fallback: manual `for i in 0..vec_len(v)` always works.
- `for s in svec_var` iterates String elements; **native-supported** (v0.98.9+) when `svec_var` from `svec_new()`, `str_split()`, `str_split_whitespace()`, `str_lines()`, `str_hashmap_keys()`, or `str_hashmap_sorted_keys()` — generates index loop internally
- `hashmap_get` and `str_hashmap_get` return `i64::MIN` (not 0) when key is absent — always check `*_contains` first
- `str_hashmap_new/insert/get/contains/len/delete/free/inc/update` are **native-supported** (v0.98.9+/v0.98.10+); `str_hashmap_keys/sorted_keys` are **native-supported** (v0.98.9+); `str_hashmap_values` is **native-supported** (v0.98.10+, Cycle 2894)
- `to_string(x)` converts any value to String without extra quotes (v0.98.2+); **native-supported** (v0.98.9+) for all arg types: `i64`, `f64`, `String`, `bool`
- `int_to_string(n)` is i64-only; use `to_string(n)` when type may vary
- `while let` only supports enum-variant patterns (e.g., `Opt::Some(x)`) — bare `while let x = e` not supported (would infinite-loop anyway)
- `format()` and string interpolation `"Hello {name}"` are **native-supported** (v0.98.9+, Cycle 2890) — lowered to string concat chain at compile time (template must be a literal)
- `while let` with enum-variant patterns now works in native (`bmb build`) since v0.98.9+
- In string interpolation, `{{` → literal `{` and `}}` → literal `}` (v0.98.5+). Example: `"{{key}}: {val}"` → `"{key}: <value>"`
- String interpolation `{expr}` supports arithmetic, field access, and function calls `{fn(args)}` (v0.98.6+). Example: `"result: {to_string(n)}"`, `"upper: {str_to_upper(s)}"`
- `str_split(s, delim)`, `str_split_whitespace(s)`, `str_lines(s)` are **native-supported** (v0.98.9+, Cycle 2887) — return svec handles
- `+=`, `-=`, `*=`, `/=`, `%=` compound assignment operators available (v0.98.4+) — desugars to `x = x op e`; also available on struct fields: `set obj.field += e` (v0.98.5+, native-supported v0.98.9+ for local struct vars)
- String builtins with **native support** (both `bmb run` and `bmb build`): `str_len`, `str_is_empty`, `str_contains`, `str_starts_with`, `str_ends_with`, `str_find`, `str_trim`, `str_trim_left`, `str_trim_right`, `str_to_int`, `str_to_f64`, `str_substr`, `str_count`, `str_pad_left`, `str_pad_right`, `str_replace`, `str_repeat`, `str_to_upper`, `str_to_lower`, `str_reverse`, `str_char_at`, `int_to_hex`, `int_to_bin`, `str_split`, `str_split_whitespace`, `str_lines` (v0.98.9+)
- All string/vec/svec/hashmap/math builtins are now **native-supported** (`bmb build` works) as of v0.98.10+ (Cycles 2871-2894). No interpreter-only builtins remain.
- `svec_sort`, `svec_remove`, `svec_clear` are **native-supported** (v0.98.10+); all other `svec_*` already native since v0.98.9+
- Vec aggregate/search/mutation builtins with **native support** (both `bmb run` and `bmb build`): `vec_sum`, `vec_max`, `vec_min`, `vec_sort`, `vec_contains`, `vec_index_of`, `vec_remove`, `vec_reverse`, `vec_fill` (v0.98.9+)
- **Native struct parameter behavior**: In `bmb build` (native), struct arguments are passed by pointer — mutations inside a callee function DO affect the caller's copy (unlike interpreter which uses pass-by-value). Avoid mutating struct params inside callees if interpreter/native parity is needed.

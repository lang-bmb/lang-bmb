# Performance Optimization in BMB

> Achieving C-level performance with zero-cost contracts

## Performance Results

BMB achieves competitive performance with C, often surpassing it:

```
Benchmark         C        Rust      BMB       Winner
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
fibonacci(45)     1.65s    1.66s     1.63s     ★ BMB (0.99x)
fibonacci(40)     177ms    180ms     150ms     ★ BMB (0.85x)
mandelbrot        42ms     42ms      39ms      ★ BMB (0.93x)
spectral_norm     44ms     44ms      39ms      ★ BMB (0.89x)
self-compile      -        -         0.56s     ✅ < 60s target
```

## Why BMB is Fast

### 1. Zero-Cost Contracts

Contracts are verified at compile time and have no runtime overhead:

```bmb
fn divide(a: i64, b: i64) -> i64
  pre b != 0  // Verified at compile time - no runtime check
= a / b;

fn factorial(n: i64) -> i64
  pre n >= 0       // No runtime bounds check
  post ret >= 1    // No runtime assertion
= if n <= 1 { 1 } else { n * factorial(n - 1) };
```

In C, you'd need:
```c
int divide(int a, int b) {
    assert(b != 0);  // Runtime check - overhead
    return a / b;
}
```

### 2. Expression-Based Design

Everything is an expression, enabling better optimization:

```bmb
// Single expression - optimizer can inline and simplify
fn abs(x: i64) -> i64 = if x < 0 { 0 - x } else { x };

// vs C-style statements that require more analysis
int abs(int x) {
    if (x < 0) {
        return -x;
    } else {
        return x;
    }
}
```

### 3. Tail Call Optimization

Recursive functions are optimized to loops:

```bmb
fn sum_to_n(n: i64) -> i64 = sum_iter(n, 0);

fn sum_iter(n: i64, acc: i64) -> i64 =
    if n <= 0 { acc }
    else { sum_iter(n - 1, acc + n) };  // Tail call - becomes loop
```

Generated code is equivalent to:
```c
int64_t sum_to_n(int64_t n) {
    int64_t acc = 0;
    while (n > 0) {
        acc += n;
        n--;
    }
    return acc;
}
```

### 4. Zero-Cost Safety with Dependent Types (v0.52+)

BMB's dependent types eliminate safety checks entirely at compile time:

```bmb
// Traditional approach: Runtime bounds check on every access
fn sum_old(arr: [i64]) -> i64 = {
    let total = 0;
    for i in 0..arr.len() {
        total = total + arr[i];  // Hidden: if (i >= len) panic
    }
    total
};

// BMB v0.52+: Zero-cost bounds safety with Fin[N]
fn sum_new(arr: [i64; N]) -> i64 = {
    let total = 0;
    for i: Fin[N] in 0..N {
        total = total + arr[i];  // NO check: type proves 0 <= i < N
    }
    total
};
```

**Assembly comparison**:
```asm
; Traditional (with bounds check)
loop:
    cmp rax, rcx        ; bounds check
    jae panic           ; conditional jump
    mov rdx, [rbx+rax*8]
    add rsi, rdx
    inc rax
    jmp loop

; BMB Fin[N] (zero-cost)
loop:
    mov rdx, [rbx+rax*8]  ; direct access
    add rsi, rdx
    inc rax
    cmp rax, rcx
    jl loop
```

**Performance Impact**: Gate #3.2/3.3 achieved **0% bounds/overflow check overhead**.

### 5. Range Arithmetic for Overflow Safety

```bmb
// Range[lo, hi] proves value is within bounds at compile time
type Byte = Range[0, 255];
type Percentage = Range[0, 100];

// Compiler proves no overflow: 100 + 100 = 200 fits in i64
fn add_percentages(a: Percentage, b: Percentage) -> Range[0, 200]
= a + b;  // No overflow check generated

// Array indexing with Range proves bounds
fn safe_get(arr: [i64; 1000], idx: Range[0, 999]) -> i64
= arr[idx];  // No bounds check: Range proves idx < 1000
```

### 6. LLVM Optimizations with `disjoint`

The `disjoint` predicate enables LLVM's most aggressive optimizations:

```bmb
// With disjoint, LLVM can use SIMD and memcpy
fn copy_fast(src: [i64; N], dst: [i64; N]) -> i64
  pre disjoint(src, dst)  // Generates LLVM noalias
= {
    for i: Fin[N] in 0..N { dst[i] = src[i]; }
    0
};

// Without disjoint, LLVM must assume aliasing (slower)
fn copy_slow(src: [i64], dst: [i64]) -> i64 = {
    for i in 0..src.len() { dst[i] = src[i]; }
    0
};
```

### 7. Direct LLVM Codegen

BMB compiles directly to LLVM IR, benefiting from all LLVM optimizations:

```bash
# Compile with optimizations
bmb build main.bmb -o main -O3

# Emit LLVM IR for inspection
bmb build main.bmb --emit-llvm
```

## Optimization Techniques

### Avoid Allocation in Hot Paths

```bmb
// Bad: creates string each iteration
fn process_bad(items: Vec<i64>) -> i64 = {
    let results = vec_new();
    process_iter(items, 0, results)
};

// Good: use accumulator
fn process_good(items: Vec<i64>) -> i64 = sum_iter(items, 0, 0);

fn sum_iter(items: Vec<i64>, idx: i64, acc: i64) -> i64 =
    if idx >= vec_len(items) { acc }
    else { sum_iter(items, idx + 1, acc + vec_get(items, idx)) };
```

### Use Bit Operations

```bmb
// Slow: modulo operation
fn is_even_slow(n: i64) -> bool = (n - (n / 2) * 2) == 0;

// Fast: bitwise AND
fn is_even_fast(n: i64) -> bool = (n band 1) == 0;

// Power of 2 multiplication
fn mul_by_8(n: i64) -> i64 = n << 3;

// Fast division by power of 2
fn div_by_4(n: i64) -> i64 = n >> 2;
```

### Minimize Function Call Overhead

```bmb
// Bad: extra function call
fn abs(x: i64) -> i64 = if x < 0 { negate(x) } else { x };
fn negate(x: i64) -> i64 = 0 - x;

// Good: inline the operation
fn abs(x: i64) -> i64 = if x < 0 { 0 - x } else { x };
```

### Prefer Iteration Over Recursion (When Needed)

For non-tail-recursive algorithms, rewrite as iteration:

```bmb
// Tail recursive - good, becomes loop
fn fib_tail(n: i64) -> i64 = fib_iter(n, 0, 1);
fn fib_iter(n: i64, a: i64, b: i64) -> i64 =
    if n <= 0 { a } else { fib_iter(n - 1, b, a + b) };

// vs naive recursion - exponential time
fn fib_slow(n: i64) -> i64 =
    if n <= 1 { n } else { fib_slow(n - 1) + fib_slow(n - 2) };
```

## Memory-Efficient Patterns

### In-Place Operations

```bmb
fn reverse_in_place(arr: i64, len: i64) -> i64
  pre arr != 0
  pre len >= 0
= reverse_iter(arr, 0, len - 1);

fn reverse_iter(arr: i64, lo: i64, hi: i64) -> i64 =
    if lo >= hi { 0 }
    else {
        let tmp = load_i64(arr + lo * 8);
        let s1 = store_i64(arr + lo * 8, load_i64(arr + hi * 8));
        let s2 = store_i64(arr + hi * 8, tmp);
        reverse_iter(arr, lo + 1, hi - 1)
    };
```

### Stack Allocation Pattern

```bmb
// Use fixed-size buffers when possible
fn process_small(data: i64) -> i64 = {
    // Small buffer on stack (conceptually)
    let buf0 = 0; let buf1 = 0; let buf2 = 0; let buf3 = 0;
    // Process with local variables
    process_with_buf(data, buf0, buf1, buf2, buf3)
};
```

## Benchmarking

### Run Benchmarks

```bash
# In WSL with LLVM installed
cd ecosystem/benchmark-bmb

# Compile all BMB benchmarks to native
for f in benches/*/*/bmb/main.bmb; do
    bmb build "$f" -o "${f%.bmb}" -O3
done

# Run benchmark suite
./runner/target/release/benchmark-bmb run all -i 5 -w 2

# Check gate criteria
./runner/target/release/benchmark-bmb gate 3.1 -v
```

### Write Your Own Benchmarks

```bmb
fn benchmark_function() -> i64 = {
    let start = time_now();

    // Run operation many times
    let result = run_iterations(1000000, 0);

    let end = time_now();
    let elapsed = end - start;

    let p = println(elapsed);
    result
};

fn run_iterations(n: i64, acc: i64) -> i64 =
    if n <= 0 { acc }
    else { run_iterations(n - 1, acc + expensive_operation()) };
```

## Gate Criteria

| Gate | Requirement | Status |
|------|-------------|--------|
| #3.1 | Compute ≤ 1.10x C | ✅ 0.89x-0.99x |
| #3.2 | Bounds check overhead 0% | ✅ Fin[N] types (v0.54.5) |
| #3.3 | Overflow check overhead 0% | ✅ Range types (v0.54.5) |
| #3.4 | 3+ benchmarks faster than C | ✅ fibonacci, mandelbrot, spectral_norm |
| #4.1 | Self-compile < 60s | ✅ 0.56s |

## Comparison with Other Languages

| Language | Paradigm | Typical Overhead | Safety |
|----------|----------|------------------|--------|
| C | Manual | Baseline | None |
| Rust | Ownership | ~0-5% | Compile-time |
| BMB | Contracts | ~0-10% | Compile-time |
| Go | GC | ~20-50% | Runtime |
| Java | GC + JIT | ~50-200% | Runtime |

## Profiling

```bash
# Profile with perf (Linux)
perf record ./my_program
perf report

# Profile with Instruments (macOS)
xcrun xctrace record --template "Time Profiler" --launch ./my_program

# LLVM optimization report
bmb build main.bmb -O3 --emit-llvm -Rpass=inline
```

## Best Practices Summary

1. **Let contracts replace runtime checks** - Zero overhead safety
2. **Use Fin[N] for array indexing** - Eliminates bounds checks entirely
3. **Use Range[lo, hi] for bounded arithmetic** - Eliminates overflow checks
4. **Use disjoint for non-aliasing** - Enables SIMD/vectorization
5. **Use tail recursion** - Compiler optimizes to loops
6. **Prefer bit operations** - Faster than arithmetic
7. **Minimize allocations** - Use accumulators and in-place operations
8. **Profile before optimizing** - Measure, don't guess
9. **Trust LLVM** - Let the optimizer do its job

## Performance Trade-offs (v0.51)

BMB prioritizes **type safety** over raw performance in certain areas. Understanding these trade-offs helps write optimal code.

### 1. FFI String Wrapper Overhead

BMB uses `BmbString` (a fat pointer with length) instead of raw `char*`:

```
C:   stat(".", &st)                 // Direct syscall
BMB: file_exists(BmbString* path)   // Wrapper → stat(path->data, &st)
```

**Impact**: ~3x overhead for syscall-heavy loops (10,000 file_exists calls)

**Why**: Type safety - BmbString provides length information and null-termination guarantees.

**Mitigation**: Batch syscalls, cache results, or use raw FFI for performance-critical paths.

### 2. Recursion vs Loops

BMB's while/for syntax has limitations for complex state changes:

```bmb
// This works:
while i < n { { i = i + 1; () } };

// Complex multi-variable updates are harder:
while cond { {
    x = f(x);    // Multiple assignments
    y = g(y);    // require careful structuring
    ()
} };
```

**Impact**: Some algorithms (e.g., fannkuch) are ~2x slower due to recursion overhead.

**Mitigation**: Use tail recursion (TCO applies), or restructure into simpler loops.

### 3. String Concatenation

String `+` creates new allocations:

```bmb
// Inefficient: 5 allocations
let s = "a" + b + "c" + d + "e";

// Better: Use StringBuilder (planned stdlib addition)
let sb = sb_new();
sb_push(sb, "a"); sb_push(sb, b); ...
```

**Impact**: ~1.5x overhead for string-heavy code (http_parse, json_serialize).

**Mitigation**: Minimize concatenations, use format functions, or batch string building.

### Performance Summary (v0.51)

| Category | Pass Rate | Notes |
|----------|-----------|-------|
| **Overall** | 77% (37/48) | ≤1.10x C |
| **Bootstrap** | 100% | BMB compiling BMB |
| **Zero-Cost Proof** | 100% | Contracts have zero overhead |
| **Compute** | 80% | LLVM optimization excellent |
| **Real World** | 43% | String handling needs work |

**Key Insight**: BMB excels at computational code and contract verification. String-heavy and syscall-heavy code has overhead due to type safety design.

## Next Steps

- [Contracts](CONTRACTS.md) - Understanding zero-cost verification
- [Systems](SYSTEMS.md) - Low-level performance patterns
- [From Rust](FROM_RUST.md) - Performance comparison with Rust

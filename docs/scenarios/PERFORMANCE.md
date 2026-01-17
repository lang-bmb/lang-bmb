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

### 4. Direct LLVM Codegen

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
| #3.2 | Benchmarks Game ≤ 1.05x C | 🔄 Testing |
| #3.3 | 3+ benchmarks faster than C | ✅ fibonacci, mandelbrot, spectral_norm |
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
2. **Use tail recursion** - Compiler optimizes to loops
3. **Prefer bit operations** - Faster than arithmetic
4. **Minimize allocations** - Use accumulators and in-place operations
5. **Profile before optimizing** - Measure, don't guess
6. **Trust LLVM** - Let the optimizer do its job

## Next Steps

- [Contracts](CONTRACTS.md) - Understanding zero-cost verification
- [Systems](SYSTEMS.md) - Low-level performance patterns
- [From Rust](FROM_RUST.md) - Performance comparison with Rust

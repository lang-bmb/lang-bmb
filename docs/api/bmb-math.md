# bmb-math — Math Module API

Mathematical functions with contracts for zero-overhead verification.

## Constants

| Function | Value | Description |
|----------|-------|-------------|
| `pi()` | 3.14159... | Circle ratio |
| `e()` | 2.71828... | Euler's number |
| `tau()` | 6.28318... | 2π |
| `sqrt2()` | 1.41421... | √2 |
| `ln2()` | 0.69314... | ln(2) |
| `ln10()` | 2.30258... | ln(10) |

## Float Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `fabs(x)` | `(f64) -> f64` | Absolute value (`post ret >= 0.0`) |
| `fsign(x)` | `(f64) -> f64` | Sign: -1.0, 0.0, or 1.0 |
| `fmin(a, b)` | `(f64, f64) -> f64` | Minimum |
| `fmax(a, b)` | `(f64, f64) -> f64` | Maximum |
| `fclamp(x, lo, hi)` | `(f64, f64, f64) -> f64` | Clamp to range |

## Power / Root

| Function | Signature | Contract |
|----------|-----------|----------|
| `ipow(base, exp)` | `(i64, i64) -> i64` | `pre exp >= 0` |
| `sqrt(x)` | `(f64) -> f64` | Newton-Raphson, 15 iterations |
| `exp(x)` | `(f64) -> f64` | Taylor series, 20 terms |
| `ln(x)` | `(f64) -> f64` | Range reduction + atanh |
| `log10(x)` | `(f64) -> f64` | Via ln |
| `log2(x)` | `(f64) -> f64` | Via ln |

## Trigonometry

| Function | Signature | Description |
|----------|-----------|-------------|
| `sin(x)` | `(f64) -> f64` | Sine (Taylor series) |
| `cos(x)` | `(f64) -> f64` | Cosine |
| `tan(x)` | `(f64) -> f64` | Tangent |

## Integer Math

| Function | Signature | Contract |
|----------|-----------|----------|
| `gcd(a, b)` | `(i64, i64) -> i64` | `pre a >= 0 and b >= 0` |
| `lcm(a, b)` | `(i64, i64) -> i64` | `pre a > 0 and b > 0` |
| `factorial(n)` | `(i64) -> i64` | `pre n >= 0` |
| `fibonacci(n)` | `(i64) -> i64` | `pre n >= 0`, iterative |
| `isqrt(n)` | `(i64) -> i64` | `pre n >= 0`, integer sqrt |

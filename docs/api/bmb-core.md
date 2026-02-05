# packages/bmb-core/src/lib.bmb

Auto-generated documentation.

## Table of Contents

- [`bool_not`](#bool_not)
- [`bool_implies`](#bool_implies)
- [`iff`](#iff)
- [`xor`](#xor)
- [`to_int`](#to_int)
- [`from_int`](#from_int)
- [`select`](#select)
- [`abs`](#abs)
- [`min`](#min)
- [`max`](#max)
- [`clamp`](#clamp)
- [`sign`](#sign)
- [`in_range`](#in_range)
- [`diff`](#diff)
- [`pow`](#pow)
- [`gcd`](#gcd)
- [`lcm`](#lcm)
- [`is_even`](#is_even)
- [`is_odd`](#is_odd)

## Functions

### `bool_not`

```bmb
pub fn bool_not(x: bool) -> bool
```

Logical NOT (explicit function form)

---

### `bool_implies`

```bmb
pub fn bool_implies(a: bool, b: bool) -> bool
```

Logical implication: a implies b

---

### `iff`

```bmb
pub fn iff(a: bool, b: bool) -> bool
```

Logical equivalence: a iff b

---

### `xor`

```bmb
pub fn xor(a: bool, b: bool) -> bool
```

Exclusive OR: exactly one is true

---

### `to_int`

```bmb
pub fn to_int(b: bool) -> i64
```

Convert boolean to integer (0 or 1)

---

### `from_int`

```bmb
pub fn from_int(x: i64) -> bool
```

Convert integer to boolean (0 = false, else true)

---

### `select`

```bmb
pub fn select(cond: bool, a: i64, b: i64) -> i64
```

Three-way conditional selection

---

### `abs`

```bmb
pub fn abs(x: i64) -> i64
```

Absolute value of an integer

---

### `min`

```bmb
pub fn min(a: i64, b: i64) -> i64
```

Minimum of two integers

---

### `max`

```bmb
pub fn max(a: i64, b: i64) -> i64
```

Maximum of two integers

---

### `clamp`

```bmb
pub fn clamp(x: i64, lo: i64, hi: i64) -> i64
```

Clamp value to range [lo, hi]

---

### `sign`

```bmb
pub fn sign(x: i64) -> i64
```

Sign of an integer (-1, 0, or 1)

---

### `in_range`

```bmb
pub fn in_range(x: i64, lo: i64, hi: i64) -> bool
```

Check if value is in range [lo, hi] inclusive

---

### `diff`

```bmb
pub fn diff(a: i64, b: i64) -> i64
```

Difference between two values (always non-negative)

---

### `pow`

```bmb
pub fn pow(base: i64, exp: i64) -> i64
```

Integer power: base^exp (for non-negative exponents)

---

### `gcd`

```bmb
pub fn gcd(a: i64, b: i64) -> i64
```

Greatest common divisor

---

### `lcm`

```bmb
pub fn lcm(a: i64, b: i64) -> i64
```

Least common multiple

---

### `is_even`

```bmb
pub fn is_even(n: i64) -> bool
```

Check if n is even

---

### `is_odd`

```bmb
pub fn is_odd(n: i64) -> bool
```

Check if n is odd

---


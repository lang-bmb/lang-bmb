# Overflow Detect

## Description

Multiply two integers and detect whether the result overflows a 32-bit signed integer range.

A result **overflows** if it is greater than 2,147,483,647 (2^31 - 1) or less than -2,147,483,648 (-2^31). The product must be computed using 64-bit arithmetic to preserve the full result even when it overflows 32-bit.

**Input** (stdin):
- First integer: `t`, number of test cases
- For each test case: two integers `a` and `b` (may be large, use 64-bit integers)

**Output** (stdout):
- For each test case, print: `overflow product`
  - `overflow`: 1 if the product exceeds 32-bit range, 0 otherwise
  - `product`: the actual 64-bit product `a * b`
  - Separated by a space, followed by newline

## Example

Input:
```
3 1 2 3 100000 100000 1 -3 1000000000
```

Output:
```
0 6
1 10000000000
1 -3000000000
```

Explanation:
- 1×2=2 — fits in i32 (≤2147483647) → 0 6
- 100000×100000=10000000000 — exceeds 2147483647 → 1 10000000000
- -3×1000000000=-3000000000 — less than -2147483648 → overflow → 1 -3000000000

## Constraints

- 1 ≤ t ≤ 100
- -2,000,000,000 ≤ a, b ≤ 2,000,000,000
- Compute using 64-bit (i64) arithmetic
- The product `a * b` always fits in i64

## Key Notes

- Use `i64` for `a`, `b`, and the product to avoid overflow during multiplication
- Overflow check: `product > 2147483647` or `product < -2147483648`
- Print the actual 64-bit product even when it overflows 32-bit

## Category

Practical / math / overflow detection

## BMB Notes

**CRITICAL**: There is NO operation code. Input is EXACTLY: first `t`, then for each test case exactly TWO integers `a` and `b`. Do NOT read an `op` code. Just multiply and check i32 bounds.

```
fn main() -> i64 = {
    let t: i64 = read_int();
    for _i in 0..t {
        let a: i64 = read_int();
        let b: i64 = read_int();
        let product: i64 = a * b;
        let overflow: i64 = if product > 2147483647 { 1 } else { if product < -2147483648 { 1 } else { 0 } };
        print(overflow); print_str(" "); println(product)
    };
    0
};
```

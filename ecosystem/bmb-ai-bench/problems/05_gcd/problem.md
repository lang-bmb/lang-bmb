# Greatest Common Divisor

## Description

Compute the GCD of two non-negative integers using the Euclidean algorithm.

**Input** (stdin):
- Two integers: `a` and `b` (0 <= a, b <= 10^18, not both zero)

**Output** (stdout):
- Print GCD(a, b)

## Example

Input:
```
12 8
```

Output:
```
4
```

## Constraints

- 0 <= a, b <= 10^18
- At least one of a, b is non-zero
- All values fit in a 64-bit signed integer

## Category

Algorithm (number theory)

## BMB Notes
- Iterative Euclidean: while b != 0, swap a with b, b with a % b
- Both a and b can be 0 but not simultaneously; handle a=0 → result is b
```
let mut a: i64 = read_int(); let mut b: i64 = read_int();
while b != 0 { let tmp: i64 = b; set b = a % b; set a = tmp };
println(a);
0

```

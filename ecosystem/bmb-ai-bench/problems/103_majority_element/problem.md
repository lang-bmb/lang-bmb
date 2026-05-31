# Majority Element

## Description

Find the majority element — the element that appears more than ⌊n/2⌋ times. A majority element is guaranteed to exist.

Use Boyer-Moore Voting Algorithm for O(n) time, O(1) space.

**Input** (stdin):
- First integer: `n`, array size (1 <= n <= 10000)
- Next `n` integers: array elements

**Output** (stdout):
- Print the majority element

## Example

Input:
```
7 3 2 3 1 3 3 2
```

Output:
```
3
```

## Constraints

- 1 <= n <= 10000
- A majority element always exists
- All values fit in i64

## Category

Algorithm (voting)

## BMB Notes
- Boyer-Moore: maintain `candidate` and `count`
- When `count == 0`: set new candidate, count = 1
- When val == candidate: count++; else: count--
```
fn main() -> i64 = {
    let n: i64 = read_int();
    let v = vec_new();
    for _i in 0..n { let x = read_int(); vec_push(v, x) };
    let mut candidate = vec_get(v, 0); let mut count = 1;
    for i in 1..n {
        let val = vec_get(v, i);
        if count == 0 { candidate = val; count = 1 }
        else if val == candidate { count = count + 1 }
        else { count = count - 1 }
    };
    println(candidate);
    vec_free(v);
    0
};
```

# Single Element

## Description

Given an array, print the first element, the last element, and the count.

**Input** (stdin):
- First integer: `n`, the number of elements (1 <= n)
- Next `n` integers: the array elements

**Output** (stdout):
- Line 1: first element (index 0)
- Line 2: last element (index n-1)
- Line 3: n (count)

## IMPORTANT: Exactly 3 Lines of Output

Output EXACTLY 3 lines: first element, last element, n. Nothing else.

- Do NOT print elements while reading them
- Do NOT print min, max, sum, or any computed value
- Do NOT loop over elements to print them
- Do NOT print any element except index 0 and index n-1

Correct approach:
```
let n: i64 = read_int();
let v = vec_new();
let mut i: i64 = 0;
while i < n {
    let _p = vec_push(v, read_int());
    i = i + 1
};
println(vec_get(v, 0));
println(vec_get(v, n - 1));
println(n);
0
```

## Example

Input:
```
3 -1 0 1
```

Output:
```
-1
1
3
```

Input:
```
1 42
```

Output:
```
42
42
1
```

(When n=1, first=last=the single element)

## Constraints

- 1 <= n <= 100000
- All values fit in a 64-bit signed integer

## Category

Edge (boundary access)

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
let n = read_int();
// Read all into vec
let v = vec_new();
let i = 0;
while i < n {
    let val = read_int();
    let _p = vec_push(v, val);
    i = i + 1
};
// Print exactly 3 things
let _a = println(vec_get(v, 0));      // first
let _b = println(vec_get(v, n - 1)); // last
let _c = println(n);                  // count
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

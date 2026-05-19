# Linked List Sum

## Description

Simulate a singly linked list using arrays. Build the list from input, then compute the sum of all elements.

This tests the ability to simulate pointer-based data structures using arrays.

**Input** (stdin):
- First integer: `n`, the number of elements (0 <= n <= 100000)
- Next `n` integers: the elements to insert (in order)

**Output** (stdout):
- Print the sum of all elements

## Example

Input:
```
4 10 20 30 40
```

Output:
```
100
```

## Constraints

- 0 <= n <= 100000
- Sum fits in a 64-bit signed integer

## Category

System (data structure simulation)

## BMB Notes
- BMB has no pointer/struct linked list; simulate with a vec (equivalent to array-based list)
- Since we only need the sum, just read elements and accumulate — no need to store the list
```
let n: i64 = read_int();
let mut sum: i64 = 0;
for _i in 0..n {
    sum = sum + read_int()
};
println(sum);
0
```

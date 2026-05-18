# Ring Buffer

Circular buffer with fixed capacity. When the buffer is **full**, writing a new value **overwrites** the oldest element (head advances).

## Input
- First integer: capacity
- Second integer: n (number of operations)
- Operations:
  - `op=1` (write val): always writes val to the buffer. If full, head advances (oldest overwritten).
  - `op=2` (read): remove and print the oldest element (front). Print -1 if empty.
  - `op=3` (size): print current element count.

## Output

`op=2` and `op=3` each print one line.

## Example

Input:
```
3 5 1 10 1 20 1 30 3 2
```
Output:
```
3
10
```
(capacity=3, push 10,20,30 → size=3 → dequeue oldest=10)

Input:
```
2 4 1 10 1 20 1 30 2
```
Output:
```
20
```
(capacity=2, push 10→[10], push 20→[10,20] full, push 30→overwrites head: [30,20] with head at 20 → dequeue=20)

## Constraints

- 1 <= capacity <= 1000
- All read ops when non-empty, prints are valid

## Category

System (circular buffer)

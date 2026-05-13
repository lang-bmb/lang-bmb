# Python Quick Reference (for bmb-ai-bench problems)

## Basics
```python
import sys

def main():
    # body
    pass

if __name__ == "__main__":
    main()
```

## I/O
```python
n = int(input())                        # read single int
x = float(input())                      # read single float
line = input()                          # read line (strips newline)

a, b = map(int, input().split())        # read two ints on one line
nums = list(map(int, input().split()))  # read list of ints

# Fast input (for many values):
data = sys.stdin.read().split()
idx = 0
n = int(data[idx]); idx += 1

print(n)                                # print int + newline
print(f"{x:.6f}")                       # print float, 6 decimal places
print(a, b)                             # print two values space-separated
print(a, b, sep="")                     # print without separator
```

## Types
- `int` — arbitrary precision integer (no overflow)
- `float` — 64-bit floating point
- `str` — immutable string
- `bool` — `True` / `False`
- `list` — dynamic array (mutable)
- `dict` — hash map

## Control Flow
```python
if x > 0:
    ...
elif x == 0:
    ...
else:
    ...

while cond:
    ...

for i in range(n):       # 0, 1, ..., n-1
    ...

for i in range(a, b):    # a, a+1, ..., b-1
    ...

break    # exit loop
continue # next iteration
return val  # return from function
```

## Lists (dynamic arrays)
```python
a = []                  # empty list
a = [0] * n             # list of n zeros
a.append(x)             # add to end
a.pop()                 # remove last
a[i]                    # access by index (0-based)
a[i] = val              # set by index
len(a)                  # length
a.sort()                # sort in place (ascending)
a.sort(reverse=True)    # sort descending
sorted(a)               # returns new sorted list
```

## Dictionaries
```python
d = {}                   # empty dict
d[key] = value           # set
val = d[key]             # get (KeyError if missing)
val = d.get(key, -1)     # get with default
key in d                 # membership test
len(d)                   # number of keys
for k, v in d.items():   # iterate
    ...
```

## Strings
```python
s = "hello"
len(s)                   # length
s[i]                     # character at index
s.split()                # split by whitespace
s.split(",")             # split by comma
s.strip()                # remove leading/trailing whitespace
s + t                    # concatenation
str(n)                   # int to string
int(s)                   # string to int
```

## Pattern: Read n items
```python
n = int(input())
a = list(map(int, input().split()))
```

## Pattern: Multiple test cases
```python
t = int(input())
for _ in range(t):
    n = int(input())
    # solve one case
    print(result)
```

## Pattern: Stack
```python
stack = []
stack.append(val)        # push
top = stack[-1]          # peek
val = stack.pop()        # pop
```

## Pattern: Queue (efficient)
```python
from collections import deque
q = deque()
q.append(val)            # enqueue
val = q.popleft()        # dequeue
```

## Pattern: Counter / frequency map
```python
from collections import defaultdict
freq = defaultdict(int)
for x in arr:
    freq[x] += 1
```

## Pattern: Sort with key
```python
pairs.sort(key=lambda x: x[0])           # sort by first element
pairs.sort(key=lambda x: (x[1], x[0]))  # sort by second, then first
```

## Pattern: 2D grid
```python
grid = [[0] * cols for _ in range(rows)]
grid[r][c] = val
```

## Common Pitfalls
- Python `int` has no overflow — use freely for large values
- Integer division: `//` truncates, `/` returns float
- `input()` strips the newline — no need to `.strip()` for numeric input
- `print()` always adds newline — use `end=""` to suppress
- Lists are 0-indexed; negative index counts from end (`a[-1]` = last)
- `range(n)` gives `0..n-1` (exclusive end)
- Copying a list: `b = a[:]` or `b = list(a)` (not `b = a` which is a reference)
- For large inputs, `sys.stdin.read()` is faster than repeated `input()`

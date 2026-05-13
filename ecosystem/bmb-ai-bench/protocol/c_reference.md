# C Language Quick Reference (for bmb-ai-bench problems)

## Basics
```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    /* body */
    return 0;
}
```

## I/O
```c
int n;
scanf("%d", &n);             // read int

long long x;
scanf("%lld", &x);           // read long long (64-bit int)

double f;
scanf("%lf", &f);            // read double

printf("%d\n", n);           // print int + newline
printf("%lld\n", x);         // print long long + newline
printf("%f\n", f);           // print double (6 decimal places)
printf("%.6f\n", f);         // print double, 6 decimal places

// Read line of text
char buf[1024];
fgets(buf, sizeof(buf), stdin);
buf[strcspn(buf, "\n")] = 0; // strip trailing newline
```

## Types
- `int` — 32-bit signed integer
- `long long` — 64-bit signed integer (use for large values)
- `double` — 64-bit floating point
- `char` — single byte character

## Control Flow
```c
if (x > 0) { ... } else { ... }
while (cond) { ... }
for (int i = 0; i < n; i++) { ... }
break;       // exit loop
continue;    // next iteration
return val;  // return from function
```

## Dynamic Arrays (heap)
```c
long long *arr = malloc(n * sizeof(long long));
arr[0] = 42;               // index access
arr[i] = arr[j] + 1;
free(arr);                 // deallocate
```

## String Operations
```c
int len = strlen(s);       // string length
int cmp = strcmp(a, b);    // 0 if equal, <0 if a<b, >0 if a>b
strncpy(dst, src, n);      // copy at most n bytes
```

## Functions
```c
long long factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
```

## Pattern: Read n items into array
```c
int n;
scanf("%d", &n);
long long *a = malloc(n * sizeof(long long));
for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
```

## Pattern: Multiple test cases
```c
int t;
scanf("%d", &t);
while (t--) {
    int n;
    scanf("%d", &n);
    /* solve one case */
    printf("%d\n", result);
}
```

## Pattern: 2D array
```c
// Fixed size (stack):
int grid[100][100];

// Dynamic:
long long **grid = malloc(rows * sizeof(long long *));
for (int i = 0; i < rows; i++)
    grid[i] = malloc(cols * sizeof(long long));
// ... use grid[i][j] ...
for (int i = 0; i < rows; i++) free(grid[i]);
free(grid);
```

## Pattern: Stack with array
```c
long long stack[1000];
int top = 0;
stack[top++] = val;       // push
long long t = stack[--top]; // pop
long long peek = stack[top - 1]; // top
```

## Pattern: Sort array
```c
#include <stdlib.h>
int cmp_ll(const void *a, const void *b) {
    long long x = *(long long*)a, y = *(long long*)b;
    if (x < y) return -1;
    if (x > y) return 1;
    return 0;
}
qsort(arr, n, sizeof(long long), cmp_ll);
```

## Pattern: String split by space (tokenize)
```c
char line[1024];
fgets(line, sizeof(line), stdin);
char *token = strtok(line, " \t\n");
while (token) {
    /* process token */
    token = strtok(NULL, " \t\n");
}
```

## Common Pitfalls
- Use `long long` and `%lld` for values > 2 billion (avoid 32-bit overflow)
- `malloc` returns `void*` — cast when needed, or let C implicitly convert
- `scanf` needs `&var` for scalar types (NOT for arrays/strings)
- `printf("%.Xf")` for specific decimal places in float output
- Integer division `a/b` truncates toward zero — use `(double)a/b` for float division
- Always `free()` after `malloc()` to avoid memory leaks
- Array index starts at 0 in C
- Include `<string.h>` for string functions, `<math.h>` for math + link `-lm`

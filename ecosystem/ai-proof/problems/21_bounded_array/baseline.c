#include <stdio.h>
#include <stdlib.h>
#include <assert.h>

/* Bounded array access with explicit bounds assertion. */
long long bounded_get(long long *arr, int n, int idx) {
    assert(idx >= 0 && idx < n);
    return arr[idx];
}

int main(void) {
    int n;
    scanf("%d", &n);

    long long *arr = (long long *)malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) {
        scanf("%lld", &arr[i]);
    }

    int idx;
    scanf("%d", &idx);

    long long val = bounded_get(arr, n, idx);
    printf("%lld\n", val);

    free(arr);
    return 0;
}

#include <stdio.h>
#include <stdlib.h>

int binary_search(long long *arr, int n, long long target) {
    int lo = 0, hi = n - 1;
    while (lo <= hi) {
        int mid = lo + (hi - lo) / 2;
        if (arr[mid] == target) return mid;
        else if (arr[mid] < target) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

int main(void) {
    long long target;
    int n;
    scanf("%lld %d", &target, &n);

    long long *arr = NULL;
    if (n > 0) {
        arr = (long long *)malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) {
            scanf("%lld", &arr[i]);
        }
    }

    int result = binary_search(arr, n, target);
    printf("%d\n", result);

    free(arr);
    return 0;
}

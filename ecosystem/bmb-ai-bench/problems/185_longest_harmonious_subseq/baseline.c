#include <stdio.h>
#include <stdlib.h>
int cmp(const void *a, const void *b) {
    return (*(int*)a - *(int*)b);
}
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    qsort(arr, n, sizeof(int), cmp);
    int best = 0, lo = 0;
    for (int hi = 1; hi <= n; hi++) {
        if (hi == n || arr[hi] - arr[lo] > 1) {
            lo = hi;
        } else if (arr[hi] - arr[lo] == 1) {
            /* find where the value changes */
            int mid = lo;
            while (mid < hi && arr[mid] == arr[lo]) mid++;
            int len = hi - lo + 1;
            if (len > best) best = len;
        }
    }
    /* Simpler two-pointer approach */
    best = 0;
    int l = 0;
    for (int r = 0; r < n; r++) {
        while (arr[r] - arr[l] > 1) l++;
        if (arr[r] - arr[l] == 1) {
            int len = r - l + 1;
            if (len > best) best = len;
        }
    }
    printf("%d\n", best);
    free(arr);
    return 0;
}

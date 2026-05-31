#include <stdio.h>
#include <stdlib.h>

int cmp(const void *a, const void *b) { return *(int*)a - *(int*)b; }

int main() {
    int n;
    scanf("%d", &n);
    int *arr = malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    qsort(arr, n, sizeof(int), cmp);
    int best = abs(arr[1] - arr[0]);
    for (int i = 1; i < n - 1; i++) {
        int d = abs(arr[i+1] - arr[i]);
        if (d < best) best = d;
    }
    printf("%d\n", best);
    free(arr);
    return 0;
}

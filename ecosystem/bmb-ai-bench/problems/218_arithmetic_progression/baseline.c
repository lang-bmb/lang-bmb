#include <stdio.h>
#include <stdlib.h>
int cmp(const void *a, const void *b) { return *(int*)a - *(int*)b; }
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    if (n <= 1) { printf("1\n"); free(arr); return 0; }
    qsort(arr, n, sizeof(int), cmp);
    int diff = arr[1] - arr[0];
    int ok = 1;
    for (int i = 2; i < n; i++)
        if (arr[i] - arr[i-1] != diff) { ok = 0; break; }
    printf("%d\n", ok);
    free(arr);
    return 0;
}

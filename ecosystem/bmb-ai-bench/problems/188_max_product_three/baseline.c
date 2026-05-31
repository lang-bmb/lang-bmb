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
    long long a = (long long)arr[n-1]*arr[n-2]*arr[n-3];
    long long b = (long long)arr[0]*arr[1]*arr[n-1];
    printf("%lld\n", a > b ? a : b);
    free(arr);
    return 0;
}

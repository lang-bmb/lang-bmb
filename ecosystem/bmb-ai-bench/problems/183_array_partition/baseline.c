#include <stdio.h>
#include <stdlib.h>
int cmp(const void *a, const void *b) {
    return (*(int*)a - *(int*)b);
}
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(2*n*sizeof(int));
    for (int i = 0; i < 2*n; i++) scanf("%d", &arr[i]);
    qsort(arr, 2*n, sizeof(int), cmp);
    long long sum = 0;
    for (int i = 0; i < 2*n; i += 2) sum += arr[i];
    printf("%lld\n", sum);
    free(arr);
    return 0;
}

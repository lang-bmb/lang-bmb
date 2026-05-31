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
    int unique = 1;
    for (int i = 1; i < n; i++) if (arr[i] != arr[i-1]) unique++;
    int half = n/2;
    printf("%d\n", unique < half ? unique : half);
    free(arr);
    return 0;
}

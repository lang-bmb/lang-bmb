#include <stdio.h>
#include <stdlib.h>
int cmp_desc(const void *a, const void *b) { return *(int*)b - *(int*)a; }
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    int sz = n;
    while (sz > 1) {
        qsort(arr, sz, sizeof(int), cmp_desc);
        int y = arr[0], x = arr[1];
        arr[1] = y - x;
        arr[0] = 0;
        qsort(arr, sz, sizeof(int), cmp_desc);
        while (sz > 0 && arr[sz-1] == 0) sz--;
    }
    printf("%d\n", sz == 0 ? 0 : arr[0]);
    free(arr);
    return 0;
}

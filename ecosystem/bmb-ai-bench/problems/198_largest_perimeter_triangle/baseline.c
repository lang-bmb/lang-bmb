#include <stdio.h>
#include <stdlib.h>
int cmp_desc(const void *a, const void *b) { return *(int*)b - *(int*)a; }
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    qsort(arr, n, sizeof(int), cmp_desc);
    for (int i = 0; i + 2 < n; i++) {
        if ((long long)arr[i+1] + arr[i+2] > arr[i]) {
            printf("%d\n", arr[i] + arr[i+1] + arr[i+2]);
            free(arr);
            return 0;
        }
    }
    printf("0\n");
    free(arr);
    return 0;
}

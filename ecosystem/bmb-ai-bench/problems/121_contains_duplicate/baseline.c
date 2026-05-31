#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int cmp(const void *a, const void *b) {
    return (*(int*)a - *(int*)b);
}

int main() {
    int n;
    scanf("%d", &n);
    int *arr = malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &arr[i]);
    qsort(arr, n, sizeof(int), cmp);
    int dup = 0;
    for (int i = 1; i < n; i++) {
        if (arr[i] == arr[i-1]) { dup = 1; break; }
    }
    printf("%s\n", dup ? "true" : "false");
    free(arr);
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int cmp_asc(const void *a, const void *b) { return *(int*)a - *(int*)b; }
int main() {
    int n;
    scanf("%d", &n);
    int *arr = (int*)malloc(n*sizeof(int));
    int *sorted = (int*)malloc(n*sizeof(int));
    for (int i = 0; i < n; i++) { scanf("%d", &arr[i]); sorted[i] = arr[i]; }
    qsort(sorted, n, sizeof(int), cmp_asc);
    int count = 0;
    for (int i = 0; i < n; i++) if (arr[i] != sorted[i]) count++;
    printf("%d\n", count);
    free(arr); free(sorted);
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
int cmp(const void *a, const void *b) { return *(int*)a - *(int*)b; }
int main() {
    int n;
    scanf("%d", &n);
    int *t = (int*)malloc(n * sizeof(int));
    int *a = (int*)malloc(n * sizeof(int));
    for (int i = 0; i < n; i++) scanf("%d", &t[i]);
    for (int i = 0; i < n; i++) scanf("%d", &a[i]);
    qsort(t, n, sizeof(int), cmp);
    qsort(a, n, sizeof(int), cmp);
    int ok = 1;
    for (int i = 0; i < n; i++) if (t[i] != a[i]) { ok = 0; break; }
    printf("%d\n", ok);
    free(t); free(a);
    return 0;
}

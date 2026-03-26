#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(void) {
    int n, max_val; scanf("%d %d", &n, &max_val);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int *count = calloc(max_val + 1, sizeof(int));
    for (int i = 0; i < n; i++) count[a[i]]++;
    int idx = 0;
    for (int v = 0; v <= max_val; v++)
        for (int j = 0; j < count[v]; j++) a[idx++] = v;
    for (int i = 0; i < n; i++) { if (i > 0) printf(" "); printf("%lld", a[i]); }
    printf("\n");
    free(a); free(count);
    return 0;
}

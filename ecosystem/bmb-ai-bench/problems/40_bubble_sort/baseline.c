#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    for (int i = 0; i < n; i++) {
        int sw = 0;
        for (int j = 0; j < n - 1 - i; j++)
            if (a[j] > a[j+1]) { long long t = a[j]; a[j] = a[j+1]; a[j+1] = t; sw = 1; }
        if (!sw) break;
    }
    for (int i = 0; i < n; i++) { if (i > 0) printf(" "); printf("%lld", a[i]); }
    printf("\n");
    free(a);
    return 0;
}

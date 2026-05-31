#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long *out = malloc(n * sizeof(long long));
    int nz = 0;
    for (int i = 0; i < n; i++) if (a[i] != 0) out[nz++] = a[i];
    while (nz < n) out[nz++] = 0;
    for (int i = 0; i < n; i++) { if (i > 0) printf(" "); printf("%lld", out[i]); }
    printf("\n");
    free(a); free(out); return 0;
}

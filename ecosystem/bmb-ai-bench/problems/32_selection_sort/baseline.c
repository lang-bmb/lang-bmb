#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    for (int i = 0; i < n; i++) {
        int min_idx = i;
        for (int j = i + 1; j < n; j++)
            if (a[j] < a[min_idx]) min_idx = j;
        if (min_idx != i) { long long t = a[i]; a[i] = a[min_idx]; a[min_idx] = t; }
    }
    for (int i = 0; i < n; i++) { if (i > 0) printf(" "); printf("%lld", a[i]); }
    printf("\n");
    free(a);
    return 0;
}

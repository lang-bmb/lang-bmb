#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n, w; scanf("%d %d", &n, &w);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int row = 0;
    for (int i = 0; i < n; i += w, row++) {
        int end = i + w > n ? n : i + w;
        if (row % 2 == 0)
            for (int j = i; j < end; j++) { if (j > i) printf(" "); printf("%lld", a[j]); }
        else
            for (int j = end - 1; j >= i; j--) { if (j < end - 1) printf(" "); printf("%lld", a[j]); }
        printf("\n");
    }
    free(a);
    return 0;
}

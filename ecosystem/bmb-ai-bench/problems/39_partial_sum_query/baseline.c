#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long *prefix = calloc(n + 1, sizeof(long long));
    for (int i = 0; i < n; i++) prefix[i + 1] = prefix[i] + a[i];
    int q; scanf("%d", &q);
    for (int i = 0; i < q; i++) {
        int l, r; scanf("%d %d", &l, &r);
        printf("%lld\n", prefix[r + 1] - prefix[l]);
    }
    free(a); free(prefix);
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n, w; scanf("%d %d", &n, &w);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    for (int i = 0; i < n; i += w) {
        long long sum = 0;
        for (int j = 0; j < w && i + j < n; j++) sum += a[i + j];
        printf("%lld\n", sum);
    }
    free(a);
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n, k; scanf("%d %d", &n, &k); k %= n;
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    for (int i = 0; i < n; i++) {
        if (i > 0) printf(" ");
        printf("%lld", a[(i + k) % n]);
    }
    printf("\n");
    free(a);
    return 0;
}

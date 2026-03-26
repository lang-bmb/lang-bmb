#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    long long *b = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    for (int i = 0; i < n; i++) scanf("%lld", &b[i]);
    long long dist = 0;
    for (int i = 0; i < n; i++) { long long d = a[i] - b[i]; dist += d * d; }
    printf("%lld\n", dist);
    free(a); free(b);
    return 0;
}

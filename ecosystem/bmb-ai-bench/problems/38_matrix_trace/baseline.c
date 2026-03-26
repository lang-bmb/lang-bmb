#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *m = malloc(n * n * sizeof(long long));
    for (int i = 0; i < n * n; i++) scanf("%lld", &m[i]);
    long long trace = 0;
    for (int i = 0; i < n; i++) trace += m[i * n + i];
    printf("%lld\n", trace);
    free(m);
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    printf("%lld\n%lld\n%d\n", a[0], a[n-1], n);
    free(a); return 0;
}

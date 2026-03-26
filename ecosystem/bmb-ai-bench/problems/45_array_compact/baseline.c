#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int n; scanf("%d", &n);
    long long *a = malloc(n * sizeof(long long));
    for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int len = 0;
    for (int i = 0; i < n; i++) if (a[i] != 0) a[len++] = a[i];
    printf("%d\n", len);
    for (int i = 0; i < len; i++) { if (i > 0) printf(" "); printf("%lld", a[i]); }
    if (len > 0) printf("\n");
    free(a);
    return 0;
}

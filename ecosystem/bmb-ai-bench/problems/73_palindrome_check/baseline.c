#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int n; scanf("%d", &n);
        long long *a = malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
        int pal = 1;
        for (int i = 0; i < n/2; i++) if (a[i] != a[n-1-i]) { pal = 0; break; }
        printf("%d\n", pal);
        free(a);
    }
    return 0;
}

#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int n; scanf("%d", &n);
        long long *a = malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
        int ml = 1, cl = 1;
        for (int i = 1; i < n; i++) {
            if (a[i] == a[i-1]) { cl++; if (cl > ml) ml = cl; }
            else cl = 1;
        }
        printf("%d\n", ml);
        free(a);
    }
    return 0;
}

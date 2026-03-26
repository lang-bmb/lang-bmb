#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        int n; scanf("%d", &n);
        long long *a = malloc(n * sizeof(long long));
        for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
        long long cand = a[0]; int cnt = 1;
        for (int i = 1; i < n; i++) {
            if (a[i] == cand) cnt++;
            else if (--cnt == 0) { cand = a[i]; cnt = 1; }
        }
        int actual = 0;
        for (int i = 0; i < n; i++) if (a[i] == cand) actual++;
        printf("%lld\n", actual > n/2 ? cand : -1LL);
        free(a);
    }
    return 0;
}

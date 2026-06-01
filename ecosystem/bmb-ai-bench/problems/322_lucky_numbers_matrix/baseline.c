#include <stdio.h>
int main() {
    int m, n; scanf("%d %d", &m, &n);
    static long long a[10000];
    for (int i = 0; i < m * n; i++) scanf("%lld", &a[i]);
    long long ans = -1;
    for (int r = 0; r < m; r++) for (int c = 0; c < n; c++) {
        long long v = a[r*n+c]; int ok = 1;
        for (int cc = 0; cc < n; cc++) if (a[r*n+cc] < v) ok = 0;
        for (int rr = 0; rr < m; rr++) if (a[rr*n+c] > v) ok = 0;
        if (ok) ans = v;
    }
    printf("%lld\n", ans);
    return 0;
}

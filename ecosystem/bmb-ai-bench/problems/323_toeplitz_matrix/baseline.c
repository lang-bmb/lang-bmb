#include <stdio.h>
int main() {
    int m, n; scanf("%d %d", &m, &n);
    static long long a[10000];
    for (int i = 0; i < m * n; i++) scanf("%lld", &a[i]);
    int ok = 1;
    for (int r = 1; r < m; r++) for (int c = 1; c < n; c++)
        if (a[r*n+c] != a[(r-1)*n+(c-1)]) ok = 0;
    printf("%d\n", ok);
    return 0;
}

#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    static long long a[10000];
    for (int i = 0; i < n * n; i++) scanf("%lld", &a[i]);
    int ok = 1;
    for (int r = 0; r < n; r++) for (int c = 0; c < n; c++)
        if (a[r*n+c] != a[c*n+r]) ok = 0;
    printf("%d\n", ok);
    return 0;
}

#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long s = 0, x;
    for (int i = 0; i < n; i++) { scanf("%lld", &x); s += x; }
    long long total = (long long)n * (n + 1) / 2;
    printf("%lld\n", total - s);
    return 0;
}

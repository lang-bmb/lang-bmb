#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long best = -1, x, y;
    for (int i = 0; i < n; i++) {
        scanf("%lld %lld", &x, &y);
        long long d = x*x + y*y;
        if (best < 0 || d < best) best = d;
    }
    printf("%lld\n", best);
    return 0;
}

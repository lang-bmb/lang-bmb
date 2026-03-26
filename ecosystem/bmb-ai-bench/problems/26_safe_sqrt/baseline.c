#include <stdio.h>

long long isqrt(long long x) {
    if (x <= 1) return x;
    long long lo = 1, hi = 1000000000LL;
    if (hi > x) hi = x;
    while (lo <= hi) {
        long long mid = lo + (hi - lo) / 2;
        if (mid <= x / mid) lo = mid + 1;
        else hi = mid - 1;
    }
    return hi;
}

int main(void) {
    int n;
    scanf("%d", &n);
    for (int i = 0; i < n; i++) {
        long long x;
        scanf("%lld", &x);
        printf("%lld\n", isqrt(x));
    }
    return 0;
}

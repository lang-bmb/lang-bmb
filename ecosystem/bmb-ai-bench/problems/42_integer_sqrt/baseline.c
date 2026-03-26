#include <stdio.h>
long long isqrt(long long n) {
    if (n <= 0) return 0;
    long long lo = 1, hi = n > 3037000499LL ? 3037000499LL : n, result = 0;
    while (lo <= hi) {
        long long mid = lo + (hi - lo) / 2;
        if (mid <= n / mid) { result = mid; lo = mid + 1; }
        else hi = mid - 1;
    }
    return result;
}
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long n; scanf("%lld", &n);
        printf("%lld\n", isqrt(n));
    }
    return 0;
}

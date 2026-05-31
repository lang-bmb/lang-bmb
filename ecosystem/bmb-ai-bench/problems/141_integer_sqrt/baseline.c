#include <stdio.h>
int main() {
    long long n;
    scanf("%lld", &n);
    long long lo = 0, hi = n, ans = 0;
    while (lo <= hi) {
        long long mid = lo + (hi - lo) / 2;
        if (mid * mid <= n) { ans = mid; lo = mid + 1; }
        else { hi = mid - 1; }
    }
    printf("%lld\n", ans);
    return 0;
}

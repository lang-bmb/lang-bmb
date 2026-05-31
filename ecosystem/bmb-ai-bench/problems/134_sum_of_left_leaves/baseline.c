#include <stdio.h>

int main() {
    long long n;
    scanf("%lld", &n);
    long long lo = 0, hi = n, result = 0;
    while (lo <= hi) {
        long long mid = lo + (hi - lo) / 2;
        long long s = mid * (mid + 1) / 2;
        if (s <= n) { result = mid; lo = mid + 1; }
        else hi = mid - 1;
    }
    printf("%lld\n", result);
    return 0;
}

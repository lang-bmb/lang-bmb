#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long lo = 1, hi = n;
    while (lo <= hi) {
        long long mid = lo + (hi - lo) / 2;
        if (mid * mid == n) { printf("1\n"); return 0; }
        else if (mid * mid < n) lo = mid + 1; else hi = mid - 1;
    }
    printf("0\n"); return 0;
}

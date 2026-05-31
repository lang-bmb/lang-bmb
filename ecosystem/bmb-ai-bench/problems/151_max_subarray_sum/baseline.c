#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long first; scanf("%lld", &first);
    long long cur = first, best = first;
    for (long long i = 1; i < n; i++) {
        long long x; scanf("%lld", &x);
        cur = cur + x > x ? cur + x : x;
        best = cur > best ? cur : best;
    }
    printf("%lld\n", best);
    return 0;
}

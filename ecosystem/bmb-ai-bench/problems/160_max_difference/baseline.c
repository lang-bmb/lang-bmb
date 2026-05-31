#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long first; scanf("%lld", &first);
    long long min_val = first, best = -1000000000LL;
    for (long long i = 1; i < n; i++) {
        long long x; scanf("%lld", &x);
        long long diff = x - min_val;
        if (diff > best) best = diff;
        if (x < min_val) min_val = x;
    }
    printf("%lld\n", best);
    return 0;
}

#include <stdio.h>
long long gcd(long long a, long long b) { return b == 0 ? a : gcd(b, a % b); }
int main() {
    long long n; scanf("%lld", &n);
    long long first; scanf("%lld", &first);
    long long g = first;
    for (long long i = 1; i < n; i++) {
        long long x; scanf("%lld", &x);
        g = gcd(g, x);
    }
    printf("%lld\n", g);
    return 0;
}

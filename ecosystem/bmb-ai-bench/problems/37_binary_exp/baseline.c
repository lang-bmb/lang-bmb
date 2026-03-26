#include <stdio.h>
long long power(long long a, long long b) {
    long long r = 1;
    while (b > 0) { if (b & 1) r *= a; b >>= 1; a *= a; }
    return r;
}
int main(void) {
    int n; scanf("%d", &n);
    for (int i = 0; i < n; i++) {
        long long a, b; scanf("%lld %lld", &a, &b);
        printf("%lld\n", power(a, b));
    }
    return 0;
}

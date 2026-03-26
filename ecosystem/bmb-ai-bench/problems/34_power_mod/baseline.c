#include <stdio.h>
long long power_mod(long long a, long long b, long long m) {
    long long result = 1; a %= m;
    while (b > 0) {
        if (b & 1) result = result * a % m;
        b >>= 1; a = a * a % m;
    }
    return result;
}
int main(void) {
    int n; scanf("%d", &n);
    for (int i = 0; i < n; i++) {
        long long a, b, m; scanf("%lld %lld %lld", &a, &b, &m);
        printf("%lld\n", power_mod(a, b, m));
    }
    return 0;
}

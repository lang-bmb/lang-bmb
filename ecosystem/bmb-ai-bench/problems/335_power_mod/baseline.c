#include <stdio.h>
int main() {
    long long base, exp, m; scanf("%lld %lld %lld", &base, &exp, &m);
    long long result = 1, b = base % m, e = exp;
    while (e > 0) {
        if (e & 1) result = (result * b) % m;
        b = (b * b) % m;
        e >>= 1;
    }
    printf("%lld\n", result % m);
    return 0;
}

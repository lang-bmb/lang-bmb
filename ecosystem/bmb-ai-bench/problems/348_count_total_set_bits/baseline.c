#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long acc = 0;
    for (long long i = 0; i <= n; i++) { long long x = i; while (x) { acc += x & 1; x >>= 1; } }
    printf("%lld\n", acc);
    return 0;
}

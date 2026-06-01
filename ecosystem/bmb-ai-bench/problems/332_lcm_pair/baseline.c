#include <stdio.h>
int main() {
    long long a, b; scanf("%lld %lld", &a, &b);
    long long x = a, y = b;
    while (y != 0) { long long t = y; y = x % y; x = t; }
    printf("%lld\n", a / x * b);
    return 0;
}

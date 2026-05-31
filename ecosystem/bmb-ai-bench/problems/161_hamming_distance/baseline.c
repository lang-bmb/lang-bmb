#include <stdio.h>
int main() {
    long long a, b;
    scanf("%lld", &a);
    scanf("%lld", &b);
    long long x = a ^ b;
    long long count = 0;
    while (x) { count += x & 1; x >>= 1; }
    printf("%lld\n", count);
    return 0;
}

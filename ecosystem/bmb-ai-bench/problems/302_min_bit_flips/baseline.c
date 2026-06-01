#include <stdio.h>
int main() {
    long long s, g; scanf("%lld %lld", &s, &g);
    long long x = s ^ g; int c = 0;
    while (x) { c += (int)(x & 1); x >>= 1; }
    printf("%d\n", c);
    return 0;
}

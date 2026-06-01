#include <stdio.h>
int main() {
    long long x; scanf("%lld", &x);
    long long steps = 0;
    while (x != 1) { if (x % 2 == 0) x /= 2; else x = 3 * x + 1; steps++; }
    printf("%lld\n", steps);
    return 0;
}

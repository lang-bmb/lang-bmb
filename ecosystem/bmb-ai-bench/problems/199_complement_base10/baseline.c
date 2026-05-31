#include <stdio.h>
int main() {
    long long n;
    scanf("%lld", &n);
    if (n == 0) { printf("1\n"); return 0; }
    long long mask = 1;
    while (mask <= n) mask <<= 1;
    printf("%lld\n", (mask - 1) ^ n);
    return 0;
}

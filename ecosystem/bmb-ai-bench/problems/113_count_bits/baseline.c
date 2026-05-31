#include <stdio.h>
int popcount(long long x) {
    int c = 0;
    while (x) { x &= x - 1; c++; }
    return c;
}
int main() {
    long long n;
    scanf("%lld", &n);
    for (long long i = 0; i <= n; i++) printf("%d\n", popcount(i));
    return 0;
}

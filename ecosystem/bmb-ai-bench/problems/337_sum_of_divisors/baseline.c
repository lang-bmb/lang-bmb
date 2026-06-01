#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long s = 0;
    for (long long i = 1; i <= n; i++) if (n % i == 0) s += i;
    printf("%lld\n", s);
    return 0;
}

#include <stdio.h>
int main() {
    long long n, k; scanf("%lld %lld", &n, &k);
    long long r = 0;
    for (long long m = 2; m <= n; m++) r = (r + k) % m;
    printf("%lld\n", r);
    return 0;
}

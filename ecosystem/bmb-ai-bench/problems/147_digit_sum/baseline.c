#include <stdio.h>
int digit_sum(long long n) {
    long long s = 0;
    while (n > 0) { s += n % 10; n /= 10; }
    return s;
}
int main() {
    long long n; int k;
    scanf("%lld %d", &n, &k);
    while (k-- > 1) n = digit_sum(n);
    printf("%lld\n", n);
    return 0;
}

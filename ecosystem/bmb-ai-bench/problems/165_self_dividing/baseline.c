#include <stdio.h>
int is_sd(long long n) {
    long long orig = n;
    while (n) { long long d = n % 10; if (!d || orig % d) return 0; n /= 10; }
    return 1;
}
int main() {
    long long l, r;
    scanf("%lld", &l);
    scanf("%lld", &r);
    long long cnt = 0;
    for (long long i = l; i <= r; i++) if (is_sd(i)) cnt++;
    printf("%lld\n", cnt);
    return 0;
}

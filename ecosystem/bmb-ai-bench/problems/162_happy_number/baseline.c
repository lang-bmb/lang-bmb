#include <stdio.h>
long long dss(long long n) {
    long long s = 0;
    while (n) { long long d = n % 10; s += d*d; n /= 10; }
    return s;
}
int main() {
    long long n; scanf("%lld", &n);
    long long slow = n, fast = n;
    do { slow = dss(slow); fast = dss(dss(fast)); } while (slow != fast);
    printf("%lld\n", slow == 1 ? 1LL : 0LL);
    return 0;
}

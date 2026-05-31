#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long sum = 0, sign = 1;
    for (long long i = 0; i < n; i++) {
        long long x; scanf("%lld", &x);
        sum += sign * x;
        sign = -sign;
    }
    printf("%lld\n", sum);
    return 0;
}

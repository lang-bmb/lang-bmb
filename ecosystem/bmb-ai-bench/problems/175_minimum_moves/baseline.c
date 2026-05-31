#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long sum = 0, min_val = 2000000000LL;
    for (long long i = 0; i < n; i++) {
        long long x; scanf("%lld", &x);
        sum += x;
        if (x < min_val) min_val = x;
    }
    printf("%lld\n", sum - n * min_val);
    return 0;
}

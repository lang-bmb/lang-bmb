#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    if (n <= 1) { printf("0\n"); return 0; }
    long long sum = 1;
    for (long long d = 2; d * d <= n; d++) {
        if (n % d == 0) { sum += d; if (d != n/d) sum += n/d; }
    }
    printf("%lld\n", sum == n ? 1LL : 0LL);
    return 0;
}

#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long cnt = 0;
    for (long long i = 1; i * i <= n; i++) {
        if (n % i == 0) cnt += (i * i == n) ? 1 : 2;
    }
    printf("%lld\n", cnt);
    return 0;
}

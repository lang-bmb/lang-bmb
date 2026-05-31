#include <stdio.h>
int main() {
    long long n, m; scanf("%lld %lld", &n, &m);
    long long a = 0, b = 1;
    for (long long i = 0; i < n; i++) {
        long long c = (a + b) % m; a = b; b = c;
    }
    printf("%lld\n", a % m);
    return 0;
}

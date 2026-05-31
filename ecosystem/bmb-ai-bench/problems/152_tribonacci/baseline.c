#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long a = 0, b = 0, c = 1;
    if (n == 0) { printf("0\n"); return 0; }
    if (n == 1) { printf("0\n"); return 0; }
    if (n == 2) { printf("1\n"); return 0; }
    for (long long i = 3; i <= n; i++) {
        long long d = a + b + c; a = b; b = c; c = d;
    }
    printf("%lld\n", c);
    return 0;
}

#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    if (n <= 0) { printf("0\n"); return 0; }
    while (n % 3 == 0) n /= 3;
    printf("%lld\n", n == 1 ? 1LL : 0LL);
    return 0;
}

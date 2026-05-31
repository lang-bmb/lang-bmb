#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    if (n <= 0) { printf("0\n"); return 0; }
    while (n % 2 == 0) n /= 2;
    while (n % 3 == 0) n /= 3;
    while (n % 5 == 0) n /= 5;
    printf("%lld\n", n == 1 ? 1LL : 0LL);
    return 0;
}

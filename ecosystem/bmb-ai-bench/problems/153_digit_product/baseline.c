#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    if (n == 0) { printf("0\n"); return 0; }
    long long prod = 1;
    while (n > 0) { prod *= n % 10; n /= 10; }
    printf("%lld\n", prod);
    return 0;
}

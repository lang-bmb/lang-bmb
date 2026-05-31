#include <stdio.h>
int is_pow4(long long n) {
    if (n <= 0) return 0;
    if (n == 1) return 1;
    if (n % 4 != 0) return 0;
    return is_pow4(n / 4);
}
int main() {
    long long n; scanf("%lld", &n);
    printf("%d\n", is_pow4(n));
    return 0;
}

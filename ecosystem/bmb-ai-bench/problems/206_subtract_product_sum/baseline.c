#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    long long prod = 1, sum = 0;
    while (n > 0) {
        int d = n % 10;
        prod *= d;
        sum += d;
        n /= 10;
    }
    printf("%lld\n", prod - sum);
    return 0;
}

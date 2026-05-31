#include <stdio.h>

int main() {
    long long n;
    scanf("%lld", &n);
    long long count = 0;
    while (n >= 5) { n /= 5; count += n; }
    printf("%lld\n", count);
    return 0;
}

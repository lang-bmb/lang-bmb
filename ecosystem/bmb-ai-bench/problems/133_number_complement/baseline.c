#include <stdio.h>

int main() {
    long long n;
    scanf("%lld", &n);
    long long mask = 1;
    while (mask < n) mask = mask * 2 + 1;
    printf("%lld\n", n ^ mask);
    return 0;
}

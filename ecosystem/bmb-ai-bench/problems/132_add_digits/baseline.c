#include <stdio.h>

int main() {
    long long n;
    scanf("%lld", &n);
    if (n == 0) { printf("0\n"); return 0; }
    printf("%lld\n", 1 + (n - 1) % 9);
    return 0;
}

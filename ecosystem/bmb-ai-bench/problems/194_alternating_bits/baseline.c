#include <stdio.h>
int main() {
    long long n;
    scanf("%lld", &n);
    long long prev = n & 1;
    n >>= 1;
    while (n > 0) {
        long long cur = n & 1;
        if (cur == prev) { printf("0\n"); return 0; }
        prev = cur;
        n >>= 1;
    }
    printf("1\n");
    return 0;
}

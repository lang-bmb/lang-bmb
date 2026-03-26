#include <stdio.h>
#include <stdlib.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long num, base; scanf("%lld %lld", &num, &base);
        if (num == 0) { printf("0\n"); continue; }
        int digits[128], nd = 0, neg = 0;
        long long n = num;
        if (n < 0) { neg = 1; n = -n; }
        while (n > 0) { digits[nd++] = n % base; n /= base; }
        if (neg) printf("-");
        for (int i = nd - 1; i >= 0; i--) printf("%d", digits[i]);
        printf("\n");
    }
    return 0;
}

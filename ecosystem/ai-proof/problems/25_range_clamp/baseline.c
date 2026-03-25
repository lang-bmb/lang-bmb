#include <stdio.h>

long long clamp(long long val, long long lo, long long hi) {
    if (val < lo) return lo;
    if (val > hi) return hi;
    return val;
}

int main(void) {
    long long lo, hi;
    int n;
    scanf("%lld %lld %d", &lo, &hi, &n);
    for (int i = 0; i < n; i++) {
        long long x;
        scanf("%lld", &x);
        if (i > 0) printf(" ");
        printf("%lld", clamp(x, lo, hi));
    }
    printf("\n");
    return 0;
}

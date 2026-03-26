#include <stdio.h>

long long normalize(long long x, long long min, long long max) {
    return (x - min) * 100 / (max - min);
}

long long scale(long long x, long long factor) {
    return x * factor;
}

long long bound(long long x, long long limit) {
    return x > limit ? limit : x;
}

int main(void) {
    long long min, max, factor, limit;
    int n;
    scanf("%lld %lld %lld %lld %d", &min, &max, &factor, &limit, &n);
    for (int i = 0; i < n; i++) {
        long long x;
        scanf("%lld", &x);
        long long result = bound(scale(normalize(x, min, max), factor), limit);
        printf("%lld\n", result);
    }
    return 0;
}

#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long n, lo, hi; scanf("%lld %lld %lld", &n, &lo, &hi);
        if (n < lo) printf("%lld\n", lo);
        else if (n > hi) printf("%lld\n", hi);
        else printf("%lld\n", n);
    }
    return 0;
}

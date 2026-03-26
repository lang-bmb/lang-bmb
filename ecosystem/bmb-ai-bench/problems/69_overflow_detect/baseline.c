#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long a, b; scanf("%lld %lld", &a, &b);
        long long r = a * b;
        int ov = (r > 2147483647LL || r < -2147483648LL) ? 1 : 0;
        printf("%d %lld\n", ov, r);
    }
    return 0;
}

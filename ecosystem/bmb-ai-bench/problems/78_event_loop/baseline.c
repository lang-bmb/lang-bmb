#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long total = 0;
    for (int i = 0; i < n; i++) {
        long long t, v; scanf("%lld %lld", &t, &v);
        total += v;
        printf("%lld\n", total);
    }
    return 0;
}

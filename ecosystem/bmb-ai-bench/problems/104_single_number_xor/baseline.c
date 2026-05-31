#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long result = 0;
    for (int i = 0; i < n; i++) { long long x; scanf("%lld", &x); result ^= x; }
    printf("%lld\n", result);
    return 0;
}

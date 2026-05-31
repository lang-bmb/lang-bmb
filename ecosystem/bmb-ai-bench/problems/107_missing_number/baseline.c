#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long expected = (long long)n*(n+1)/2, actual = 0;
    for (int i = 0; i < n; i++) { long long x; scanf("%lld", &x); actual += x; }
    printf("%lld\n", expected - actual);
    return 0;
}

#include <stdio.h>
int classify(long long n) {
    if (n < 0) return -1;
    if (n < 10) return 0;
    if (n < 100) return 1;
    if (n < 1000) return 2;
    if (n < 10000) return 3;
    if (n < 100000) return 4;
    if (n < 1000000) return 5;
    return 6;
}
int main(void) {
    int t; scanf("%d", &t);
    while (t--) { long long n; scanf("%lld", &n); printf("%d\n", classify(n)); }
    return 0;
}

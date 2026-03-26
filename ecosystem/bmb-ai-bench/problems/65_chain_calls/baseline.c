#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) { long long x; scanf("%lld", &x); printf("%lld\n", x*x*2+1); }
    return 0;
}

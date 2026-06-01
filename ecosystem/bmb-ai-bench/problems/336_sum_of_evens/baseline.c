#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long s = 0, x;
    for (int i = 0; i < n; i++) { scanf("%lld", &x); if (x % 2 == 0) s += x; }
    printf("%lld\n", s);
    return 0;
}

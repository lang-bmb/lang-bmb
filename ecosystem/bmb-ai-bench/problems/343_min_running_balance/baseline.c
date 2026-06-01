#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long bal = 0, mn = 0, x;
    for (int i = 0; i < n; i++) { scanf("%lld", &x); bal += x; if (bal < mn) mn = bal; }
    printf("%lld\n", mn);
    return 0;
}

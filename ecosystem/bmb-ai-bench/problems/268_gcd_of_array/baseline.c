#include <stdio.h>
int gcd(int a, int b) { return b == 0 ? a : gcd(b, a % b); }
int main() {
    int n; scanf("%d", &n);
    int mn, mx, v; scanf("%d", &v); mn = mx = v;
    for (int i = 1; i < n; i++) {
        scanf("%d", &v);
        if (v < mn) mn = v;
        if (v > mx) mx = v;
    }
    printf("%d\n", gcd(mn, mx));
    return 0;
}

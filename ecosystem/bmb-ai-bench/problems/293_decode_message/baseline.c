#include <stdio.h>
int main() {
    int a, b, c; scanf("%d\n%d\n%d", &a, &b, &c);
    int mx = a > b ? a : b;
    if (c > mx) mx = c;
    int total = a + b + c;
    int ceil_half = (total + 1) / 2;
    int ans = mx > ceil_half ? mx : ceil_half;
    printf("%d\n", ans);
    return 0;
}

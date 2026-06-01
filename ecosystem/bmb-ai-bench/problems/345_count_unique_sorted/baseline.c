#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long x0; scanf("%lld", &x0);
    long long last = x0, x, acc = 1;
    for (int i = 1; i < n; i++) { scanf("%lld", &x); if (x != last) { acc++; last = x; } }
    printf("%lld\n", acc);
    return 0;
}

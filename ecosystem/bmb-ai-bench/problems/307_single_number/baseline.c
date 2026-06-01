#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long acc = 0, x;
    for (int i = 0; i < n; i++) { scanf("%lld", &x); acc ^= x; }
    printf("%lld\n", acc);
    return 0;
}

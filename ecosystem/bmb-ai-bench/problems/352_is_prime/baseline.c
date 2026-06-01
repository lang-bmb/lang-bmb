#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    int prime = n < 2 ? 0 : 1;
    for (long long d = 2; d * d <= n; d++) if (n % d == 0) { prime = 0; break; }
    printf("%d\n", prime);
    return 0;
}

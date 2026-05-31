#include <stdio.h>
#define MAX 1001
int main() {
    int n;
    scanf("%d", &n);
    long long cost[MAX];
    for (int i = 0; i < n; i++) scanf("%lld", &cost[i]);
    if (n == 1) { printf("%lld\n", cost[0]); return 0; }
    long long p2 = cost[0], p1 = cost[1];
    for (int i = 2; i < n; i++) {
        long long cur = cost[i] + (p1 < p2 ? p1 : p2);
        p2 = p1; p1 = cur;
    }
    printf("%lld\n", p1 < p2 ? p1 : p2);
    return 0;
}

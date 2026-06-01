#include <stdio.h>
static int is_prime(long long x){ if(x<2) return 0; for(long long d=2; d*d<=x; d++) if(x%d==0) return 0; return 1; }
int main() {
    int n; scanf("%d", &n);
    long long cand = 1, count = 0;
    while (count < n) { cand++; if (is_prime(cand)) count++; }
    printf("%lld\n", cand);
    return 0;
}

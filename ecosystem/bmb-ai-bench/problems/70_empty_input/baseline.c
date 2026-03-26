#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long sum = 0;
    for (int i = 0; i < n; i++) { long long v; scanf("%lld", &v); sum += v; }
    printf("%lld\n%d\n", sum, n);
    return 0;
}

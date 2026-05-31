#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long sum = 0;
    for (int i = 0; i < n; i++) {
        long long x; scanf("%lld", &x);
        sum += x;
        printf("%lld\n", sum);
    }
    return 0;
}

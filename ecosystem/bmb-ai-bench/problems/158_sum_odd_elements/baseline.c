#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long sum = 0;
    for (long long i = 0; i < n; i++) {
        long long x; scanf("%lld", &x);
        if (x % 2 != 0) sum += x;
    }
    printf("%lld\n", sum);
    return 0;
}

#include <stdio.h>
#include <math.h>
int main() {
    long long n; scanf("%lld", &n);
    if (n == 2) { printf("1\n"); return 0; }
    if (n == 3) { printf("2\n"); return 0; }
    if (n == 4) { printf("4\n"); return 0; }
    long long rem = n % 3;
    long long p = 1;
    long long k = n / 3;
    if (rem == 1) { k = (n - 4) / 3; rem = 4; } else if (rem == 2) { k = (n - 2) / 3; rem = 2; } else { rem = 1; }
    for (long long i = 0; i < k; i++) p *= 3;
    printf("%lld\n", p * rem);
    return 0;
}

#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long acc = 0;
    for (long long i = 0; i < n; i++) if (i % 3 == 0 || i % 5 == 0) acc += i;
    printf("%lld\n", acc);
    return 0;
}

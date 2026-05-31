#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    printf("%d\n", n % 4 != 0 ? 1 : 0);
    return 0;
}

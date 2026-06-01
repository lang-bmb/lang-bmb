#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    int r = (n > 0 && (n & (n - 1)) == 0) ? 1 : 0;
    printf("%d\n", r);
    return 0;
}

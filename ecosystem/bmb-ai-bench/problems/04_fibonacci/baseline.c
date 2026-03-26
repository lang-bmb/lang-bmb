#include <stdio.h>

int main(void) {
    int n;
    scanf("%d", &n);
    if (n == 0) { printf("0\n"); return 0; }
    if (n == 1) { printf("1\n"); return 0; }
    long long a = 0, b = 1;
    for (int i = 2; i <= n; i++) {
        long long c = a + b;
        a = b;
        b = c;
    }
    printf("%lld\n", b);
    return 0;
}

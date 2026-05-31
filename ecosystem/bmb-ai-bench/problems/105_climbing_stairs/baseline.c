#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    if (n == 1) { printf("1\n"); return 0; }
    if (n == 2) { printf("2\n"); return 0; }
    long long a = 1, b = 2;
    for (int i = 2; i < n; i++) { long long c = a + b; a = b; b = c; }
    printf("%lld\n", b);
    return 0;
}

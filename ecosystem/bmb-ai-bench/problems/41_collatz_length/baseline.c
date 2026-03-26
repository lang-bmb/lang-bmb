#include <stdio.h>
int main(void) {
    int t; scanf("%d", &t);
    while (t--) {
        long long n; scanf("%lld", &n);
        int count = 1;
        while (n != 1) { n = (n % 2 == 0) ? n / 2 : 3 * n + 1; count++; }
        printf("%d\n", count);
    }
    return 0;
}

#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    if (n == 0) { printf("0\n"); return 0; }
    char buf[64]; int p = 0;
    while (n > 0) { buf[p++] = (char)('0' + n % 2); n /= 2; }
    for (int i = p - 1; i >= 0; i--) putchar(buf[i]);
    putchar('\n');
    return 0;
}

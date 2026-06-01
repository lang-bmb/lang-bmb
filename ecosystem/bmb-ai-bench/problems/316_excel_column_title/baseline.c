#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    char buf[64]; int p = 0;
    while (n > 0) { int m = (int)((n - 1) % 26); buf[p++] = (char)('A' + m); n = (n - 1) / 26; }
    for (int i = p - 1; i >= 0; i--) putchar(buf[i]);
    putchar('\n');
    return 0;
}

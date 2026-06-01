#include <stdio.h>
int main() {
    long long n; scanf("%lld", &n);
    long long x = n < 0 ? -n : n;
    int freq[10] = {0}, dup = 0;
    if (x == 0) { printf("0\n"); return 0; }
    while (x > 0) { int d = (int)(x % 10); if (freq[d] > 0) dup = 1; freq[d]++; x /= 10; }
    printf("%d\n", dup);
    return 0;
}

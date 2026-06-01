#include <stdio.h>
static long long ipow(long long b, long long e){ long long r=1; while(e-->0) r*=b; return r; }
int main() {
    long long n; scanf("%lld", &n);
    long long t = n, nd = 0;
    if (n == 0) nd = 1; else while (t > 0) { nd++; t /= 10; }
    long long s = 0; t = n;
    while (t > 0) { s += ipow(t % 10, nd); t /= 10; }
    printf("%d\n", s == n ? 1 : 0);
    return 0;
}

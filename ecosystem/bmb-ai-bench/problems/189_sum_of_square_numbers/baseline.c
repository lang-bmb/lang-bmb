#include <stdio.h>
#include <math.h>
int main() {
    long long c;
    scanf("%lld", &c);
    long long lo = 0, hi = (long long)sqrt((double)c);
    while (lo <= hi) {
        long long s = lo*lo + hi*hi;
        if (s == c) { printf("1\n"); return 0; }
        else if (s < c) lo++;
        else hi--;
    }
    printf("0\n");
    return 0;
}

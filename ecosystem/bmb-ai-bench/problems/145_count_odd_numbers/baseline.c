#include <stdio.h>
int main() {
    long long low, high;
    scanf("%lld %lld", &low, &high);
    printf("%lld\n", (high + 1) / 2 - low / 2);
    return 0;
}

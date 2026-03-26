#include <stdio.h>
int main(void) {
    int n; scanf("%d", &n);
    long long sum = 0;
    for (int i = 0; i < n; i++) { long long v; scanf("%lld", &v); sum += v; }
    long long result = sum % 256;
    if (result < 0) result += 256;
    printf("%lld\n", result);
    return 0;
}

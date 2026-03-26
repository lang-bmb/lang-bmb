#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int n;
    scanf("%d", &n);
    long long sum = 0;
    for (int i = 0; i < n; i++) {
        long long x;
        scanf("%lld", &x);
        sum += x;
    }
    printf("%lld\n", sum);
    return 0;
}

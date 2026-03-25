#include <stdio.h>
#include <stdlib.h>

int main(void) {
    long long target;
    int n;
    scanf("%lld %d", &target, &n);
    int count = 0;
    for (int i = 0; i < n; i++) {
        long long x;
        scanf("%lld", &x);
        if (x == target) count++;
    }
    printf("%d\n", count);
    return 0;
}

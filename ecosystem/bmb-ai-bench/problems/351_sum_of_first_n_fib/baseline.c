#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long prev = 0, cur = 1, acc = 0;
    for (int i = 0; i < n; i++) { acc += cur; long long nx = prev + cur; prev = cur; cur = nx; }
    printf("%lld\n", acc);
    return 0;
}

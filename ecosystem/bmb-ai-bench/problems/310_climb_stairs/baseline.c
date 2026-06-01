#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long prev = 1, cur = 1;
    for (int i = 1; i < n; i++) { long long nx = prev + cur; prev = cur; cur = nx; }
    printf("%lld\n", cur);
    return 0;
}

#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long a[1000]; for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long ops = 0, prev = a[0];
    for (int i = 1; i < n; i++) {
        if (a[i] <= prev) { ops += prev + 1 - a[i]; prev = prev + 1; }
        else prev = a[i];
    }
    printf("%lld\n", ops);
    return 0;
}

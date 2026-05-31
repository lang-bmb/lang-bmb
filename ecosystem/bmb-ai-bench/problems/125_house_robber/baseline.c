#include <stdio.h>

int main() {
    int n;
    scanf("%d", &n);
    long long prev2 = 0, prev1 = 0;
    for (int i = 0; i < n; i++) {
        long long x;
        scanf("%lld", &x);
        long long with_cur = prev2 + x;
        long long new_val = with_cur > prev1 ? with_cur : prev1;
        prev2 = prev1;
        prev1 = new_val;
    }
    printf("%lld\n", prev1);
    return 0;
}

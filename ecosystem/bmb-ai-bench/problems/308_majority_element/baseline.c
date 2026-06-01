#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    long long cand = 0, cnt = 0, x;
    for (int i = 0; i < n; i++) {
        scanf("%lld", &x);
        if (cnt == 0) { cand = x; cnt = 1; }
        else if (x == cand) cnt++;
        else cnt--;
    }
    printf("%lld\n", cand);
    return 0;
}

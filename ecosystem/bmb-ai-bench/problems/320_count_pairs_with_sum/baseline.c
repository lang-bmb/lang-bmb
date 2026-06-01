#include <stdio.h>
int main() {
    int n; long long target; scanf("%d %lld", &n, &target);
    long long a[2000]; for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    long long cnt = 0;
    for (int i = 0; i < n; i++) for (int j = i + 1; j < n; j++) if (a[i] + a[j] == target) cnt++;
    printf("%lld\n", cnt);
    return 0;
}

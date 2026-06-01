#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int a[1000]; for (int i = 0; i < n; i++) scanf("%d", &a[i]);
    int lo = a[0], hi = a[0];
    for (int i = 1; i < n; i++) { if (a[i] < lo) lo = a[i]; if (a[i] > hi) hi = a[i]; }
    int cnt = 0;
    for (int i = 0; i < n; i++) if (a[i] > lo && a[i] < hi) cnt++;
    printf("%d\n", cnt);
    return 0;
}

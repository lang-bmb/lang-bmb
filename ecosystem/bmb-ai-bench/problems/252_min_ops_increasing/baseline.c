#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int a[1001];
    for (int i = 0; i < n; i++) scanf("%d", &a[i]);
    int ops = 0;
    for (int i = 1; i < n; i++) {
        if (a[i] <= a[i-1]) {
            ops += a[i-1] - a[i] + 1;
            a[i] = a[i-1] + 1;
        }
    }
    printf("%d\n", ops);
    return 0;
}

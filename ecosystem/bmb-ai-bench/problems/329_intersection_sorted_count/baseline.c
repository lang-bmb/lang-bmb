#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    static long long a[100000]; for (int i = 0; i < n; i++) scanf("%lld", &a[i]);
    int m; scanf("%d", &m);
    static long long b[100000]; for (int j = 0; j < m; j++) scanf("%lld", &b[j]);
    int i = 0, j = 0; long long last = -1000000000, acc = 0;
    while (i < n && j < m) {
        if (a[i] < b[j]) i++;
        else if (a[i] > b[j]) j++;
        else { if (a[i] != last) { acc++; last = a[i]; } i++; j++; }
    }
    printf("%lld\n", acc);
    return 0;
}

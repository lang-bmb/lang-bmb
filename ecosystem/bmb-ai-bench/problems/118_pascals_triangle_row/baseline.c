#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    long long row[1001];
    for (int i = 0; i <= n; i++) row[i] = 0;
    row[0] = 1;
    for (int step = 1; step <= n; step++) {
        for (int i = step; i >= 1; i--) row[i] += row[i-1];
    }
    for (int i = 0; i <= n; i++) printf("%lld\n", row[i]);
    return 0;
}

#include <stdio.h>
int main() {
    int m, n;
    scanf("%d%d", &m, &n);
    int arr[10000];
    for (int i = 0; i < m*n; i++) scanf("%d", &arr[i]);
    int best = 0;
    for (int i = 0; i < m; i++) {
        int s = 0;
        for (int j = 0; j < n; j++) s += arr[i*n+j];
        if (s > best) best = s;
    }
    printf("%d\n", best);
    return 0;
}

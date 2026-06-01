#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int freq[501] = {0}, x;
    for (int i = 0; i < n; i++) { scanf("%d", &x); freq[x]++; }
    int ans = -1;
    for (int v = 1; v <= 500; v++) if (freq[v] == v && v > ans) ans = v;
    printf("%d\n", ans);
    return 0;
}

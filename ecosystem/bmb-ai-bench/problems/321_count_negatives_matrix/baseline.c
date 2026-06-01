#include <stdio.h>
int main() {
    int m, n; scanf("%d %d", &m, &n);
    int cnt = 0, x;
    for (int i = 0; i < m * n; i++) { scanf("%d", &x); if (x < 0) cnt++; }
    printf("%d\n", cnt);
    return 0;
}

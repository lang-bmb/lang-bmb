#include <stdio.h>
int main() {
    int m, n; scanf("%d %d", &m, &n);
    int cnt = 0;
    for (int r = 0; r < m; r++) {
        int ok = 1, x;
        for (int c = 0; c < n; c++) { scanf("%d", &x); if (x <= 0) ok = 0; }
        cnt += ok;
    }
    printf("%d\n", cnt);
    return 0;
}

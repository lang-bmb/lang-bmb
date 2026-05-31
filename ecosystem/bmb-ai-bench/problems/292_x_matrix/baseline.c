#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int grid[100][100];
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++) scanf("%d", &grid[i][j]);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            int on_diag = (i == j) || (i + j == n - 1);
            if (on_diag && grid[i][j] == 0) { printf("0\n"); return 0; }
            if (!on_diag && grid[i][j] != 0) { printf("0\n"); return 0; }
        }
    }
    printf("1\n");
    return 0;
}

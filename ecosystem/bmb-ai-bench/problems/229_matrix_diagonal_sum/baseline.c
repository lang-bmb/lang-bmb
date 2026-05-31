#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int mat[100][100];
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++)
            scanf("%d", &mat[i][j]);
    int sum = 0;
    for (int i = 0; i < n; i++) {
        sum += mat[i][i];
        sum += mat[i][n-1-i];
    }
    if (n % 2 == 1) sum -= mat[n/2][n/2];
    printf("%d\n", sum);
    return 0;
}

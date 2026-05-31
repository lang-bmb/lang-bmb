#include <stdio.h>
#include <string.h>
int main() {
    int n;
    scanf("%d", &n);
    char strs[100][1001];
    for (int i = 0; i < n; i++) scanf("%s", strs[i]);
    if (n == 0) { printf("\n"); return 0; }
    int plen = strlen(strs[0]);
    for (int i = 1; i < n; i++) {
        int j = 0;
        while (j < plen && strs[i][j] == strs[0][j]) j++;
        plen = j;
    }
    for (int i = 0; i < plen; i++) putchar(strs[0][i]);
    printf("\n");
    return 0;
}

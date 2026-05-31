#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int freq[501] = {0};
    for (int i = 0; i < n; i++) {
        int v; scanf("%d", &v);
        freq[v]++;
    }
    for (int i = 0; i <= 500; i++) {
        if (freq[i] % 2 != 0) { printf("0\n"); return 0; }
    }
    printf("1\n");
    return 0;
}

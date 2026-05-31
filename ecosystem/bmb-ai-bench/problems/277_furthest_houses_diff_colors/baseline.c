#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int colors[100];
    for (int i = 0; i < n; i++) scanf("%d", &colors[i]);
    int ans = 0;
    for (int i = 0; i < n; i++) {
        for (int j = i+1; j < n; j++) {
            if (colors[i] != colors[j]) {
                int d = j - i;
                if (d > ans) ans = d;
            }
        }
    }
    printf("%d\n", ans);
    return 0;
}

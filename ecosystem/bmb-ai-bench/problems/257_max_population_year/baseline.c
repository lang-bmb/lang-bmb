#include <stdio.h>
int main() {
    int n; scanf("%d", &n);
    int births[100], deaths[100];
    for (int i = 0; i < n; i++) {
        scanf("%d", &births[i]);
        scanf("%d", &deaths[i]);
    }
    int best = 1950, best_c = 0;
    for (int y = 1950; y <= 2050; y++) {
        int c = 0;
        for (int i = 0; i < n; i++) {
            if (births[i] <= y && y < deaths[i]) c++;
        }
        if (c > best_c) { best_c = c; best = y; }
    }
    printf("%d\n", best);
    return 0;
}

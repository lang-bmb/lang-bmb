#include <stdio.h>
int main() {
    int n;
    scanf("%d", &n);
    int alt = 0, max = 0;
    for (int i = 0; i < n; i++) {
        int g;
        scanf("%d", &g);
        alt += g;
        if (alt > max) max = alt;
    }
    printf("%d\n", max);
    return 0;
}
